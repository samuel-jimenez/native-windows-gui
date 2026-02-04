/*!
 Derives setters for struct.
 use `#[derive(Setters)]`
 and optionally #[setter(skip,into, strip_option,name=fn_name)]

 #[derive(Setters)]
 pub struct LabeledEditBuilder<'a> {
     #[setter(name=label)]
     label_text: &'a str,
     #[setter(skip)]
     label_v_align: u32,
     text: &'a str,
     #[setter(strip_option)]
     placeholder_text: Option<&'a str>,
     size: (i32, i32),
     #[setter(strip_option)]
     flags: Option<u32>,
     ex_flags: u32,
     limit: usize,
     password: Option<char>,
     readonly: bool,
     #[setter(into, strip_option)]
     font: Option<&'a String>,
 }
*/

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Attribute, Data, DataStruct, DeriveInput, Error, Field, Fields, FieldsNamed, Generics, Ident,
    ImplGenerics, PathArguments, Type, TypeGenerics, WhereClause, parse_macro_input, parse2,
};

#[proc_macro_derive(Setters, attributes(setter))]
pub fn derive_setters(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_struct(parse_macro_input!(input as DeriveInput)).into()
}

fn derive_struct(input: DeriveInput) -> proc_macro2::TokenStream {
    match SetterStruct::parse(&input) {
        Ok(val) => val.into_token_stream(),
        Err(err) => err.into_compile_error(),
    }
}
struct SetterStruct<'a> {
    pub ident: &'a Ident,
    pub impl_generics: ImplGenerics<'a>,
    pub ty_generics: TypeGenerics<'a>,
    pub where_clause: Option<&'a WhereClause>,
    fields: Vec<SetterField<'a>>,
}
impl<'a> SetterStruct<'a> {
    pub fn parse(base: &'a DeriveInput) -> Result<Self, Error> {
        let ident = &base.ident;
        let generics = &base.generics;
        match &base.data {
            Data::Struct(struct_data) => {
                Ok(SetterStruct::parse_fields(ident, generics, struct_data)?)
            }
            _ => Err(Error::new(
                ident.span(),
                format!("`{}` must be an Struct to derive `Setter`", ident),
            )),
        }
    }
    fn parse_fields(
        ident: &'a Ident,
        generics: &'a Generics,
        struct_data: &'a DataStruct,
    ) -> Result<Self, Error> {
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        match &struct_data.fields {
            Fields::Named(fields) => Ok(Self {
                ident,
                impl_generics,
                fields: SetterField::parse(ident, ty_generics.clone(), fields)?,
                ty_generics,
                where_clause,
            }),
            _ => Err(Error::new(
                ident.span(),
                format!("`{}` must be an Struct to derive `Setter`", ident),
            )),
        }
    }
}

impl ToTokens for SetterStruct<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        let impl_generics = &self.impl_generics;
        let ty_generics = &self.ty_generics;
        let where_clause = &self.where_clause;
        let fields = &self.fields;

        let out_tokens = quote! {

            impl #impl_generics  #ident #ty_generics #where_clause {
                 #(#fields)*
            }
        };

        out_tokens.to_tokens(tokens);
    }
}

struct SetterField<'a> {
    field_ident: Ident,
    ty: Type,
    setter_ident: Ident,
    generics: Generics,
    into: bool,
    strip_option: bool,
    is_option: bool,
    struct_ident: &'a Ident,
    struct_generics: TypeGenerics<'a>,
}

impl<'a> SetterField<'a> {
    pub fn parse(
        struct_ident: &'a Ident,
        struct_generics: TypeGenerics<'a>,
        fields: &'a FieldsNamed,
    ) -> Result<Vec<Self>, Error> {
        fields
            .named
            .iter()
            .map(|x| SetterField::parse_fields(struct_ident, struct_generics.clone(), &x))
            .map(|x| x.transpose())
            .flatten()
            .collect()
    }

