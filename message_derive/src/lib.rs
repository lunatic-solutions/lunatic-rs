use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields, Ident, Index};

/// Helper for implementing a `lunatic::Message` trait on structs containing resources.
///
/// If the custom type doesn't contain resources it's recommended to implement the
/// `serde::Serialize` **and** `serde::Deserialize` traits instead.
///
/// ### Example
/// ```
/// #[derive(Message)]
/// struct ProcWrapper {
///     proc: lunatic::process::Process,
/// }
/// ```
#[proc_macro_derive(Message)]
pub fn message_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // Get the name of the type we want to implement the trait for
    let name = &input.ident;
    // Get the generics for the type
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let from = from_bincode(&input.data);
    let to = to_bincode(&input.data);

    let expanded = quote! {
      impl #impl_generics lunatic::Message for #name #ty_generics #where_clause {
        fn from_bincode(data: &[u8], res: &[u64]) -> (usize, Self) {
            #from
        }

        unsafe fn to_bincode(self, dest: &mut Vec<u8>) {
            #to
        }
      }
    };

    proc_macro::TokenStream::from(expanded)
}

fn to_bincode(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    // Expands to an expression like
                    //
                    //     Message::to_bincode(self.x, dest);
                    //     Message::to_bincode(self.y, dest);
                    //     Message::to_bincode(self.z, dest);
                    //
                    // but using fully qualified function call syntax.
                    //
                    // We take some care to use the span of each `syn::Field` as
                    // the span of the corresponding `to_bincode` call. This way
                    // if one of the field types does not implement `Message` then
                    // the compiler's error message underlines which field it is.
                    let recurse = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        quote_spanned! {
                            f.span() => lunatic::Message::to_bincode(self.#name, dest);
                        }
                    });
                    quote! { #(#recurse)* }
                }
                Fields::Unnamed(ref fields) => {
                    // Expands to an expression like
                    //
                    //     Message::to_bincode(self.0, dest);
                    //     Message::to_bincode(self.1, dest);
                    let recurse = fields.unnamed.iter().enumerate().map(|(i, f)| {
                        let index = Index::from(i);
                        quote_spanned! {
                            f.span() => lunatic::Message::to_bincode(self.#index, dest);
                        }
                    });
                    quote! { #(#recurse)* }
                }
                Fields::Unit => {
                    // Unit structs occupy 0 bytes
                    quote!()
                }
            }
        }
        Data::Enum(ref data) => {
            // Expands to an expression like
            //
            //     match self {
            //         Self::V1(f1, f2) => {
            //             Message::to_bincode(0 as u32, dest);
            //             Message::to_bincode(f1, dest);
            //             Message::to_bincode(f2, dest);
            //         },
            //         Self::V2(f1) => {
            //             Message::to_bincode(1 as u32, dest);
            //             Message::to_bincode(f1, dest);
            //         }
            //     }
            let variants = data.variants.iter().enumerate().map(|(index, variant)| {
                let recurse = variant.fields.iter().enumerate().map(|(i, f)| {
                    let name = Ident::new(&format!("f{}", i), f.span());
                    quote_spanned! { f.span() =>
                        lunatic::Message::to_bincode(#name, dest);
                    }
                });
                let fields = variant.fields.iter().enumerate().map(|(i, f)| {
                    let name = Ident::new(&format!("f{}", i), f.span());
                    quote! { #name }
                });
                let ident = &variant.ident;
                if variant.fields.is_empty() {
                    quote_spanned! { variant.span() =>
                        Self::#ident => {
                            lunatic::Message::to_bincode(#index as u32, dest);
                        }
                    }
                } else {
                    quote_spanned! { variant.span() =>
                        Self::#ident( #(#fields,)* ) => {
                            lunatic::Message::to_bincode(#index as u32, dest);
                            #(#recurse)*
                        }
                    }
                }
            });
            quote! {
                match self {
                    #(#variants,)*
                }

            }
        }
        Data::Union(_) => unimplemented!(),
    }
}

