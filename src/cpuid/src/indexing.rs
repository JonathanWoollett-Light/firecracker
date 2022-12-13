// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

/// Indexs leaf.
pub trait IndexLeaf<const INDEX: usize> {
    /// Leaf type.
    type Output<'a>
    where
        Self: 'a;
    /// Gets immutable reference to leaf.
    fn index_leaf<'a>(&'a self) -> Self::Output<'a>;
}
/// Indexs leaf.
pub trait IndexLeafMut<const INDEX: usize> {
    /// Leaf type.
    type Output<'a>
    where
        Self: 'a;
    /// Gets mutable reference to leaf.
    fn index_leaf_mut<'a>(&'a mut self) -> Self::Output<'a>;
}

/// Conveniance macro for indexing shared leaves.
macro_rules! cpuid_index_leaf {
    ($index: literal, $leaf: ty) => {
        impl IndexLeaf<$index> for super::Cpuid {
            type Output<'a> = Option<&'a $leaf>;

            fn index_leaf<'a>(&'a self) -> Self::Output<'a> {
                match self {
                    Self::Intel(intel_cpuid) => intel_cpuid.leaf::<$index>(),
                    Self::Amd(amd_cpuid) => amd_cpuid.leaf::<$index>(),
                }
            }
        }
        impl IndexLeafMut<$index> for super::Cpuid {
            type Output<'a> = Option<&'a mut $leaf>;

            fn index_leaf_mut<'a>(&'a mut self) -> Self::Output<'a> {
                match self {
                    Self::Intel(intel_cpuid) => intel_cpuid.leaf_mut::<$index>(),
                    Self::Amd(amd_cpuid) => amd_cpuid.leaf_mut::<$index>(),
                }
            }
        }
        index_leaf!($index, $leaf, crate::AmdCpuid);
        index_leaf!($index, $leaf, crate::IntelCpuid);
    };
}

// TODO Remove this export
/// Conveniance macro for indexing leaves.
#[macro_export]
macro_rules! index_leaf {
    ($index: literal, $leaf: ty, $cpuid: ty) => {
        impl IndexLeaf<$index> for $cpuid {
            type Output<'a> = Option<&'a $leaf>;

            fn index_leaf<'a>(&'a self) -> Self::Output<'a> {
                self.0
                    .get(&crate::CpuidKey::leaf($index))
                    // SAFETY: Transmuting reference to same sized types is safe.
                    .map(|entry| unsafe { std::mem::transmute::<_, &$leaf>(&entry.result) })
            }
        }
        impl IndexLeafMut<$index> for $cpuid {
            type Output<'a> = Option<&'a mut $leaf>;

            fn index_leaf_mut<'a>(&'a mut self) -> Self::Output<'a> {
                self.0
                    .get_mut(&crate::CpuidKey::leaf($index))
                    // SAFETY: Transmuting reference to same sized types is safe.
                    .map(|entry| unsafe { std::mem::transmute::<_, &mut $leaf>(&mut entry.result) })
            }
        }
    };
}

cpuid_index_leaf!(0x0, super::Leaf0);

cpuid_index_leaf!(0x80000002, super::Leaf80000002);

cpuid_index_leaf!(0x80000003, super::Leaf80000003);

cpuid_index_leaf!(0x80000004, super::Leaf80000004);
