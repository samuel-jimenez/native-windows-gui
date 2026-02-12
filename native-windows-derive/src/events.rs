use std::collections::HashMap;

use itertools::Itertools;
use pm2::Span;
use proc_macro2 as pm2;
use quote::ToTokens;
use syn::{
    self, Attribute, Error, Expr, Ident, Path, Result,
    parse::{Parse, ParseBuffer, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
};

/// Wrapper over a basic event dispatcher
pub struct ControlEvents {
    partial: bool,
    handles: Vec<Ident>,
    callbacks: HashMap<Expr, Vec<EventCallback>>,
    partials_callbacks: Vec<pm2::TokenStream>,
    shortcuts: HashMap<ShortcutKeystrokes, Vec<EventCallback>>,
}

impl ControlEvents {
    pub fn with_capacity(partial: bool, cap: usize) -> ControlEvents {
        ControlEvents {
            partial,
            handles: Vec::with_capacity(1),
            callbacks: HashMap::with_capacity(cap),
            partials_callbacks: Vec::with_capacity(6),
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
        self.partials_callbacks.push(quote! {
            evt_ui.#id.process_event(_evt, &_evt_data, _handle);
        })
    }

    fn find_shortcuts_attr(attr: &&Attribute) -> bool {
        attr.path().is_ident("nwg_shortcuts")
    }

    fn find_events_attr(attr: &&Attribute) -> bool {
        attr.path().is_ident("nwg_events")
    }

    pub fn parse(&mut self, field: &syn::Field) -> Result<()> {
        self.parse_shortcuts(field)?;
        self.parse_events(field)
    }

    pub fn parse_global(&mut self, attr: &Attribute) -> Result<()> {
        self._parse_shortcut_impl(&None, attr)
    }
    pub fn parse_shortcuts(&mut self, field: &syn::Field) -> Result<()> {
        let attr = match field.attrs.iter().find(Self::find_shortcuts_attr) {
            Some(attr) => attr,
            None => return Ok(()),
        };
        let ident = field.ident.as_ref().ok_or(Error::new_spanned(
            field,
            "Cannot find member name when generating control",
        ))?;

        self._parse_shortcut_impl(&Some(&ident), attr)
    }

    fn _parse_shortcut_impl(&mut self, target: &Option<&Ident>, attr: &Attribute) -> Result<()> {
        let shortcut_definitions =
            attr.parse_args_with(Punctuated::<ShortcutDefinition, Token![,]>::parse_terminated)?;

        for shortcut_def in shortcut_definitions.into_iter() {
            let span = shortcut_def.callback_id.span();
            let mapped_event = shortcut_def.callback_id;

            let shortcut_callbacks = self.shortcuts.entry(mapped_event).or_insert(Vec::new());
            EventCallback::parse(
                span,
                target,
                &shortcut_def.field_name,
                shortcut_def.callbacks,
                shortcut_callbacks,
            )?
        }

        for mapped_event in [
            syn::parse_str("Event::OnKeyPress")?,
            syn::parse_str("Event::OnSysKeyPress")?,
        ] {
            let shortcut_event_callback = EventCallback {
                field: None,
                path: syn::parse_str("Self::handle_shortcuts").unwrap(),
                args: parse_quote! {&evt_ui, &_handle, &_evt_data},
            };

            let event_callbacks = self.callbacks.entry(mapped_event).or_insert(Vec::new());
            event_callbacks.push(shortcut_event_callback);
        }

        Ok(())
    }

    pub fn parse_events(&mut self, field: &syn::Field) -> Result<()> {
        let attr = match field.attrs.iter().find(Self::find_events_attr) {
            Some(attr) => attr,
            None => return Ok(()),
        };
        let target = &Some(field.ident.as_ref().ok_or(Error::new_spanned(
            field,
            "Cannot find member name when generating control",
        ))?);

        let callback_definitions =
            attr.parse_args_with(Punctuated::<CallbackDefinition, Token![,]>::parse_terminated)?;

        for callback_def in callback_definitions.into_iter() {
            let mapped_event = Self::map_event_enum(&callback_def.callback_id)?;

            let evt_callbacks = self.callbacks.entry(mapped_event).or_insert(Vec::new());
            EventCallback::parse(
                callback_def.callback_id.span(),
                target,
                &callback_def.field_name,
                callback_def.callbacks,
                evt_callbacks,
            )?;
        }
        Ok(())
    }

    pub fn shortcuts_len(&self) -> usize {
        self.shortcuts.len()
    }

    fn map_event_enum(event_ident: &Ident) -> Result<Expr> {
        Ok(match &event_ident.to_string() as &str {
            "MousePressLeftUp"
            | "MousePressLeftDown"
            | "MousePressRightUp"
            | "MousePressRightDown" => {
                parse_quote_spanned! {event_ident.span()=> Event::OnMousePress(MousePressEvent::#event_ident)}
            }
            "OnMousePress" => parse_quote_spanned! {event_ident.span()=> Event::OnMousePress(_)},
            _ => parse_quote_spanned! {event_ident.span()=> Event::#event_ident},
        })
    }
}

impl ToTokens for ControlEvents {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let handles = &self.handles;

        let mut events: Vec<&Expr> = Vec::with_capacity(self.callbacks.len());
        let partial_callbacks = &self.partials_callbacks;
        let mut callbacks = Vec::with_capacity(self.callbacks.len());
        for (event, cb) in self.callbacks.iter() {
            events.push(event);
            callbacks.push(EventCallbackCol(cb));
        }

        let mut shortcuts: Vec<&ShortcutKeystrokes> = Vec::with_capacity(self.shortcuts.len());
        let mut shortcut_callbacks = Vec::with_capacity(self.shortcuts.len());
        for (shortcut, cb) in self.shortcuts.iter() {
            shortcuts.push(shortcut);
            shortcut_callbacks.push(ShortcutCallbackCol(cb));
        }

        let events_tk = if self.partial {
            // There's no need to bind events handler in a partials
            quote! {
                let evt_ui = self;

                #( #partial_callbacks );*

                match _evt {
                    #( #events => #callbacks ),*
                    _ => {}
                }
            }
        } else {
            quote! {
                let window_handles: &[&ControlHandle] = &[#(&ui.#handles.handle),*];
                for handle in window_handles.iter() {
                    let evt_ui = Rc::downgrade(&inner);
                    let handle_events = move |_evt, _evt_data, _handle| {

                        if let Some(evt_ui) = evt_ui.upgrade() {
                            #( #partial_callbacks );*
                            match _evt {
                                #( #events => #callbacks ),*
                                _ => {}
                            }
                        }
                    };

                    ui.default_handlers.borrow_mut().push(full_bind_event_handler(handle, handle_events));
                }
            }
        };

        events_tk.to_tokens(tokens);
    }
}

/// The callback definition in a `nwg_shortcuts` attribute
/// A single pair of (PATH, CALLBACK_EVENT_ID): [CALLBACK_FUNCTIONS,]
#[allow(unused)]
struct ShortcutDefinition {
    field_name: Option<Expr>,
    callback_id: ShortcutKeystrokes,
    callbacks: Punctuated<CallbackFunction, Token![,]>,
}
impl ShortcutDefinition {
    // Try to parse the optional `(PATH, CALLBACK_EVENT_ID)` syntax
    fn maybe_parse_callback_name(
        input: &mut ParseStream,
    ) -> Result<(Option<Expr>, ShortcutKeystrokes)> {
        let event_content;
        let _paren_token = parenthesized!(event_content in input);

        let field_name: Expr = event_content.parse()?;
        let _comma: Token![,] = event_content.parse()?;
        let callback_id = event_content.parse()?;

        Ok((Some(field_name), callback_id))
    }
}
impl Parse for ShortcutDefinition {
    fn parse(mut input: ParseStream) -> Result<Self> {
        let content;

        let (field_name, callback_id) = match Self::maybe_parse_callback_name(&mut input) {
            Ok(v) => v,
            Err(_) => (None, input.parse()?),
        };

        let _sep: Token![:] = input.parse()?;
        let _bracket_token = bracketed!(content in input);

        Ok(Self {
            field_name,
            callback_id,
            callbacks: Punctuated::<CallbackFunction, Token![,]>::parse_terminated(&content)?,
        })
    }
}

/// A shortcut key combo
#[derive(Eq, Hash, PartialEq, Clone)]
struct ShortcutKeystrokes {
    modifiers: Path,
    key: Path,
}

impl ShortcutKeystrokes {
    fn span(&self) -> Span {
        self.modifiers.span()
    }
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
        let key = parse_quote_spanned!(span=>VirtualKey::#key_press);

        Ok(Self { modifiers, key })
    }
}

impl ToTokens for ShortcutKeystrokes {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let modifiers = &self.modifiers;
        let key = &self.key;

        let tk = quote! {    KeyCombo {
            modifiers: #modifiers,
            key: #key,
        }        };

        tk.to_tokens(tokens);
    }
}

/// A callback function definition
struct CallbackFunction {
    path: Path,
    args: Option<Punctuated<Ident, Token![,]>>,
}
impl CallbackFunction {
    fn maybe_parse_parens<'a>(input: &ParseBuffer<'a>) -> Result<ParseBuffer<'a>> {
        let content;
        parenthesized!(content in input);
        Ok(content)
    }
}
impl Parse for CallbackFunction {
    fn parse(input: ParseStream) -> Result<Self> {
        let path = input.parse()?;
        let args = match Self::maybe_parse_parens(input) {
            Ok(parse_buffer) => Some(Punctuated::<Ident, Token![,]>::parse_terminated(
                &parse_buffer,
            )?),
            Err(_) => None,
        };

        Ok(CallbackFunction { path, args })
    }
}

/// The callback definition in a `nwg_events` attribute
/// A single pair of (PATH, CALLBACK_EVENT_ID): [CALLBACK_FUNCTIONS,]
#[allow(unused)]
struct CallbackDefinition {
    field_name: Option<Expr>,
    callback_id: Ident,
    callbacks: Punctuated<CallbackFunction, Token![,]>,
}

impl CallbackDefinition {
    // Try to parse the optional `(PATH, CALLBACK_EVENT_ID)` syntax
    fn maybe_parse_callback_name(input: &mut ParseStream) -> Result<(Option<Expr>, Ident)> {
        let event_content;
        let _paren_token = parenthesized!(event_content in input);

        let field_name: Expr = event_content.parse()?;
        let _comma: Token![,] = event_content.parse()?;
        let callback_id = event_content.parse()?;

        Ok((Some(field_name), callback_id))
    }
}
impl Parse for CallbackDefinition {
    fn parse(mut input: ParseStream) -> Result<Self> {
        let content;

        let (field_name, callback_id) = match Self::maybe_parse_callback_name(&mut input) {
            Ok(v) => v,
            Err(_) => (None, input.parse()?),
        };

        let _sep: Token![:] = input.parse()?;
        let _bracket_token = bracketed!(content in input);

        Ok(CallbackDefinition {
            field_name,
            callback_id,
            callbacks: Punctuated::<CallbackFunction, Token![,]>::parse_terminated(&content)?,
        })
    }
}

/// Parsed callbacks for an event type
#[derive(Debug)]
struct EventCallback {
    // the field listening
    field: Option<Expr>,

    // the callback function:
    path: Path,
    args: Punctuated<Expr, Token![,]>,
}

impl EventCallback {
    fn parse(
        span: Span,
        target: &Option<&Ident>,
        field_name: &Option<Expr>,
        callbacks: Punctuated<CallbackFunction, Token![,]>,
        output: &mut Vec<Self>,
    ) -> Result<()> {
        let parsed_target = target.map(|ident| {
            let mut ident = ident.clone();
            ident.set_span(span);
            match field_name {
                Some(field) => parse_quote_spanned!(field.span()=>  evt_ui.#ident.#field ),
                None => parse_quote_spanned! (span=>   evt_ui.#ident ),
            }
        });

        // collect handles and callbacks
        for func in callbacks.into_iter() {
            let callback = EventCallback {
                field: parsed_target.clone(),
                path: func.path,
                args: Self::map_callback_args(&target, &func.args)?,
            };
            output.push(callback);
        }
        Ok(())
    }

    fn map_callback_args(
        event_target: &Option<&Ident>,
        args: &Option<Punctuated<Ident, Token![,]>>,
    ) -> Result<Punctuated<Expr, Token![,]>> {
        let mut p = Punctuated::new();
        if args.is_none() {
            p.push(parse_quote_spanned!(Span::call_site()=> &evt_ui)); //TODO

            return Ok(p);
        }

        for a in args.as_ref().unwrap().iter() {
            let span = a.span();
            match &*a.to_string() {
                "SELF" | "RC_SELF" => {
                    p.push(parse_quote_spanned!(span=> &evt_ui));
                }
                "CTRL" => {
                    let mut target = event_target
                        .ok_or_else(|| {
                            Error::new_spanned(
                                a,
                                format!("Cannot use {} on global event. Use `SELF` instead", a),
                            )
                        })?
                        .clone();
                    target.set_span(span);

                    p.push(parse_quote_spanned!(span=> &evt_ui.#target));
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
                            stringify!(["SELF", "CTRL", "HANDLE", "EVT", "EVT_DATA", "RC_SELF"])
                        ),
                    ));
                }
            }
        }

        Ok(p)
    }
}

/// Just a wrapper to implement ToTokens over Vec<&'a [EventCallback]>
struct EventCallbackCol<'a>(&'a [EventCallback]);

impl<'a> ToTokens for EventCallbackCol<'a> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let cb = &self.0;
        let span = Span::call_site();

        // Group callbacks by field
        let mut field_callbacks: HashMap<&Expr, Vec<(&Path, &Args)>> = HashMap::new();
        let mut path = &cb[0].path;
        let mut args = &cb[0].args;
        let mut unbound_p = false;

        for c in cb.iter() {
            match &c.field {
                Some(member) => {
                    let mc = field_callbacks.entry(&member).or_insert(Vec::new());
                    mc.push((&c.path, &c.args));
                }
                None => {
                    // remove dupes
                    if !unbound_p {
                        (path, args) = (&c.path, &c.args);
                        unbound_p = true
                    }
                }
            }
        }

        let callbacks: Vec<_> = field_callbacks
            .keys()
            .zip(field_callbacks.values().map(|c| FunctionCalls(c)))
            .map(|(field, funcall)| {
                quote_spanned! (field.span()=>
                if &_handle == &#field { #funcall })
            })
            .collect();

        match unbound_p {
            false => {
                let callback_0 = &callbacks[0];
                let callbacks = &callbacks[1..];

                quote_spanned! {span=>
                    #callback_0
                    #(else #callbacks)*
                }
            }
            _ => {
                quote_spanned! {span=>
                    if #path(#args){}
                    #(else #callbacks)*
                }
            }
        }
        .to_tokens(tokens);
    }
}

pub struct ControlEventShortcuts<'a>(pub &'a ControlEvents);

