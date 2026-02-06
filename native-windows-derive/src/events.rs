use std::collections::HashMap;

use proc_macro2 as pm2;
use quote::ToTokens;
use syn::{
    self, Attribute, Error, Expr, Ident, Path, Result,
    parse::{Parse, ParseBuffer, ParseStream},
    punctuated::Punctuated,
};

/// A callback function definition
struct CallbackFunction {
    path: Path,
    args: Option<Punctuated<Ident, Token![,]>>,
}

impl Parse for CallbackFunction {
    fn parse(input: ParseStream) -> Result<Self> {
        fn maybe_parse_parens<'a>(input: &ParseBuffer<'a>) -> Result<ParseBuffer<'a>> {
            let content;
            parenthesized!(content in input);
            Ok(content)
        }

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

/// The callback definition in a `nwg_events` attribute
/// A single pair of (PATH, CALLBACK_EVENT_ID): [CALLBACK_FUNCTIONS,]
#[allow(unused)]
struct CallbackDefinition {
    field_name: Option<Expr>,
    callback_id: Ident,
    callbacks: Punctuated<CallbackFunction, Token![,]>,
}

impl Parse for CallbackDefinition {
    fn parse(mut input: ParseStream) -> Result<Self> {
        let content;

        /// Try to parse the optional `(PATH, CALLBACK_EVENT_ID)` syntax
        fn parse_callback_name(input: &mut ParseStream) -> Result<(Option<Expr>, Ident)> {
            let event_content;
            let _paren_token = parenthesized!(event_content in input);

            let field_name: Expr = event_content.parse()?;
            let _comma: Token![,] = event_content.parse()?;
            let callback_id = event_content.parse()?;

            Ok((Some(field_name), callback_id))
        }

        let (field_name, callback_id) = match parse_callback_name(&mut input) {
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

/// Parsed callbacks for a event type
#[derive(Debug)]
struct EventCallback {
    member: Expr,
    path: Path,
    args: Punctuated<Expr, Token![,]>,
}

/// Wrapper over a basic event dispatcher
pub struct ControlEvents {
    partial: bool,
    handles: Vec<Ident>,
    callbacks: HashMap<Expr, Vec<EventCallback>>,
    partials_callbacks: Vec<pm2::TokenStream>,
    callback_args_cache: HashMap<usize, Expr>,
}

impl ControlEvents {
    pub fn with_capacity(partial: bool, cap: usize) -> ControlEvents {
        let mut cache = HashMap::with_capacity(4);
        cache.insert(0, syn::parse_str("&evt_ui").unwrap());
        cache.insert(2, syn::parse_str("&_handle").unwrap());
        cache.insert(3, syn::parse_str("_evt").unwrap());
        cache.insert(4, syn::parse_str("&_evt_data").unwrap());

        ControlEvents {
            partial,
            handles: Vec::with_capacity(1),
            callbacks: HashMap::with_capacity(cap),
            partials_callbacks: Vec::with_capacity(6),
            callback_args_cache: cache,
        }
    }

    pub fn add_top_level_handle(&mut self, field: &syn::Field) {
        let attrs = &field.attrs;
        if attrs.len() == 0 {
            return;
        }

        let member = field
            .ident
            .as_ref()
            .expect("Cannot find member name when generating control");

        if top_level_window(field) {
            self.handles.push(member.clone());
        }
    }

    pub fn add_partial(&mut self, id: &Ident) {
        self.partials_callbacks.push(quote! {
            evt_ui.#id.process_event(_evt, &_evt_data, _handle);
        })
    }

    fn find_attr(attr: &&Attribute) -> bool {
        attr.path().is_ident("nwg_events")
    }

    pub fn parse(&mut self, field: &syn::Field) -> Result<()> {
        let attr = match field.attrs.iter().find(Self::find_attr) {
            Some(attr) => attr,
            None => return Ok(()),
        };
        let member = field.ident.as_ref().ok_or(Error::new_spanned(
            field,
            "Cannot find member name when generating control",
        ))?;

        let callback_definitions =
            attr.parse_args_with(Punctuated::<CallbackDefinition, Token![,]>::parse_terminated)?;

        for callback_def in callback_definitions.iter() {
            let mapped_event = map_event_enum(&callback_def.callback_id)?;
            let evt_callbacks = self
                .callbacks
                .entry(mapped_event)
                .or_insert(Vec::with_capacity(3));

            for cb_fn in callback_def.callbacks.iter() {
                let callback = EventCallback {
                    member: Self::parse_member(&callback_def.field_name, &member),
                    path: cb_fn.path.clone(),
                    args: map_callback_args(&member, &cb_fn.args, &self.callback_args_cache)?,
                };

                evt_callbacks.push(callback);
            }
        }
        Ok(())
    }

    fn parse_member(base: &Option<Expr>, id: &Ident) -> Expr {
        let tokens = match base {
            Some(b) => quote! { evt_ui.#id.#b },
            None => quote! {  evt_ui.#id },
        };

        syn::parse2(tokens).expect("Failed to generate event match code")
    }
}

impl ToTokens for ControlEvents {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let handles = &self.handles;

        let mut pats: Vec<&Expr> = Vec::with_capacity(self.callbacks.len());
        let partial_callbacks = &self.partials_callbacks;
        let mut callbacks = Vec::with_capacity(self.callbacks.len());
        for (pat, cb) in self.callbacks.iter() {
            pats.push(pat);
            callbacks.push(EventCallbackCol(cb));
        }

        let events_tk = if self.partial {
            // There's no need to bind events handler in a partials
            quote! {
                let evt_ui = self;

                #( #partial_callbacks );*

                match _evt {
                    #( #pats => #callbacks ),*
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
                                #( #pats => #callbacks ),*
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

/// Just a wrapper to implement ToTokens over Vec<&'a [EventCallback]>
struct EventCallbackCol<'a>(&'a [EventCallback]);

impl<'a> ToTokens for EventCallbackCol<'a> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let cb = &self.0;

        let tk = match cb.len() {
            0 => quote! { {} },
            1 => {
                let member = &cb[0].member;
                let path = &cb[0].path;
                let args = &cb[0].args;
                quote! { if &_handle == &#member { #path(#args) } }
            }
            _ => {
                // Group callbacks by members
                let mut members_callbacks: HashMap<&Expr, Vec<(&Path, &Args)>> = HashMap::new();
                for c in cb.iter() {
                    let mc = members_callbacks.entry(&c.member).or_insert(Vec::new());
                    mc.push((&c.path, &c.args));
                }

                let members: Vec<&&Expr> = members_callbacks.keys().collect();
                let values: Vec<PathArgs> =
                    members_callbacks.values().map(|c| PathArgs(c)).collect();

                let member0 = members[0];
                let value0 = &values[0];
                let members = &members[1..];
                let values = &values[1..];

                quote! {
                    if &_handle == &#member0 { #value0 }
                    #(else if &_handle == &#members { #values })*
                }
            }
        };

        tk.to_tokens(tokens);
    }
}

type Args = Punctuated<Expr, Token![,]>;
struct PathArgs<'a>(&'a [(&'a Path, &'a Args)]);

impl<'a> ToTokens for PathArgs<'a> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let paths = self.0.iter().map(|pa| pa.0);
        let args = self.0.iter().map(|pa| pa.1);
        let tk = quote! {
            #(#paths(#args);)*
        };

        tk.to_tokens(tokens);
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

fn map_callback_args(
    member: &Ident,
    args: &Option<Punctuated<Ident, Token![,]>>,
    cache: &HashMap<usize, Expr>,
) -> Result<Punctuated<Expr, Token![,]>> {
    let mut p = Punctuated::new();
    if args.is_none() {
        p.push(cache[&0].clone());
        return Ok(p);
    }

    let values = ["SELF", "CTRL", "HANDLE", "EVT", "EVT_DATA", "RC_SELF"];
    for a in args.as_ref().unwrap().iter() {
        let pos = values.iter().position(|v| &a == &v);
        match pos {
            Some(0) | Some(5) => {
                p.push(cache[&0].clone());
            }
            Some(1) => {
                let param = format!("&evt_ui.{}", member);
                p.push(syn::parse_str(&param).unwrap());
            }
            Some(2) => {
                p.push(cache[&2].clone());
            }
            Some(3) => {
                p.push(cache[&3].clone());
            }
            Some(4) => {
                p.push(cache[&4].clone());
            }
            Some(_) => {
                unreachable!();
            }
            None => {
                return Err(Error::new_spanned(
                    a,
                    format!(
                        "Unknown callback argument: {}. Should be one of those values: {:?}",
                        a, values
                    ),
                ));
            }
        }
    }

    Ok(p)
}

fn map_event_enum(ident: &Ident) -> Result<Expr> {
    let evt = ident.to_string();
    let pat = match &evt as &str {
        "MousePressLeftUp" | "MousePressLeftDown" | "MousePressRightUp" | "MousePressRightDown" => {
            format!("Event::OnMousePress(MousePressEvent::{})", evt)
        }
        "OnMousePress" => "Event::OnMousePress(_)".into(),
        _ => format!("Event::{}", evt),
    };

    Ok(syn::parse_str(&pat)
        .map_err(|e| Error::new_spanned(ident, format!("{} while mapping nwg_events", e)))?)
}
