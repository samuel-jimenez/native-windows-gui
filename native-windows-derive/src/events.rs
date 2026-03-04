use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap},
};

use itertools::Itertools;
use pm2::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    self, Attribute, Error, Expr, Ident, Path, Result,
    parse::{Parse, ParseBuffer, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
};

fn maybe_parse_parens<'a>(input: &ParseBuffer<'a>) -> Result<ParseBuffer<'a>> {
    let content;
    parenthesized!(content in input);
    Ok(content)
}

fn maybe_parse_bracketed<'a>(input: &ParseBuffer<'a>) -> Result<ParseBuffer<'a>> {
    let content;
    bracketed!(content in input);
    Ok(content)
}

/// Wrapper over a basic event dispatcher
pub struct ControlEvents {
    partial: bool,
    handles: Vec<Ident>,
    callbacks: HashMap<Ident, EventCallback>,
    partial_members: Vec<Ident>,
    shortcuts: HashMap<ShortcutKeystrokes, EventCallback>,
}

impl ControlEvents {
    pub fn with_capacity(partial: bool, cap: usize) -> ControlEvents {
        ControlEvents {
            partial,
            handles: Vec::with_capacity(1),
            callbacks: HashMap::with_capacity(cap),
            partial_members: Vec::new(),
            shortcuts: HashMap::with_capacity(cap),
        }
    }

    fn top_level_window(field: &syn::Field) -> bool {
        static TOP_LEVEL: &'static [&'static str] = &["Window", "FancyWindow", "MessageWindow"];