fn from_bincode(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    // Expands to an expression like
                    //
                    //     let mut cursor: usize = 0;
                    //     let (bytes_read, x) = Message::from_bincode(&data[cursor..], res);
                    //     cursor = cursor + bytes_read;
                    //     let (bytes_read, y) = Message::from_bincode(&data[cursor..], res);
                    //     cursor = cursor + bytes_read;
                    //
                    //     (cursor, Self { x, y })
                    //
                    // but using fully qualified function call syntax.
                    //
                    // We take some care to use the span of each `syn::Field` as
                    // the span of the corresponding `to_bincode`
                    // call. This way if one of the field types does not
                    // implement `Message` then the compiler's error message
                    // underlines which field it is.
                    let recurse = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        quote_spanned! { f.span() =>
                            let (bytes_read, #name) = lunatic::Message::from_bincode(&data[cursor..], res);
                            cursor = cursor + bytes_read;
                        }
                    });
                    let fields = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        quote! { #name }
                    });
                    quote! {
                        let mut cursor: usize = 0;
                        #(#recurse)*
                        (cursor, Self { #(#fields,)*})
                    }
                }
                Fields::Unnamed(ref fields) => {
                    // Expands to an expression like
                    //
                    //     let mut cursor: usize = 0;
                    //     let (bytes_read, v1) = Message::from_bincode(&data[cursor..], res);
                    //     cursor = cursor + bytes_read;
                    //     let (bytes_read, v2) = Message::from_bincode(&data[cursor..], res);
                    //     cursor = cursor + bytes_read;
                    //
                    //     (cursor, Self(v1,v2))
                    let recurse =fields.unnamed.iter().enumerate().map(|(i, f)| {
                        let name = Ident::new(&format!("v_{}", i), f.span());
                        quote_spanned! { f.span() =>
                            let (bytes_read, #name) = lunatic::Message::from_bincode(&data[cursor..], res);
                            cursor = cursor + bytes_read;
                        }
                    });
                    let fields = fields.unnamed.iter().enumerate().map(|(i, f)| {
                        let name = Ident::new(&format!("v_{}", i), f.span());
                        quote! { #name }
                    });
                    quote! {
                        let mut cursor: usize = 0;
                        #(#recurse)*
                        (cursor, Self ( #(#fields,)* ))
                    }
                }
                Fields::Unit => {
                    // Unit structs occupy 0 bytes
                    quote!((0, Self))
                }
            }
        }
        Data::Enum(ref data) => {
            // Expands to an expression like
            //
            //     let mut cursor: usize = 0;
            //     let (bytes_read, variant): (_, u32) = = Message::from_bincode(&data[cursor..], res);
            //     cursor = cursor + bytes_read;
            //     let variant = match variant {
            //         0 => {
            //             let (bytes_read, f1) = Message::from_bincode(&data[cursor..], res);
            //             cursor = cursor + bytes_read;
            //             let (bytes_read, f2) = Message::from_bincode(&data[cursor..], res);
            //             cursor = cursor + bytes_read;
            //             ...
            //             Self::V1(f1, f2)
            //         },
            //         ...
            //         _ => panic!("Enum deserialization failed")
            //     }
            //
            //     (cursor, variant)
            let variants = data.variants.iter().enumerate().map(|(index, variant)| {
                let recurse = variant.fields.iter().enumerate().map(|(i, f)| {
                    let name = Ident::new(&format!("f{}", i), f.span());
                    quote_spanned! { f.span() =>
                        let (bytes_read, #name) = lunatic::Message::from_bincode(&data[cursor..], res);
                        cursor = cursor + bytes_read;
                    }
                });
                let fields = variant.fields.iter().enumerate().map(|(i, f)| {
                    let name = Ident::new(&format!("f{}", i), f.span());
                    quote! { #name }
                });
                let ident = &variant.ident;
                if variant.fields.is_empty() {
                    quote_spanned! { variant.span() =>
                        #index => Self::#ident
                    }
                }
                else {
                    quote_spanned! { variant.span() =>
                        #index => {
                            #(#recurse)*
                            Self::#ident(#(#fields,)*)
                        }
                    }
                }
            });
            quote! {
                let mut cursor: usize = 0;
                let (bytes_read, variant): (usize, u32) = lunatic::Message::from_bincode(&data[cursor..], res);
                cursor = cursor + bytes_read;
                let variant = match variant as usize {
                    #(#variants,)*
                    _ => panic!("Enum deserialization failed")
                };
                (cursor, variant)
            }
        }
        Data::Union(_) => unimplemented!(),
    }
}