    pub fn parse_fields(
        struct_ident: &'a Ident,
        struct_generics: TypeGenerics<'a>,

        field: &'a Field,
    ) -> Result<Option<Self>, Error> {
        let attrs = SetterAttr::parse_attr(field.attrs.iter().find(SetterField::find_attr))?;
        let field_ident = field.ident.clone().unwrap();
        let mut setter_ident = field_ident.clone();
        let mut generics = Generics::default();
        let mut ty = field.ty.clone();
        let mut is_option = false;

        if attrs.skip {
            Ok(None)
        } else {
            if attrs.strip_option {
                ty = SetterField::strip_option(&field_ident, &field.ty)?;
            }
            if attrs.into {
                let mut into_ty = ty;
                ty = parse2(quote! {C})?;
                let generic_ty = ty.clone();

                let ty_is_option = SetterField::strip_option(&field_ident, &into_ty);
                if ty_is_option.is_ok() {
                    into_ty = ty_is_option.unwrap();
                    ty = parse2(quote! {Option<C>})?;
                    is_option = true;
                }

                generics = parse2(quote! {<#generic_ty: Into<#into_ty>>})?;
            }
            if attrs.name.is_some() {
                setter_ident = attrs.name.unwrap();
            }

            Ok(Some(Self {
                field_ident,
                ty,
                generics,
                setter_ident,
                into: attrs.into,
                strip_option: attrs.strip_option,
                is_option,
                struct_ident,
                struct_generics,
            }))
        }
    }

    fn find_attr(attr: &&Attribute) -> bool {
        attr.path().is_ident("setter")
    }

    pub fn strip_option(field_ident: &'a Ident, ty: &'a Type) -> Result<Type, Error> {
        match ty {
            Type::Path(path) => {
                let opt_path = path.path.segments.iter().last().unwrap();
                match &*opt_path.ident.to_string() {
                    "Option" => match &opt_path.arguments {
                        PathArguments::AngleBracketed(args) => {
                            let ty = &args.args;
                            Ok(parse2(quote! {#ty})?)
                        }
                        _ => unreachable!(),
                    },
                    _ => Err(Error::new(
                        opt_path.ident.span(),
                        format!(
                            "`{}` must be type `Option` to use `strip_option`, found `{}`",
                            field_ident, opt_path.ident
                        ),
                    )),
                }
            }
            _ => Err(Error::new(
                field_ident.span(),
                format!(
                    "`{}` must be type `Option` to use `strip_option`",
                    field_ident
                ),
            )),
        }
    }
}

impl ToTokens for SetterField<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let field_ident = &self.field_ident;
        let ty = &self.ty;
        let generics = &self.generics;

        let setter_ident = &self.setter_ident;
        let struct_ident = &self.struct_ident;
        let struct_generics = &self.struct_generics;
        let mut ident = quote! {#field_ident};
        if self.into {
            if !self.strip_option && self.is_option {
                ident = quote! {#ident.map(|x| x.into())};
            } else {
                ident = quote! {#ident.into()};
            }
        }
        if self.strip_option {
            ident = quote! {Some(#ident)};
        }

        let out_tokens = quote! {

                pub fn #setter_ident #generics(mut self, #field_ident: #ty) -> #struct_ident #struct_generics {
                    self.#field_ident = #ident;
                    self
                }
        };

        out_tokens.to_tokens(tokens);
    }
}

#[derive(Default)]
struct SetterAttr {
    skip: bool,
    strip_option: bool,
    into: bool,
    name: Option<Ident>,
}

impl SetterAttr {
    fn parse_attr<'a>(attr: Option<&'a Attribute>) -> Result<Self, Error> {
        let mut this = Self::default();
        match attr {
            None => Ok(this),
            Some(attr) => {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("skip") {
                        this.skip = true;
                    }
                    if meta.path.is_ident("strip_option") {
                        this.strip_option = true;
                    }
                    if meta.path.is_ident("into") {
                        this.into = true;
                    }
                    if meta.path.is_ident("name") {
                        this.name = Some(meta.value()?.parse()?);
                    }
                    Ok(())
                })?;
                Ok(this)
            }
        }
    }
}
