// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#![warn(clippy::pedantic, clippy::restriction)]
#![allow(
    clippy::blanket_clippy_restriction_lints,
    clippy::implicit_return,
    clippy::pattern_type_mismatch,
    clippy::std_instead_of_alloc,
    clippy::std_instead_of_core,
    clippy::pub_use,
    clippy::non_ascii_literal,
    clippy::single_char_lifetime_names,
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::unseparated_literal_suffix,
    clippy::mod_module_files
)]

//! Procedural macros for the `construct` crate.

use std::convert::TryFrom;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_macro_input, DeriveInput};

/// Formats a given `usize` into an alphabetical string.
#[allow(clippy::arithmetic_side_effects, clippy::integer_arithmetic)]
fn format_radix(mut x: usize) -> String {
    let mut result = vec![];
    loop {
        let m = x % 25;
        x /= 25;
        // SAFETY: `0 <= m < 25` therefore this is always safe.
        result.push(char::from(unsafe {
            u8::try_from(m + 97).unwrap_unchecked()
        }));
        if x == 0 {
            break;
        }
    }
    result.into_iter().rev().collect()
}

/// Generates tuple implementations for large tuples.
///
/// # Panics
///
/// When the 1st token is not a literal which can be parsed to a `usize`.
#[proc_macro]
pub fn derive_tuple(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    /// Error message when 1st token cannot be parsed to a `usize`.
    const FIRST_TOKEN_EXPECT: &str = "Expected single literal token of a `usize`.";
    // Attempts to parsd 1st token as a `usize`.
    let num_res = match stream.into_iter().next() {
        Some(proc_macro::TokenTree::Literal(lit)) => {
            lit.to_string().parse::<usize>().map_err(|err| {
                let err_str = format!("{}: {}", FIRST_TOKEN_EXPECT, err);
                quote_spanned! {
                    Span::call_site() =>
                    compile_error!(#err_str)
                }
            })
        }
        _ => Err(quote_spanned! {
            Span::call_site() =>
            compile_error!(#FIRST_TOKEN_EXPECT)
        }),
    };
    let num = match num_res {
        Ok(ok) => ok,
        Err(err) => return proc_macro::TokenStream::from(err),
    };

    // Defines `construct::inline` implementations for tuples up to `num` elements.
    #[allow(clippy::shadow_reuse)]
    let collected = (1..num)
        .map(|i| {
            let (values, generics): (Vec<_>, Vec<_>) = (0..i)
                .map(|j| {
                    (
                        format_ident!("__{}", format_radix(j)),
                        format_ident!("__{}", format_radix(j).to_uppercase()),
                    )
                })
                .unzip();
            let index = (0..i).map(syn::Index::from);

            quote! {
                impl<#(#generics:Inline,)*> Inline for (#(#generics,)*) {
                    fn inline(&self) -> TokenStream {
                        let (#(#values,)*) = (#(self.#index.inline(),)*);
                        quote! {
                            (#(##values,)*)
                        }
                    }
                }
            }
        })
        .collect::<proc_macro2::TokenStream>();

    proc_macro::TokenStream::from(collected)
}

/// Derives the [`construct::Inline`] trait.
///
/// # Panics
///
/// When applied to:
/// - A union.
/// - An enum with more than 25 variants.
/// - An enum with named variants.
#[proc_macro_derive(Inline)]
pub fn derive_inline(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let ast = parse_macro_input!(input as DeriveInput);

    let ident = ast.ident;
    let generics = ast.generics.params;
    let generic_idents = generics
        .iter()
        .map(|g| match g {
            syn::GenericParam::Type(x) => x.ident.clone(),
            syn::GenericParam::Lifetime(x) => x.lifetime.ident.clone(),
            syn::GenericParam::Const(x) => x.ident.clone(),
        })
        .collect::<Vec<_>>();

    let expanded = match ast.data {
        syn::Data::Struct(data_struct) => {
            parse_struct(&ident, &generics, &generic_idents, data_struct)
        }
        syn::Data::Enum(data_enum) => parse_enum(&ident, &generics, &generic_idents, data_enum),
        syn::Data::Union(_) => quote_spanned! {
            Span::call_site() =>
            compile_error!("Unions are not currently supported.")
        },
    };

    proc_macro::TokenStream::from(expanded)
}

/// Derives `Inline` for struct.
#[allow(clippy::shadow_reuse)]
fn parse_struct(
    ident: &syn::Ident,
    generics: &syn::punctuated::Punctuated<syn::GenericParam, syn::token::Comma>,
    generic_idents: &[syn::Ident],
    data_struct: syn::DataStruct,
) -> TokenStream {
    let inner = match data_struct.fields {
        syn::Fields::Named(named_fields) => {
            let fields_res = named_fields
                .named
                .iter()
                .map(|field| match field.ident.as_ref() {
                    Some(field_ident) => Ok((
                        field_ident.clone(),
                        format_ident!("{}_inlined", field_ident),
                    )),
                    None => Err(quote_spanned! {
                        Span::call_site() =>
                        compile_error!("Named struct fields should have names")
                    }),
                })
                .collect::<Result<Vec<_>, TokenStream>>();
            let fields = match fields_res {
                Ok(ok) => ok,
                Err(err) => return err,
            };
            let (names, fields_inlined) = fields.into_iter().unzip::<_, _, Vec<_>, Vec<_>>();
            quote! {
                let (#(#fields_inlined,)*) = (#(self.#names.inline(),)*);
                construct::quote! {
                    #ident {
                        #(
                            #names: ##fields_inlined,
                        )*
                    }
                }
            }
        }
        syn::Fields::Unnamed(fields) => {
            let (i, values): (Vec<_>, Vec<_>) = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| (syn::Index::from(i), format_ident!("__{}", format_radix(i))))
                .unzip();
            quote! {
                let (#(#values,)*) = (#(self.#i.inline(),)*);
                construct::quote! {
                    #ident (
                        #(##values,)*
                    )
                }
            }
        }
        syn::Fields::Unit => quote! { construct::quote! { #ident } },
    };
    quote! {
        impl<#generics> construct::Inline for #ident<#(#generic_idents,)*> {
            fn inline(&self) -> construct::TokenStream {
                #inner
            }
        }
    }
}

/// Derives `Inline` for enum.
#[allow(clippy::shadow_reuse)]
fn parse_enum(
    ident: &syn::Ident,
    generics: &syn::punctuated::Punctuated<syn::GenericParam, syn::token::Comma>,
    generic_idents: &[syn::Ident],
    data_enum: syn::DataEnum,
) -> TokenStream {
    let matching_res = data_enum.variants.into_iter().map(|variant| {
        let variant_ident = variant.ident.clone();
        match &variant.fields {
            syn::Fields::Unnamed(unnamed_fields) => {
                let fields_res = unnamed_fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, _)| match i {
                        0..=24 => {
                            // SAFETY: `0 <= i <= 24` therefore this is always safe.
                            let x = char::from(unsafe { u8::try_from(i.checked_add(97).unwrap_unchecked()).unwrap_unchecked() });
                            Ok(format_ident!("{}", x))
                        },
                        _ => Err(quote_spanned! {
                            Span::call_site() =>
                            compile_error!("Only up to 25 fields in an enum variant are supported.")
                        }),
                    })
                    .collect::<Result<Vec<proc_macro2::Ident>,TokenStream>>();
                let fields = fields_res?;

                let fields_inlined = fields
                    .iter()
                    .map(|f| format_ident!("{}_inlined", f))
                    .collect::<Vec<_>>();
                Ok(quote! {
                    #ident::#variant_ident(#(#fields,)*) => {
                        let (#(#fields_inlined,)*) = (#(#fields.inline(),)*);
                        construct::quote! { #ident::#variant_ident(#(##fields_inlined,)*) }
                    }
                })
            }
            syn::Fields::Unit => Ok(quote! {
                    #ident::#variant_ident => construct::quote! { #ident::#variant_ident }
            }),
            syn::Fields::Named(_) => Err(quote_spanned! {
                Span::call_site() =>
                compile_error!("Named fields on enum variants are not supported.")
            })
        }
    }).collect::<Result<Vec<_>,TokenStream>>();
    let matching = match matching_res {
        Ok(ok) => ok,
        Err(err) => return err,
    };
    quote! {
        impl<#generics> construct::Inline for #ident<#(#generic_idents,)*> {
            fn inline(&self) -> construct::TokenStream {
                match self {
                    #(
                        #matching,
                    )*
                }
            }
        }
    }
}