        match &field.ty {
            syn::Type::Path(p) => {
                let seg_len = p.path.segments.len();
                let seg = &p.path.segments[seg_len - 1];

                TOP_LEVEL.iter().any(|top| seg.ident == top)
            }
            _ => false,
        }
    }

    pub fn add_top_level_handle(&mut self, field: &syn::Field) {
        let attrs = &field.attrs;
        if attrs.len() == 0 {
            return;
        }

        if Self::top_level_window(field) {
            self.handles.push(
                field
                    .ident
                    .as_ref()
                    .expect("Cannot find member name when generating control")
                    .clone(),
            );
        }
    }

    pub fn add_partial(&mut self, id: &Ident) {
        self.partial_members.push(id.clone())
    }

    pub fn find_shortcuts_attr(attr: &&Attribute) -> bool {
        attr.path().is_ident("nwg_shortcuts")
    }

    pub fn find_events_attr(attr: &&Attribute) -> bool {
        attr.path().is_ident("nwg_events")
    }

    pub fn parse(&mut self, field: &syn::Field) -> Result<()> {
        self.parse_shortcuts(field)?;
        self.parse_events(field)
    }

    pub fn parse_global(&mut self, attrs: Vec<&Attribute>) -> Result<()> {
        let mut iter = attrs.into_iter();
        if let Some(attr) = iter.by_ref().find(Self::find_shortcuts_attr) {
            self.parse_shortcut_impl(&None, attr)?
        }
        if let Some(attr) = iter.by_ref().find(Self::find_events_attr) {
            self.parse_events_impl(&None, attr)?
        }
        Ok(())
    }

    pub fn parse_shortcuts(&mut self, field: &syn::Field) -> Result<()> {
        let attr = match field.attrs.iter().find(Self::find_shortcuts_attr) {
            Some(attr) => attr,
            None => return Ok(()),
        };
        let ident = &Some(field.ident.as_ref().ok_or(Error::new_spanned(
            field,
            "Cannot find member name when generating control",
        ))?);

        self.parse_shortcut_impl(ident, attr)
    }
    fn parse_shortcut_impl(&mut self, target: &Option<&Ident>, attr: &Attribute) -> Result<()> {
        let shortcut_definitions =
            attr.parse_args_with(Punctuated::<ShortcutDefinition, Token![,]>::parse_terminated)?;

        for shortcut_def in shortcut_definitions.into_iter() {
            for mapped_event in shortcut_def.key_combo.into_iter() {
                let span = mapped_event.span();
                let shortcut_callbacks = self
                    .shortcuts
                    .entry(mapped_event)
                    .or_insert(EventCallback::new());
                shortcut_callbacks.parse(
                    target,
                    Self::map_event_target(span, target, &shortcut_def.field_names),
                    &shortcut_def.callbacks,
                )?
            }
        }
        Ok(())
    }

    pub fn parse_events(&mut self, field: &syn::Field) -> Result<()> {
        let attr = match field.attrs.iter().find(Self::find_events_attr) {
            Some(attr) => attr,
            None => return Ok(()),
        };
        let ident = &Some(field.ident.as_ref().ok_or(Error::new_spanned(
            field,
            "Cannot find member name when generating control",
        ))?);
        self.parse_events_impl(ident, attr)
    }
    pub fn parse_events_impl(&mut self, target: &Option<&Ident>, attr: &Attribute) -> Result<()> {
        let callback_definitions =
            attr.parse_args_with(Punctuated::<CallbackDefinition, Token![,]>::parse_terminated)?;

        for callback_def in callback_definitions.into_iter() {
            for mapped_event in callback_def.callback_id.into_iter() {
                let span = mapped_event.span();
                let evt_callbacks = self
                    .callbacks
                    .entry(mapped_event)
                    .or_insert(EventCallback::new());
                evt_callbacks.parse(
                    target,
                    Self::map_event_target(span, target, &callback_def.field_names),
                    &callback_def.callbacks,
                )?;
            }
        }
        Ok(())
    }

    pub fn shortcuts_len(&self) -> usize {
        self.shortcuts.len()
    }

    fn map_event_target(
        span: Span,
        target: &Option<&Ident>,
        field_names: &Option<Punctuated<Expr, Token![,]>>,
    ) -> Vec<Option<EventField>> {
        target.map_or_else(
            || {
                field_names.as_ref().map_or_else(
                    || vec![None],
                    |f| {
                        f.iter()
                            .map(|field| {
                                Some(EventField(
                                    parse_quote_spanned!(field.span()=>   evt_ui.#field ),
                                ))
                            })
                            .collect()
                    },
                )
            },
            |ident| {
                let mut ident = ident.clone();
                ident.set_span(span);
                field_names.as_ref().map_or_else(
                    || {
                        vec![Some(EventField(
                            parse_quote_spanned!(ident.span()=>   evt_ui.#ident ),
                        ))]
                    },
                    |f| {
                        f.iter()
                            .map(|field| {
                                Some(EventField(
                                    parse_quote_spanned!(field.span()=>  evt_ui.#ident.#field ),
                                ))
                            })
                            .collect()
                    },
                )
            },
        )
    }

    fn map_event_enum(event_ident: &Ident) -> Expr {
        match &event_ident.to_string() as &str {
            "MousePressLeftUp"
            | "MousePressLeftDown"
            | "MousePressRightUp"
            | "MousePressRightDown" => {
                parse_quote_spanned! {event_ident.span()=> Event::OnMousePress(MousePressEvent::#event_ident)}
            }
            "OnMousePress" => parse_quote_spanned! {event_ident.span()=> Event::OnMousePress(_)},
            _ => parse_quote_spanned! {event_ident.span()=> Event::#event_ident},
        }
    }
}

/// Parsed callbacks for an event type
struct EventCallback(HashMap<Option<EventField>, BTreeSet<FunctionCall>>);

impl EventCallback {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn parse(
        &mut self,
        target: &Option<&Ident>,
        parsed_targets: Vec<Option<EventField>>,
        callbacks: &Vec<CallbackFunction>,
    ) -> Result<()> {
        // collect handles and callbacks
        for parsed_target in parsed_targets.into_iter() {
            for func in callbacks.iter() {
                let callbacks = self
                    .0
                    .entry(parsed_target.clone())
                    .or_insert(BTreeSet::new());
                let callback = FunctionCall(
                    func.path.clone(),
                    Self::map_callback_args(func.path.span(), target, &parsed_target, &func.args)?,
                );
                callbacks.insert(callback);
            }
        }
        Ok(())
    }

    fn map_callback_args(
        span: Span,
        bind_target: &Option<&Ident>,
        event_target: &Option<EventField>,
        args: &Option<Punctuated<Ident, Token![,]>>,
    ) -> Result<Punctuated<Expr, Token![,]>> {
        let mut p = Punctuated::new();
        match args.as_ref() {
            None => p.push(parse_quote_spanned!(span=> &evt_ui)),
            Some(args) => {
                for a in args.iter() {
                    let span = a.span();
                    match &*a.to_string() {
                        "SELF" => {
                            p.push(parse_quote_spanned!(span=> &evt_ui));
                        }
                        "CTRL" => {
                            p.push(bind_target.map_or_else(
                                || parse_quote_spanned!(span=> &evt_ui), // SELF
                                |ident| {
                                    let mut ident = ident.clone();
                                    ident.set_span(span);
                                    parse_quote_spanned!(span=>   &evt_ui.#ident )
                                },
                            ));
                        }
                        "TARGET" => {
                            let target: Expr = syn::LitStr::new(
                                // this is the easiest way to replace a span
                                &event_target
                                    .as_ref()
                                    .ok_or_else(|| {
                                        Error::new(
                                            span,
                                            format!("Cannot use {} on global event", a),
                                        )
                                    })?
                                    .to_string(),
                                span,
                            )
                            .parse()?;
                            p.push(parse_quote_spanned!(span=> &#target));
                        }
                        "HANDLE" => {
                            p.push(parse_quote_spanned!(span=> &_handle));
                        }
                        "EVT" => {
                            p.push(parse_quote_spanned!(span=> _evt));
                        }
                        "EVT_DATA" => {
                            p.push(parse_quote_spanned!(span=> &_evt_data));
                        }
                        _ => {
                            return Err(Error::new_spanned(
                                a,
                                format!(
                                    "Unknown callback argument: {}. Should be one of those values: {}",
                                    a,
                                    stringify!([
                                        "SELF", "CTRL", "TARGET", "HANDLE", "EVT", "EVT_DATA"
                                    ])
                                ),
                            ));
                        }
                    }
                }
            }
        };
        Ok(p)
    }
}

/// The callback definition in a `nwg_shortcuts` attribute
/// An instance of  (TARGET_PATH,*)? KEY_COMBO: [CALLBACK_FUNCTIONS,*]
/// or  (TARGET_PATH,*)? [KEY_COMBO,*]: [CALLBACK_FUNCTIONS,*]
#[allow(unused)]
struct ShortcutDefinition {
    field_names: Option<Punctuated<Expr, Token![,]>>,
    key_combo: Punctuated<ShortcutKeystrokes, Token![,]>,
    callbacks: Vec<CallbackFunction>,
}

impl Parse for ShortcutDefinition {
    fn parse(mut input: ParseStream) -> Result<Self> {
        let content;
        // Try to parse the optional `(TARGET_PATH)` syntax
        let field_names = match maybe_parse_parens(&mut input) {
            Ok(e) => Some(Punctuated::<Expr, Token![,]>::parse_terminated(&e)?),
            Err(_) => None,
        };
        let key_combo = match maybe_parse_bracketed(&mut input) {
            Ok(e) => Punctuated::parse_terminated(&e)?,
            Err(_) => Punctuated::parse_separated_nonempty(&input)?,
        };

        let _sep: Token![:] = input.parse()?;
        let _bracket_token = bracketed!(content in input);

        Ok(Self {
            field_names,
            key_combo,
            callbacks: Punctuated::<CallbackFunction, Token![,]>::parse_terminated(&content)?
                .into_iter()
                .sorted()
                .collect(),
        })
    }
}

/// The callback definition in a `nwg_events` attribute
/// An instance of  (TARGET_PATH,*)? CALLBACK_EVENT_ID: [CALLBACK_FUNCTIONS,*]
/// or  (TARGET_PATH,*)? [CALLBACK_EVENT_ID,*]: [CALLBACK_FUNCTIONS,*]
#[allow(unused)]
struct CallbackDefinition {
    field_names: Option<Punctuated<Expr, Token![,]>>,
    callback_id: Punctuated<Ident, Token![,]>,
    callbacks: Vec<CallbackFunction>,
}

impl Parse for CallbackDefinition {
    fn parse(mut input: ParseStream) -> Result<Self> {
        let content;

        // Try to parse the optional `(TARGET_PATH)` syntax
        let field_names = match maybe_parse_parens(&mut input) {
            Ok(e) => Some(Punctuated::<Expr, Token![,]>::parse_terminated(&e)?),
            Err(_) => None,
        };
        let callback_id = match maybe_parse_bracketed(&mut input) {
            Ok(e) => Punctuated::parse_terminated(&e)?,
            Err(_) => Punctuated::parse_separated_nonempty(&input)?,
        };

        let _sep: Token![:] = input.parse()?;
        let _bracket_token = bracketed!(content in input);

        Ok(CallbackDefinition {
            field_names,
            callback_id,
            callbacks: Punctuated::<CallbackFunction, Token![,]>::parse_terminated(&content)?
                .into_iter()
                .sorted()
                .collect(),
        })
    }
}

/// A shortcut key combo
#[derive(Eq, Hash, PartialEq, Clone)]
struct ShortcutKeystrokes {
    modifiers: Path,
    key: Path,
}

impl Parse for ShortcutKeystrokes {
    fn parse(input: ParseStream) -> Result<Self> {
        let modifier_list = ["CTRL", "ALT", "SHIFT"];

        let args = Punctuated::<Ident, Token![+]>::parse_separated_nonempty(&input)?;
        let span = args.span();

        let mut key_candidates = Vec::new();
        let mut modifier_candidates = Vec::with_capacity(3);

        for arg in &args {
            let arg_string = &*arg.to_string();
            let arg_upcase = arg_string.to_uppercase();
            match modifier_list.into_iter().position(|x| x == arg_upcase) {
                Some(i) => {
                    modifier_candidates.push((i, arg_upcase));
                }
                None => {
                    if key_candidates.len() > 0 {
                        Err(Error::new_spanned(
                            &arg,
                            format!(
                                "extra key found: `{}` when mapping nwg_shortcuts",
                                arg_string
                            ),
                        ))?
                    }
                    key_candidates.push(Ident::new(&arg_string, arg.span()));
                }
            }
        }

        if key_candidates.len() == 0 {
            Err(Error::new(
                span,
                format!("no key found when mapping nwg_shortcuts"),
            ))?
        }
        let key_press = &key_candidates[0];

        let modifier = Ident::new(
            &*match modifier_candidates.len() {
                0 => "NONE".to_string(),
                _ => modifier_candidates.into_iter().map(|x| x.1).join("_"),
            },
            span,
        );

        let modifiers = parse_quote_spanned!(span=>ModifierKeys::#modifier);
        let key = parse_quote_spanned!(span=>KeyPress::#key_press);

        Ok(Self { modifiers, key })
    }
}

impl ToTokens for ShortcutKeystrokes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let modifiers = &self.modifiers;
        let key = &self.key;

        let tk = quote! {    KeyCombo {
            modifiers: #modifiers,
            key: #key,
        }        };

        tk.to_tokens(tokens);
    }
}
impl ShortcutKeystrokes {
    fn span(&self) -> Span {
        self.modifiers.span()
    }
}

impl Ord for ShortcutKeystrokes {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key
            .segments
            .last()
            .unwrap()
            .ident
            .cmp(&other.key.segments.last().unwrap().ident)
            .then(
                self.modifiers
                    .segments
                    .last()
                    .unwrap()
                    .ident
                    .cmp(&other.modifiers.segments.last().unwrap().ident),
            )
    }
}
impl PartialOrd for ShortcutKeystrokes {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A callback function definition
#[derive(Clone)]
struct CallbackFunction {
    path: Path,
    args: Option<Punctuated<Ident, Token![,]>>,
}

impl Parse for CallbackFunction {
    fn parse(input: ParseStream) -> Result<Self> {
        let path = input.parse()?;
        let args = match maybe_parse_parens(input) {
            Ok(parse_buffer) => Some(Punctuated::<Ident, Token![,]>::parse_terminated(
                &parse_buffer,
            )?),
            Err(_) => None,
        };

        Ok(CallbackFunction { path, args })
    }
}

impl Ord for CallbackFunction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path
            .segments
            .len()
            .cmp(&other.path.segments.len())
            .reverse()
            .then(
                self.path
                    .segments
                    .first()
                    .unwrap()
                    .ident
                    .cmp(&other.path.segments.first().unwrap().ident)
                    .then(
                        self.path
                            .segments
                            .last()
                            .unwrap()
                            .ident
                            .cmp(&other.path.segments.last().unwrap().ident),
                    ),
            )
    }
}
impl PartialOrd for CallbackFunction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for CallbackFunction {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}
impl Eq for CallbackFunction {}

/// The name of the field receiving the callback
#[derive(Clone, Hash, PartialEq, Eq)]
struct EventField(Expr);

impl<'a> ToTokens for EventField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}
impl ToString for EventField {
    fn to_string(&self) -> String {
        self.0.to_token_stream().to_string()
    }
}
impl Ord for EventField {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_string().cmp(&other.to_string())
    }
}
impl PartialOrd for EventField {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Build event handling
impl ToTokens for ControlEvents {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let handles = &self.handles;
        let mut events: Vec<Expr> = Vec::with_capacity(self.callbacks.len());
        let partial_members = &self.partial_members;
        let mut callbacks = Vec::with_capacity(self.callbacks.len());

        for (event, cb) in self.callbacks.iter().sorted_by_key(|x| x.0) {
            events.push(Self::map_event_enum(event));
            callbacks.push(EventCallbackCol(cb));
        }

        let events_callbacks = quote! {
            #(  evt_ui.#partial_members.process_event(_evt, &_evt_data, _handle); )*
            match _evt {
                #( #events => #callbacks ),*
                _ => {}
            }
        };

        let events_tk = if self.partial {
            // There's no need to bind events handler in a partials
            quote! {
                let evt_ui = self;

                #events_callbacks
            }
        } else {
            quote! {
                let window_handles: &[&ControlHandle] = &[#(&ui.#handles.handle),*];
                for handle in window_handles.iter() {
                    let evt_ui = Rc::downgrade(&inner);
                    let handle_events = move |_evt, _evt_data, _handle| {

                        if let Some(evt_ui) = evt_ui.upgrade() {
                            #events_callbacks
                        }
                    };

                    ui.default_handlers.borrow_mut().push(full_bind_event_handler(handle, handle_events));
                }
            }
        };

        events_tk.to_tokens(tokens);
    }
}

/// Just a wrapper to implement ToTokens over &'a ControlEvents for shortcuts
pub struct ControlEventShortcuts<'a>(pub &'a ControlEvents);

/// Build preprocess_event function
impl<'a> ToTokens for ControlEventShortcuts<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let partial_members = &self.0.partial_members;

        let mut shortcuts: Vec<&ShortcutKeystrokes> = Vec::with_capacity(self.0.shortcuts.len());
        let mut shortcut_callbacks = Vec::with_capacity(self.0.shortcuts.len());
        for (shortcut, cb) in self.0.shortcuts.iter().sorted_by_key(|x| x.0) {
            shortcuts.push(shortcut);
            shortcut_callbacks.push(ShortcutCallbackCol(&cb));
        }

        quote! {
            fn preprocess_event(&self, _evt: &KeyCombo, _handle: ControlHandle) -> bool {
                let evt_ui = self;
                #(  evt_ui.#partial_members.preprocess_event(_evt, _handle) ||)*
                match _evt {
                    #( #shortcuts => #shortcut_callbacks ),*
                    _ => false,
                }
            }
        }
        .to_tokens(tokens);
    }
}

/// Just a wrapper to implement ToTokens over &'a EventCallback for events
struct EventCallbackCol<'a>(&'a EventCallback);

impl<'a> ToTokens for EventCallbackCol<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let cb = self.0;
        let span = Span::call_site();

        // Group fields by callbacks
        let mut fn_callbacks: HashMap<FunctionCalls, Vec<&EventField>> = HashMap::new();
        for (field, funcalls) in cb.0.iter() {
            if field.is_some() {
                let cb_vec = fn_callbacks
                    .entry(FunctionCalls(funcalls))
                    .or_insert(Vec::new());
                cb_vec.push(&field.as_ref().unwrap());
            }
        }

        let callbacks: Vec<_> = fn_callbacks
            .into_iter()
            .sorted_by_key(|x| x.0.clone())
            .map(|(funcalls, mut fields)| {
                fields.sort();
                quote_spanned! (fields[0].span()=>
                if #(&_handle == &#fields) ||* { #funcalls })
            })
            .collect();

        quote_spanned! {span=>
            #(#callbacks) else*
        }
        .to_tokens(tokens);
    }
}

/// Just a wrapper to implement ToTokens over &'a EventCallback for shortcuts
struct ShortcutCallbackCol<'a>(&'a EventCallback);

