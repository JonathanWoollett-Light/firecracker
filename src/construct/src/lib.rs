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

//! Defines the [`Inline`] trait and associated implementations.
//!
//! This trait defines a function which produces a [`proc_macro2::TokenStream`] of code which would
//! allocate `self`.

pub use construct_macros::Inline;
pub use proc_macro2::TokenStream;
pub use quote::quote;

/// Trait defining `inline` a function which produces a [`proc_macro2::TokenStream`] of code which
/// would allocate `self`.
pub trait Inline {
    /// Produces a [`proc_macro2::TokenStream`] of code which would allocate `self`.
    fn inline(&self) -> TokenStream;
}

// Primitive implementations
// -----------------------------------------------------------------------------
/// Convenience macro for defining `construct::Inline` implementations on primitive types.
macro_rules! inline_primitive {
    ($x:ty) => {
        impl Inline for $x {
            #[inline]
            fn inline(&self) -> TokenStream {
                quote! { #self }
            }
        }
    };
}

impl<T: Inline, const N: usize> Inline for [T; N] {
    #[allow(clippy::shadow_reuse)]
    #[inline]
    fn inline(&self) -> TokenStream {
        let values = self.iter().map(Inline::inline);
        quote! {
            [ #(#values,)* ]
        }
    }
}
inline_primitive!(bool);
inline_primitive!(char);
inline_primitive!(f32);
inline_primitive!(f64);
inline_primitive!(i8);
inline_primitive!(i16);
inline_primitive!(i32);
inline_primitive!(i64);
inline_primitive!(i128);
inline_primitive!(isize);
inline_primitive!(str);
inline_primitive!(u8);
inline_primitive!(u16);
inline_primitive!(u32);
inline_primitive!(u64);
inline_primitive!(u128);
impl Inline for () {
    #[inline]
    fn inline(&self) -> TokenStream {
        quote! {
            ()
        }
    }
}
inline_primitive!(usize);

// Non-zero implementations
// -----------------------------------------------------------------------------
/// Convenience macro for defining `construct::Inline` implementations on `NonZero` types.
macro_rules! inline_nonzero {
    ($x:ty) => {
        impl Inline for $x {
            #[inline]
            fn inline(&self) -> TokenStream {
                let y = self.get();
                quote! { unsafe { $x::new_unchecked(#y) } }
            }
        }
    };
}
inline_nonzero!(std::num::NonZeroI8);
inline_nonzero!(std::num::NonZeroI16);
inline_nonzero!(std::num::NonZeroI32);
inline_nonzero!(std::num::NonZeroI64);
inline_nonzero!(std::num::NonZeroI128);
inline_nonzero!(std::num::NonZeroIsize);
inline_nonzero!(std::num::NonZeroU8);
inline_nonzero!(std::num::NonZeroU16);
inline_nonzero!(std::num::NonZeroU32);
inline_nonzero!(std::num::NonZeroU64);
inline_nonzero!(std::num::NonZeroU128);
inline_nonzero!(std::num::NonZeroUsize);

// Collections implementations
// -----------------------------------------------------------------------------
impl<T: Inline> Inline for Vec<T> {
    #[allow(clippy::shadow_reuse)]
    #[inline]
    fn inline(&self) -> TokenStream {
        let fields = self.iter().map(Inline::inline);
        quote! {
            vec![#(#fields,)*]
        }
    }
}
impl<K: Inline, V: Inline> Inline for std::collections::BTreeMap<K, V> {
    #[allow(clippy::shadow_reuse)]
    #[inline]
    fn inline(&self) -> TokenStream {
        let (keys, values) = self
            .iter()
            .map(|(k, v)| (k.inline(), v.inline()))
            .unzip::<_, _, Vec<_>, Vec<_>>();

        quote! {
            {
                let mut map = std::collections::BTreeMap::new();
                #(
                    map.insert(#keys,#values);
                )*
                map
            }
        }
    }
}

// Misc implementations
// -----------------------------------------------------------------------------
impl<T> Inline for std::marker::PhantomData<T> {
    #[inline]
    fn inline(&self) -> TokenStream {
        quote! {
            std::marker::PhantomData
        }
    }
}
impl Inline for String {
    #[inline]
    fn inline(&self) -> TokenStream {
        quote! {
            String::from(#self)
        }
    }
}
impl<T: Inline> Inline for Option<T> {
    #[inline]
    fn inline(&self) -> TokenStream {
        match self {
            None => quote! { None },
            Some(x) => {
                let a = x.inline();
                quote! { Some(#a) }
            }
        }
    }
}
impl<T: Inline, E: Inline> Inline for Result<T, E> {
    #[inline]
    fn inline(&self) -> TokenStream {
        match self {
            Ok(ok) => {
                let a = ok.inline();
                quote! { Ok(#a) }
            }
            Err(err) => {
                let a = err.inline();
                quote! { Err(#a) }
            }
        }
    }
}

// We create implementations up to tuples of 128 elements.
construct_macros::derive_tuple!(128);

// Tests
// -----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn primitive_test() {
        let a = 2u8;
        assert_eq!(a.inline().to_string(), quote! { 2u8 }.to_string())
    }
    #[test]
    fn vec_test() {
        let a: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7];
        assert_eq!(
            a.inline().to_string(),
            quote! { vec![1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8,] }.to_string()
        )
    }
    #[test]
    fn str_test() {
        let a = "1 2 3 a b c 4 d";
        assert_eq!(format!("\"{}\"", a), a.inline().to_string());
    }
    #[test]
    fn string_test() {
        let a = String::from("1 2 3 a b c 4 d");
        assert_eq!(
            a.inline().to_string(),
            quote! { String::from("1 2 3 a b c 4 d") }.to_string()
        )
    }
}
