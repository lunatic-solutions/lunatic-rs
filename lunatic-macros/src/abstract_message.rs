use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{
    AngleBracketedGenericArguments, Data, DeriveInput, Fields, GenericArgument, Ident,
    PathArguments, Result, Type, Variant,
};

pub struct DeriveAbstractMessage {
    ident: Ident,
    messages: Vec<Message>,
}

#[derive(Debug)]
struct Message {
    name: Ident,
    params: Params,
    reply_param: Option<(usize, Type)>,
}

#[derive(Debug)]
enum Params {
    Named(Vec<(Ident, Type)>),
    Unnamed(Vec<(Ident, Type)>),
    Unit,
}

impl Params {
    fn items(&self) -> &[(Ident, Type)] {
        match self {
            Params::Named(named) => named,
            Params::Unnamed(unnamed) => unnamed,
            Params::Unit => &[],
        }
    }

    fn expand_params(&self, ident: &Ident, variant: &Ident) -> TokenStream {
        let params = self.items().iter().map(|(ident, _)| ident);

        match self {
            Params::Named(_) => {
                quote! {
                    #ident::#variant { #( #params ),* }
                }
            }
            Params::Unnamed(_) => {
                quote! {
                    #ident::#variant( #( #params ),* )
                }
            }
            Params::Unit => quote! {
                #ident::#variant
            },
        }
    }
}

impl From<Variant> for Message {
    fn from(variant: Variant) -> Self {
        let name = variant.ident;
        let params = match variant.fields {
            Fields::Named(named) => Params::Named(
                named
                    .named
                    .into_iter()
                    .map(|field| (field.ident.unwrap(), field.ty))
                    .collect(),
            ),
            Fields::Unnamed(unnamed) => Params::Unnamed(
                unnamed
                    .unnamed
                    .into_iter()
                    .enumerate()
                    .map(|(i, field)| (format_ident!("arg{i}"), field.ty))
                    .collect(),
            ),
            Fields::Unit => {
                return Message {
                    name,
                    params: Params::Unit,
                    reply_param: None,
                }
            }
        };

        let reply_param = params
            .items()
            .iter()
            .enumerate()
            .find_map(|(i, (_, ty))| get_request_ty(ty).map(|ty| (i, ty.clone())));

        Message {
            name,
            params,
            reply_param,
        }
    }
}

impl Parse for DeriveAbstractMessage {
    fn parse(input: ParseStream) -> Result<Self> {
        let input: DeriveInput = input.parse()?;
        let ident = input.ident;
        let messages = match input.data {
            Data::Enum(data) => data.variants.into_iter().map(From::from).collect(),
            _ => unimplemented!("only enums are supported"),
        };

        Ok(DeriveAbstractMessage { ident, messages })
    }
}

impl ToTokens for DeriveAbstractMessage {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { ident, messages } = self;

        let methods = messages.iter().map(|message| {
            let method_ident = format_ident!("{}", message.name.to_string().to_case(Case::Snake));
            let params = message.params.items().iter().enumerate().filter_map(|(i, (param_ident, param_ty))| {
                let is_reply_param = message.reply_param.as_ref().map(|(reply_param_index, _)| reply_param_index == &i).unwrap_or(false);
                if is_reply_param {
                    return None;
                }

                Some(quote! { #param_ident: #param_ty })
            });

            let message_expanded = message.params.expand_params(ident, &message.name);

            match &message.reply_param {
                None => {
                    quote! {
                        pub fn #method_ident(process: ::lunatic::Process<#ident> #( , #params )*) {
                            process.send(#message_expanded)
                        }
                    }
                },
                Some((i, ty)) => {
                    let request_ident = &message.params.items().get(*i).unwrap().0;
                    quote! {
                        pub fn #method_ident(process: ::lunatic::Process<#ident> #( , #params )*) -> #ty {
                            let #request_ident = ::lunatic::Request::new();
                            process.send(#message_expanded);
                            #request_ident.wait()
                        }
                    }
                },
            }
        });

        tokens.extend(quote! {
            impl #ident {
                #( #methods )*
            }
        });
    }
}

fn get_request_ty(ty: &Type) -> Option<&Type> {
    match ty {
        Type::Path(path) => {
            let path_str = path.path.to_token_stream().to_string().replace(' ', "");
            let path_str = path_str
                .split_once('<')
                .map(|(s, _)| s)
                .unwrap_or(&path_str);
            let is_request = matches!(path_str, "Request" | "lunatic::Request");
            if !is_request {
                return None;
            }

            let last_segment = path.path.segments.last()?;
            let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &last_segment.arguments else {
                return None;
            };
            if args.len() > 1 {
                return None;
            }
            let GenericArgument::Type(arg) = args.first()? else {
                return None;
            };

            Some(arg)
        }
        _ => None,
    }
}
