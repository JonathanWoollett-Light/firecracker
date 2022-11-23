// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::utils::{DataTypeToken, MultiLineString};
use crate::{Field, Flag, Member};

/// Builder for bit fields.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct BitFieldBuilder {
    /// String used to define `From<HashSet<String>>`.
    flag_matching_from_hashset: TokenStream,
    /// String used to define `From<HashSet<String>>`.
    flag_setting_hashset: TokenStream,
    /// String used to define `From<HashMap<String,$data_type>>`.
    field_matching_from_hashmap: TokenStream,
    /// String used to define `From<HashMap<String,$data_type>>`.
    field_setting_hashmap: TokenStream,
    /// String used to define the table used in the rustdoc for the bit field.
    struct_doc_table_layout: Vec<TokenStream>,
    /// Accessor methods to members.
    struct_accessors: TokenStream,
    /// String used to form the display the bit field, the lines represent:
    /// 1. Top border
    /// 2. Bit numbers
    /// 3. Border
    /// 4. Field idents
    /// 5. Border
    /// 6. Field values
    /// 7. Bottom border
    /// Fmt values (since write doesn't work with in place ones)
    display_string: MultiLineString,
    /// String used to pass arguments for `std::fmt::Display` implementation.
    display_fmt_string: TokenStream,
    /// Struct data type (e.g. `u8`)
    data_type: DataTypeToken,
    /// Struct identifier
    struct_name: Ident,
    /// Bit flag constants to construct a field with a single flag active.
    flag_constants: TokenStream,
    /// Bit mask including all bit flags.
    bit_flag_mask: TokenStream,
    /// Bit mask including all bit ranges.
    bit_range_mask: TokenStream,
}

impl BitFieldBuilder {
    /// Constructs new `BitFieldBuilder`.
    pub fn new(struct_name: Ident, data_type: DataTypeToken) -> Self {
        Self {
            flag_matching_from_hashset: TokenStream::new(),
            flag_setting_hashset: TokenStream::new(),
            field_matching_from_hashmap: TokenStream::new(),
            field_setting_hashmap: TokenStream::new(),
            struct_doc_table_layout: vec![quote! {
                #[doc = "<tr><th>Bit/s</th><th>Identifier</th><th>Description</th></tr>"]
            }],
            struct_accessors: TokenStream::new(),
            #[rustfmt::skip]
            display_string: MultiLineString::from("\
                ┌───────\n\
                │ \x1b[1mBit/s\x1b[0m \n\
                ├───────\n\
                │ \x1b[1mDesc\x1b[0m  \n\
                ├───────\n\
                │ \x1b[1mValue\x1b[0m \n\
                └───────",
            ),
            display_fmt_string: TokenStream::new(),
            data_type,
            struct_name,
            flag_constants: TokenStream::new(),
            bit_flag_mask: quote! { 0 },
            bit_range_mask: quote! { 0 },
        }
    }

    /// Adds a bit member to the structure.
    pub(crate) fn add(self, member: Member) -> Self {
        match member {
            Member::Field(field) => self.add_bit_range(field),
            Member::Flag(flag) => self.add_bit_flag(flag),
        }
    }