impl<'a> ToTokens for ControlEventShortcuts<'a> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let mut shortcuts: Vec<&ShortcutKeystrokes> = Vec::with_capacity(self.0.shortcuts.len());
        let mut shortcut_callbacks = Vec::with_capacity(self.0.shortcuts.len());
        for (shortcut, cb) in self.0.shortcuts.iter() {
            shortcuts.push(shortcut);
            shortcut_callbacks.push(ShortcutCallbackCol(cb));
        }

        quote! {
            fn handle_shortcuts(&self, _handle: &ControlHandle, _evt_data: &EventData) -> bool {

                let evt_ui = self;
                let _evt = match KeyCombo::read(_evt_data) {
                    Some(k) => k,
                    None => return false,
                };

                match _evt {
                    #( #shortcuts => #shortcut_callbacks ),*
                    _ => return false,
                }
                true
            }
        }
        .to_tokens(tokens);
    }
}

/// Just a wrapper to implement ToTokens over Vec<&'a [ShortcutCallback]>
struct ShortcutCallbackCol<'a>(&'a [EventCallback]);

impl<'a> ToTokens for ShortcutCallbackCol<'a> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let cb = &self.0;
        let span = Span::call_site();

        // Group callbacks by field
        let mut field_callbacks: HashMap<&Expr, Vec<(&Path, &Args)>> = HashMap::new();
        let mut global_callbacks: Vec<(&Path, &Args)> = Vec::new();

        for c in cb.iter() {
            match &c.field {
                Some(member) => {
                    let mc = field_callbacks.entry(&member).or_insert(Vec::new());
                    mc.push((&c.path, &c.args));
                }
                None => {
                    global_callbacks.push((&c.path, &c.args));
                }
            }
        }

        let callbacks: Vec<_> = field_callbacks
            .keys()
            .zip(field_callbacks.values().map(|c| FunctionCalls(c)))
            .map(|(field, funcall)| {
                quote_spanned! (field.span()=>
                if _handle == &#field { #funcall })
            })
            .collect();
        let global_funccalls = FunctionCalls(&global_callbacks);
        match callbacks.len() {
            0 => quote! { {#global_funccalls} },
            _ => {
                let callback_0 = &callbacks[0];
                let callbacks = &callbacks[1..];

                let global_cb = match global_callbacks.len() {
                    0 => quote! {},
                    _ => quote! {else {#global_funccalls}},
                };

                quote_spanned! {span=>
                    #callback_0
                    #(else #callbacks)*
                    #global_cb
                }
            }
        }
        .to_tokens(tokens);
    }
}

type Args = Punctuated<Expr, Token![,]>;
// A set of function calls
struct FunctionCalls<'a>(&'a [(&'a Path, &'a Args)]);

impl<'a> ToTokens for FunctionCalls<'a> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let paths = self.0.iter().map(|pa| pa.0);
        let args = self.0.iter().map(|pa| pa.1);
        let tk = quote! {
            #(#paths(#args);)*
        };

        tk.to_tokens(tokens);
    }
}