impl<'a> ToTokens for ShortcutCallbackCol<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let cb = &self.0;
        let span = Span::call_site();
        let empty_set = BTreeSet::new();

        // Group fields by callbacks
        let mut fn_callbacks: HashMap<FunctionCalls, Vec<&EventField>> = HashMap::new();
        let global_funccalls = FunctionCalls(cb.0.get(&None).unwrap_or(&empty_set));

        for (field, funcalls) in cb.0.iter() {
            if field.is_some() {
                let cb_vec = fn_callbacks
                    .entry(FunctionCalls(funcalls))
                    .or_insert(Vec::new());
                cb_vec.push(&field.as_ref().unwrap());
            }
        }

        let callbacks: Vec<_> = fn_callbacks
            .into_iter()
            .sorted_by_key(|x| x.0.clone())
            .map(|(funcalls, mut fields)| {
                fields.sort();
                quote_spanned! (fields[0].span()=>
                if #(&_handle == &#fields) ||* { #funcalls true })
            })
            .collect();
        match callbacks.len() {
            0 => quote! { {#global_funccalls true} },
            _ => {
                let global_cb = match global_funccalls.0.len() {
                    0 => quote! {false},
                    _ => quote! {#global_funccalls true},
                };

                quote_spanned! {span=>
                    #(#callbacks else) *
                    {#global_cb}
                }
            }
        }
        .to_tokens(tokens);
    }
}

/// function args
type Args = Punctuated<Expr, Token![,]>;

/// A single function call
#[derive(Clone, Hash)]
struct FunctionCall(Path, Args);
impl Ord for FunctionCall {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .segments
            .len()
            .cmp(&other.0.segments.len())
            .reverse()
            .then(
                self.0
                    .segments
                    .first()
                    .unwrap()
                    .ident
                    .cmp(&other.0.segments.first().unwrap().ident)
                    .then(
                        self.0
                            .segments
                            .last()
                            .unwrap()
                            .ident
                            .cmp(&other.0.segments.last().unwrap().ident),
                    ),
            )
    }
}
impl PartialOrd for FunctionCall {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for FunctionCall {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for FunctionCall {}

impl ToTokens for FunctionCall {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let path = &self.0;
        let args = &self.1;
        quote_spanned!(args.span()=> #path(#args)).to_tokens(tokens);
    }
}

/// A set of function calls
#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct FunctionCalls<'a>(&'a BTreeSet<FunctionCall>);

impl<'a> ToTokens for FunctionCalls<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let funcalls = self.0;
        quote! {
            #(#funcalls;)*
        }
        .to_tokens(tokens);
    }
}