    /// Adds a bit range to the structure.
    pub(crate) fn add_bit_range(
        mut self,
        Field {
            start,
            rustdoc,
            identifier,
            stop,
        }: Field,
    ) -> Self {
        let identifier_str = identifier.to_string();
        let data_type = &self.data_type;

        // Display
        // ------------------------
        // Use first 10 characters of identifier_str.
        let cropped = identifier_str.chars().take(10).collect::<String>();
        #[rustfmt::skip]
        self.display_string.push_str(&format!("\
            ┬─────────────\n\
            │\x20     {:02}..{:02} \n\
            ┼─────────────\n\
            │\x20 {:>10} \n\
            ┼─────────────\n\
            │\x20{{:>11}} \n\
            ┴─────────────\
            ",
            start,
            stop,
            cropped,
        ));
        self.display_fmt_string.extend(quote! {
            self.#identifier().to_string(),
        });

        // Struct member
        // ------------------------
        let ident_mut = quote::format_ident!("{}_mut", identifier);
        self.struct_accessors.extend(quote! {
            #[doc=#rustdoc]
            pub fn #identifier(&self) -> bit_fields::BitRange<#data_type,#start,#stop> {
                bit_fields::BitRange(&self.0)
            }
            #[doc=#rustdoc]
            pub fn #ident_mut(&mut self) -> bit_fields::BitRangeMut<#data_type,#start,#stop> {
                bit_fields::BitRangeMut(&mut self.0)
            }
        });

        let base = data_type.base();

        // Bit range mask
        // ------------------------
        self.bit_range_mask
            .extend(quote! { | bit_fields::BitRange::<#base,#start,#stop>::MASK });

        // try_from_field_map
        // ------------------------
        {
            self.field_matching_from_hashmap.extend(quote! {
                #identifier_str => {
                    match bit_fields::BitRangeMut::<#base,#start,#stop>(&mut acc).checked_assign(#base::from(v)) {
                        Ok(_) => Ok(acc),
                        Err(err) => Err(bit_fields::TryFromFieldMapError::CheckedAssign(err)),
                    }
                },
            });
            self.field_setting_hashmap.extend(quote! {
                map.insert(T::from(#identifier_str),#data_type::from(&bit_field.#identifier()));
            });
        }

        // Struct rustdoc table
        // ------------------------
        let rustdoc_string = format!(
            "<tr><td>{:02}..{:02}</td><td>{}</td><td>{}</td></tr>",
            start,
            // Due to the earlier check on `stop <= start` we can guarantee
            // `stop > start >= 0`, thus `stop >= 1` thus `stop - 1 >=0` thus this
            // will never panic.
            stop,
            identifier_str,
            rustdoc
        );
        self.struct_doc_table_layout.push(quote! {
            #[doc=#rustdoc_string]
        });

        self
    }
    /// Adds a bit flag to the structure.
    #[allow(clippy::too_many_lines)]
    pub(crate) fn add_bit_flag(
        mut self,
        Flag {
            index,
            rustdoc,
            identifier,
        }: Flag,
    ) -> Self {
        let identifier_str = identifier.to_string();
        let data_type = &self.data_type;

        // Display
        // ------------------------
        // Use first 4 characters of the identifier_str.
        let cropped = identifier_str.chars().take(4).collect::<String>();
        #[rustfmt::skip]
        self.display_string.push_str(&format!("\
            ┬───────\n\
            │\x20   {:02} \n\
            ┼───────\n\
            │\x20{:>5} \n\
            ┼───────\n\
            │\x20{{:>5}} \n\
            ┴───────\
            ",
            index,cropped
        ));
        self.display_fmt_string.extend(quote! {
            self.#identifier().to_string(),
        });
        // Struct member
        // ------------------------
        let ident_mut = quote::format_ident!("{}_mut", identifier);
        self.struct_accessors.extend(quote! {
            #[doc=#rustdoc]
            pub fn #identifier(&self) -> bit_fields::Bit<#data_type,#index> {
                bit_fields::Bit(&self.0)
            }
            #[doc=#rustdoc]
            pub fn #ident_mut(&mut self) ->  bit_fields::BitMut<#data_type,#index> {
                bit_fields::BitMut(&mut self.0)
            }
        });

        // Flag constant
        // ------------------------
        use convert_case::{Case, Casing};
        let identifier_str_pascal = identifier_str.to_case(Case::Pascal);
        let ident_pascal = quote::format_ident!("{}", identifier_str_pascal);
        if data_type.is_nonzero() {
            self.flag_constants.extend(quote! {
                #[doc=#rustdoc]
                // SAFETY: Always safe.
                pub const #ident_pascal: Self = Self(unsafe { <#data_type>::new_unchecked(1 <<  #index) });
            });
        } else {
            self.flag_constants.extend(quote! {
                #[doc=#rustdoc]
                pub const #ident_pascal: Self = Self(1 << (#index as #data_type));
            });
        }

        // Struct rustdoc table
        // ------------------------
        let rustdoc_string = format!(
            "<tr><td>{:02}</td><td>{}</td><td>{}</td></tr>",
            index, identifier_str, rustdoc
        );
        self.struct_doc_table_layout.push(quote! {
            #[doc=#rustdoc_string]
        });

        let base = data_type.base();

        // Bit flag mask
        // ------------------------
        self.bit_flag_mask
            .extend(quote! { | bit_fields::Bit::<#base,#index>::MASK });

        // try_from_flag_set
        // ------------------------
        {
            self.flag_matching_from_hashset.extend(quote! {
                #identifier_str => Ok(acc | bit_fields::Bit::<#base,#index>::MASK),
            });
            self.flag_setting_hashset.extend(quote! {
                if bit_field.#identifier().is_on() {
                    set.insert(T::from(#identifier_str));
                }
            });
        }

        self
    }

    /// Ends the bit field, completing the display string.
    fn end(&mut self) {
        #[rustfmt::skip]
            self.display_string.push_str("\
                ┐\n\
                │\n\
                ┤\n\
                │\n\
                ┤\n\
                │\n\
                ┘\
            ");
    }

    /// Composes `self` into `proc_macro::TokenStream`.
    // When `quote!` is used on an iterable it always prouces this warnings.
    #[allow(clippy::too_many_lines, clippy::shadow_reuse)]
    pub fn compose(mut self) -> TokenStream {
        self.end();

        // De-structure self into values we can pass to `quote!`.
        let BitFieldBuilder {
            flag_matching_from_hashset,
            flag_setting_hashset,
            field_matching_from_hashmap,
            field_setting_hashmap,
            struct_doc_table_layout,
            struct_accessors,
            display_string,
            display_fmt_string,
            data_type,
            struct_name,
            flag_constants,
            bit_flag_mask,
            bit_range_mask,
        } = self;

        let serde = if cfg!(feature = "serde") {
            let visitor = quote::format_ident!("{}Visitor", struct_name);
            let error_msg = format!("Failed to deserialize {}", struct_name);
            quote! {
                impl serde::Serialize for #struct_name {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where
                        S: serde::Serializer,
                    {
                        use serde::ser::{Serialize, SerializeMap, SerializeSeq, SerializeTuple, Serializer};
                        let (set, map): (std::collections::HashSet<&'static str>, std::collections::HashMap<&'static str, #data_type>) = self.into();
                        let mut tup = serializer.serialize_tuple(2)?;
                        tup.serialize_element(&set)?;
                        tup.serialize_element(&map)?;
                        tup.end()
                    }
                }
                struct #visitor;
                impl<'de> serde::de::Visitor<'de> for #visitor {
                    type Value = #struct_name;
                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(
                            formatter,
                            "a set of bit flags followed by a map of fields"
                        )
                    }
                    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::SeqAccess<'de>,
                    {
                        use std::convert::TryFrom;
                        if let Some(set) = seq.next_element::<std::collections::HashSet<String>>()? {
                            if let Some(map) = seq.next_element::<std::collections::HashMap<String, #data_type>>()? {
                                Ok(#struct_name::try_from((set, map)).expect(#error_msg))
                            } else {
                                Err(serde::de::Error::custom("no 2nd value in seq"))
                            }
                        } else {
                            Err(serde::de::Error::custom("no 1st value in seq"))
                        }
                    }
                }
                impl<'de> serde::Deserialize<'de> for #struct_name {
                    fn deserialize<D>(deserializer: D) -> Result<#struct_name, D::Error>
                    where
                        D: serde::Deserializer<'de>,
                    {
                        deserializer.deserialize_tuple(2,#visitor)
                    }
                }
            }
        } else {
            TokenStream::new()
        };

        let set_theory = {
            let (superset, subset, disjoint, intersection, union_fn) = if data_type.is_nonzero() {
                (
                    quote! { (self.0.get() & other.0.get() & Self::BIT_FLAG_MASK.0.get()) == (other.0.get() & Self::BIT_FLAG_MASK.0.get()) },
                    quote! { (self.0.get() & other.0.get() & Self::BIT_FLAG_MASK.0.get()) == (self.0.get() & Self::BIT_FLAG_MASK.0.get()) },
                    quote! { (self.0.get() & other.0.get() & Self::BIT_FLAG_MASK.0.get()) == 0 },
                    quote! {
                        /// Returns the [`intersection`](https://en.wikipedia.org/wiki/Intersection_(set_theory)) of `self` and `other`.
                        ///
                        /// Returns the intersection of defined bit flags, all other bits will be 0.
                        ///
                        /// # Safety
                        ///
                        /// 2 bit fields which share no bit flags will produce a bit field with 0
                        /// filds which will result in the underlying `NonZero` data type being 0.
                        pub unsafe fn intersection(&self, other: &Self) -> Self {
                            Self(#data_type::new_unchecked(self.0.get() & other.0.get() & Self::BIT_FLAG_MASK.0.get()))
                        }
                    },
                    quote! {
                        /// Returns the [`union`](https://en.wikipedia.org/wiki/Union_(set_theory)) of `self` and `other`.
                        ///
                        /// Returns the union of defined bit flags, all other bits will be 0.
                        ///
                        /// # Safety
                        ///
                        /// A union of 2 bit fields with 0 bit flags returns a union 0 fields, this
                        /// will result in the underlying `NonZero` data type being 0.
                        pub unsafe fn union(&self, other: &Self) -> Self {
                            Self(#data_type::new_unchecked((self.0.get() | other.0.get()) & Self::BIT_FLAG_MASK.0.get()))
                        }
                    },
                )
            } else {
                (
                    quote! { (self.0 & other.0 & Self::BIT_FLAG_MASK.0) == (other.0 & Self::BIT_FLAG_MASK.0) },
                    quote! { (self.0 & other.0 & Self::BIT_FLAG_MASK.0) == (self.0 & Self::BIT_FLAG_MASK.0) },
                    quote! { (self.0 & other.0 & Self::BIT_FLAG_MASK.0) == 0 },
                    quote! {
                        /// Returns the [`intersection`](https://en.wikipedia.org/wiki/Intersection_(set_theory)) of `self` and `other`.
                        ///
                        /// Returns the intersection of defined bit flags, all other bits will be 0.
                        pub fn intersection(&self, other: &Self) -> Self {
                            Self(self.0 & other.0 & Self::BIT_FLAG_MASK.0)
                        }
                    },
                    quote! {
                        /// Returns the [`union`](https://en.wikipedia.org/wiki/Union_(set_theory)) of `self` and `other`.
                        ///
                        /// Returns the union of defined bit flags, all other bits will be 0.
                        pub fn union(&self, other: &Self) -> Self {
                            Self((self.0 | other.0) & Self::BIT_FLAG_MASK.0)
                        }
                    },
                )
            };

            quote! {
                /// Returns `true` if `self` is a [`superset`](https://en.wikipedia.org/wiki/Subset) of `other`.
                ///
                /// This only considers defined bit flags.
                pub fn superset(&self, other: &Self) -> bool {
                    #superset
                }
                /// Returns `true` if `self` is a [`subset`](https://en.wikipedia.org/wiki/Subset) of `other`.
                ///
                /// This only considers defined bit flags.
                pub fn subset(&self, other: &Self) -> bool {
                    #subset
                }
                /// Returns `true` if `self` and `other` are [`disjoint sets`](https://en.wikipedia.org/wiki/Disjoint_sets).
                ///
                /// This only considers defined bit flags.
                pub fn disjoint(&self, other: &Self) -> bool {
                    #disjoint
                }
                #intersection
                #union_fn
            }
        };

        let bit_index = (0..self.data_type.size())
            .map(|i| {
                quote! {
                    impl bit_fields::BitIndex<#data_type,#i> for #struct_name {
                        fn bit(&self) -> bit_fields::Bit<#data_type,#i> {
                            bit_fields::Bit(&self.0)
                        }
                    }
                    impl bit_fields::BitIndexMut<#data_type,#i> for #struct_name {
                        fn bit_mut(&mut self) -> bit_fields::BitMut<#data_type,#i> {
                            bit_fields::BitMut(&mut self.0)
                        }
                    }
                }
            })
            .collect::<TokenStream>();

        let index_fn = quote! {
            /// Returns a reference to the `N`th bit.
            #[allow(clippy::same_name_method)]
            pub fn bit<const N: u8>(&self) -> bit_fields::Bit<#data_type,N>
            where
                Self: bit_fields::BitIndex<#data_type,N>,
            {
                <Self as bit_fields::BitIndex<#data_type,N>>::bit(self)
            }
            /// Returns a mutable reference to the `N`th bit.
            #[allow(clippy::same_name_method)]
            pub fn bit_mut<const N: u8>(&mut self) -> bit_fields::BitMut<#data_type,N>
            where
                Self: bit_fields::BitIndexMut<#data_type,N>,
            {
                <Self as bit_fields::BitIndexMut<#data_type,N>>::bit_mut(self)
            }
        };

        // - Constructing a bit field from a set of bit flags
        let from_type_parts = |err: TokenStream| {
            if data_type.is_nonzero() {
                (
                    quote! { Option<#err> },
                    quote! {
                        match value {
                            0 => Err(None),
                            _ => Ok(unsafe { Self(#data_type::new_unchecked(0)) })
                        }
                    },
                )
            } else {
                (err, quote! { Ok(Self(value)) })
            }
        };
        let try_from_flag_set = {
            let err = quote! { bit_fields::TryFromFlagSetError };
            let (error, ending) = from_type_parts(err);
            quote! {
                impl<T:std::fmt::Display> std::convert::TryFrom<std::collections::HashSet<T>> for #struct_name {
                    type Error = #error;
                    fn try_from(set: std::collections::HashSet<T>) -> Result<Self,Self::Error> {
                        let value = set.into_iter().try_fold(0,|mut acc,key| match key.to_string().as_str() {
                            #flag_matching_from_hashset
                            _ => Err(bit_fields::TryFromFlagSetError)
                        })?;
                        #ending
                    }
                }
            }
        };

        // - Constructing a bit field from a map of fields
        // - Constructing a map of fields from a reference to the bit field
        let try_from_field_map = {
            let base = data_type.base();
            let err = quote! { bit_fields::TryFromFieldMapError };
            let (error, ending) = from_type_parts(err);
            quote! {
                impl<T:std::fmt::Display, K> std::convert::TryFrom<std::collections::HashMap<T,K>> for #struct_name where #base: From<K> {
                    type Error = #error;
                    fn try_from(map: std::collections::HashMap<T,K>) -> Result<Self,Self::Error> {
                        let value = map.into_iter().try_fold(0,|mut acc,(k,v)|match k.to_string().as_str() {
                            #field_matching_from_hashmap
                            _ => Err(bit_fields::TryFromFieldMapError::UnknownField)
                        })?;
                        #ending
                    }
                }
            }
        };

        let try_from_flag_set_and_field_map = {
            let base = data_type.base();
            let err = quote! { bit_fields::TryFromFlagSetAndFieldMapError };
            let (error, ending) = from_type_parts(err);
            quote! {
                impl<T:std::fmt::Display, K> std::convert::TryFrom<(std::collections::HashSet<T>,std::collections::HashMap<T,K>)> for #struct_name where #base: From<K> {
                    type Error = #error;
                    fn try_from((set,map):(std::collections::HashSet<T>,std::collections::HashMap<T,K>)) -> Result<Self,Self::Error> {
                        let value = set.into_iter().try_fold(0,|mut acc,key| match key.to_string().as_str() {
                            #flag_matching_from_hashset
                            _ => Err(bit_fields::TryFromFlagSetError)
                        }).map_err(bit_fields::TryFromFlagSetAndFieldMapError::FlagSet)?;

                        let value = map.into_iter().try_fold(value,|mut acc,(k,v)|match k.to_string().as_str() {
                            #field_matching_from_hashmap
                            _ => Err(bit_fields::TryFromFieldMapError::UnknownField)
                        }).map_err(bit_fields::TryFromFlagSetAndFieldMapError::FieldMap)?;

                        #ending
                    }
                }
            }
        };

        let binary_ops = if data_type.is_nonzero() {
            quote! {
                impl #struct_name {
                    //
                    unsafe fn unchecked_bitand(self, rhs: Self) -> Self {
                        Self(#data_type::new_unchecked(self.0.get() & rhs.0.get()))
                    }
                    unsafe fn unchecked_not(self) -> Self {
                        Self(#data_type::new_unchecked(!self.0.get()))
                    }
                }
            }
        } else {
            quote! {
                impl std::ops::BitAnd for #struct_name {
                    type Output = Self;
                    fn bitand(self, rhs: Self) -> Self::Output {
                        Self(self.0 & rhs.0)
                    }
                }
                impl std::ops::Not for #struct_name {
                    type Output = Self;
                    fn not(self) -> Self::Output {
                        Self(!self.0)
                    }
                }
            }
        };

        let equal_cmp = if data_type.is_nonzero() {
            quote! { (self.0.get() & mask.0.get()) == (other.0.get() & mask.0.get()) }
        } else {
            quote! { (self.0 & mask.0) == (other.0 & mask.0) }
        };

        let display_impl = {
            let display_full_string = display_string.to_string();
            quote! {
                impl std::fmt::Display for #struct_name {
                    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(f,#display_full_string,#display_fmt_string)
                    }
                }
            }
        };

        let header = format!(
            "A {} bit structure containing a number of bit flags and bit fields.",
            self.data_type.size()
        );

        let (flag_mask, range_mask) = if data_type.is_nonzero() {
            (
                quote! { unsafe { #data_type::new_unchecked(#bit_flag_mask) } },
                quote! { unsafe { #data_type::new_unchecked(#bit_range_mask) } },
            )
        } else {
            (bit_flag_mask, bit_range_mask)
        };

        quote! {
            #[doc=#header]
            ///
            /// Implemented operations such as [`std::cmp::Eq`], [`std::ops::BitOr`] and
            /// [`std::ops::Not`] apply to the underlying integer type.
            ///
            /// ## Layout
            ///
            /// <table>
            #(#struct_doc_table_layout)*
            /// </table>
            #[derive(Debug, Clone, Copy, Eq, PartialEq)]
            #[repr(C)]
            pub struct #struct_name(pub #data_type);

            impl std::cmp::PartialEq<#data_type> for #struct_name {
                fn eq(&self, other: &#data_type) -> bool {
                    self.0 == *other
                }
            }
            impl std::fmt::Binary for #struct_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    std::fmt::Binary::fmt(&self.0, f)
                }
            }
            impl std::fmt::LowerHex for #struct_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    std::fmt::LowerHex::fmt(&self.0, f)
                }
            }
            impl std::fmt::Octal for #struct_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    std::fmt::Octal::fmt(&self.0, f)
                }
            }
            /// Constructs `self` from the data type.
            impl std::convert::From<#data_type> for #struct_name {
                fn from(data: #data_type) -> Self {
                    Self(data)
                }
            }
            /// Constructs the data type from `self`.
            impl std::convert::From<#struct_name> for #data_type {
                fn from(bit_field: #struct_name) -> Self {
                    bit_field.0
                }
            }
            impl std::ops::BitOr for #struct_name {
                type Output = Self;
                fn bitor(self, rhs: Self) -> Self::Output {
                    Self(self.0 | rhs.0)
                }
            }
            impl #struct_name {
                /// Mask for all bit flags.
                ///
                /// If no bit flags are defined this can be a zero value `NonZero` type, this is
                /// private to disallow unsafe usage.
                const BIT_FLAG_MASK: Self = Self(#flag_mask);
                /// Mask for all bit fields.
                ///
                /// If no bit ranges are defined this can be a zero value `NonZero` type, this is
                /// private to disallow unsafe usage.
                const BIT_RANGE_MASK: Self = Self(#range_mask);

                /// Compares whether `self` is equal to `other` ignoring undefined bits.
                pub fn equal(&self, other: &Self) -> bool {
                    let mask = Self::BIT_FLAG_MASK | Self::BIT_RANGE_MASK;
                    #equal_cmp
                }
                /// Returns [`std::cmp::Ordering`] based on bit flags.
                /// - `Some(Ordering::Equal)` - Bit flags match between `self` and `other`.
                /// - `Some(Ordering::Greater)` - Bit flags of `self` are a strict superset of bit flags of `other`.
                /// - `Some(Ordering::Less)` - Bit flags of `self` are a strict subset of bit flags of `other`.
                /// - `None` - None of the above conditions are met.
                pub fn cmp_flags(&self,other: &Self) -> Option<std::cmp::Ordering> {
                    if self.equal(other) {
                        Some(std::cmp::Ordering::Equal)
                    }
                    else if self.superset(other) {
                        Some(std::cmp::Ordering::Greater)
                    }
                    else if self.subset(other) {
                        Some(std::cmp::Ordering::Less)
                    }
                    else {
                        None
                    }
                }
                #struct_accessors
                #set_theory
                #index_fn
                #flag_constants
            }
            #display_impl
            #try_from_flag_set
            impl<T: From<&'static str> + std::cmp::Eq + std::hash::Hash> std::convert::From<&#struct_name> for std::collections::HashSet<T> {
                fn from(bit_field: &#struct_name) -> Self {
                    let mut set = Self::new();
                    #flag_setting_hashset
                    set
                }
            }
            #try_from_field_map
            impl<T: From<&'static str> + std::cmp::Eq + std::hash::Hash> std::convert::From<&#struct_name> for std::collections::HashMap<T,#data_type> {
                fn from(bit_field: &#struct_name) -> Self {
                    let mut map = Self::new();
                    #field_setting_hashmap
                    map
                }
            }
            #try_from_flag_set_and_field_map
            impl<T: From<&'static str> + std::cmp::Eq + std::hash::Hash> std::convert::From<&#struct_name> for (std::collections::HashSet<T>,std::collections::HashMap<T,#data_type>) {
                fn from(bit_field: &#struct_name) -> Self {
                    (
                        bit_field.into(),
                        bit_field.into()
                    )
                }
            }
            #binary_ops
            #bit_index
            #serde
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::arithmetic, clippy::integer_arithmetic)]
    use proc_macro2::{Ident, Span};
    use rand::Rng;

    use super::*;
    use crate::{Field, Flag};

    const COMPOSE_FUZZ_LIMIT: usize = 10;
    const ADD_FUZZ_LIMIT: usize = 100;
    const RAND_STR_LEN: usize = 100;

    // Construct an ident with a given string.
    fn ident(s: &str) -> Ident {
        Ident::new(s, Span::call_site())
    }

    // Construct a pseudo-random ident.
    fn rand_ident<R: Rng>(rng: &mut R) -> Ident {
        Ident::new(&rand_string(rng), Span::call_site())
    }

    // Construct a pseudo-random string.
    #[allow(clippy::as_conversions)]
    fn rand_string<R: Rng>(rng: &mut R) -> String {
        (0..RAND_STR_LEN)
            .map(|_| (rng.gen_range(0..26u8) + 65) as char)
            .collect()
    }

    // Construct a pseudo-random `BitFieldBuilder`.
    fn rand_builder<R: Rng>(rng: &mut R, len: usize) -> BitFieldBuilder {
        let iter = (0..len).map(|_| (rng.gen(), rand_string(rng), rand_ident(rng), rng.gen()));
        iter.fold(BitFieldBuilder::default(), |builder, item| {
            let (start, rustdoc, identifier, stop_opt) = item;
            if let Some(stop) = stop_opt {
                builder.add_bit_range(Field {
                    start,
                    rustdoc,
                    identifier,
                    stop,
                })
            } else {
                builder.add_bit_flag(Flag {
                    index: start,
                    rustdoc,
                    identifier,
                })
            }
        })
    }

    // Construct a default `BitFieldBuilder`
    impl std::default::Default for BitFieldBuilder {
        fn default() -> Self {
            Self::new(ident("DefaultBuilder"), DataTypeToken::default())
        }
    }

    #[test]
    fn builder_default() {
        let _builder = BitFieldBuilder::default();
    }
    #[test]
    fn builder_add_bit_range() {
        let builder = BitFieldBuilder::default();
        builder.add_bit_range(Field {
            start: 0,
            rustdoc: String::from("one rustdoc"),
            identifier: ident("one"),
            stop: 1,
        });
    }
    #[test]
    fn builder_add_bit_flag() {
        let builder = BitFieldBuilder::default();
        builder.add_bit_flag(Flag {
            index: 0,
            rustdoc: String::from("one rustdoc"),
            identifier: ident("one"),
        });
    }
    #[test]
    fn builder_add_fuzz() {
        let mut rng = rand::thread_rng();
        let _builder = rand_builder(&mut rng, ADD_FUZZ_LIMIT);
    }
    #[test]
    fn builder_compose_fuzz() {
        let mut rng = rand::thread_rng();
        for _ in 0..COMPOSE_FUZZ_LIMIT {
            let builder = rand_builder(&mut rng, COMPOSE_FUZZ_LIMIT);
            let _token_stream = builder.compose();
        }
    }
}
