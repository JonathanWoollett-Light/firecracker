// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
#![allow(clippy::similar_names, clippy::module_name_repetitions)]

use std::mem::transmute;

#[allow(clippy::wildcard_imports)]
use super::*;
// -------------------------------------------------------------------------------------------------
// Indexing traits
// -------------------------------------------------------------------------------------------------
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
macro_rules! index_leaf {
    ($index: literal, $leaf: ident) => {
        impl IndexLeaf<$index> for IntelCpuid {
            type Output<'a> = Option<&'a $leaf>;

            fn index_leaf<'a>(&'a self) -> Self::Output<'a> {
                let entry_index_result = self.0.binary_search_by(|a| a.cmp_leaf($index));
                entry_index_result
                    .ok()
                    .map(|i| unsafe { transmute::<_, &$leaf>(&self.0[i].result) })
            }
        }
        impl IndexLeafMut<$index> for IntelCpuid {
            type Output<'a> = Option<&'a mut $leaf>;

            fn index_leaf_mut<'a>(&'a mut self) -> Self::Output<'a> {
                let entry_index_result = self.0.binary_search_by(|a| a.cmp_leaf($index));
                entry_index_result
                    .ok()
                    .map(|i| unsafe { transmute::<_, &mut $leaf>(&mut self.0[i].result) })
            }
        }
    };
}

index_leaf!(0x0, Leaf0);

index_leaf!(0x1, Leaf1);

index_leaf!(0x2, Leaf2);

index_leaf!(0x3, Leaf3);

impl IndexLeaf<0x4> for IntelCpuid {
    type Output<'a> = Leaf4<'a>;
    fn index_leaf<'a>(&'a self) -> Self::Output<'a> {
        // Get 1st leaf 0x4
        let start = match self.0.binary_search_by(|a| a.cmp_leaf(0x4)) {
            Ok(i) => i,
            Err(i) => i,
        };
        // Gets 1st leaf 0x5, which will directly follow the last subleaf for 0x4.
        let end = match self.0.binary_search_by(|a| a.cmp_leaf(0x5)) {
            Ok(i) => i,
            Err(i) => i,
        };
        Leaf4(unsafe { transmute::<_, &[Leaf4Subleaf]>(&self.0[start..end]) })
    }
}
impl IndexLeafMut<0x4> for IntelCpuid {
    type Output<'a> = Leaf4Mut<'a>;
    fn index_leaf_mut<'a>(&'a mut self) -> Self::Output<'a> {
        // Get 1st leaf 0x4
        let start = match self.0.binary_search_by(|a| a.cmp_leaf(0x4)) {
            Ok(i) => i,
            Err(i) => i,
        };
        // Gets 1st leaf 0x5, which will directly follow the last subleaf for 0x4.
        let end = match self.0.binary_search_by(|a| a.cmp_leaf(0x5)) {
            Ok(i) => i,
            Err(i) => i,
        };
        Leaf4Mut(unsafe { transmute::<_, &mut [Leaf4Subleaf]>(&mut self.0[start..end]) })
    }
}
index_leaf!(0x5, Leaf5);

index_leaf!(0x6, Leaf6);

impl IndexLeaf<0x7> for IntelCpuid {
    type Output<'a> = Leaf7<'a>;
    fn index_leaf<'a>(&'a self) -> Self::Output<'a> {
        Leaf7(
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x7, 0x0))
                .ok()
                .map(|i| unsafe { transmute::<_, &Leaf7Subleaf0>(&self.0[i]) }),
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x7, 0x1))
                .ok()
                .map(|i| unsafe { transmute::<_, &Leaf7Subleaf1>(&self.0[i]) }),
        )
    }
}
impl IndexLeafMut<0x7> for IntelCpuid {
    type Output<'a> = Leaf7Mut<'a>;
    fn index_leaf_mut<'a>(&'a mut self) -> Self::Output<'a> {
        Leaf7Mut(
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x7, 0x0))
                .ok()
                .map(|i| unsafe { transmute::<_, &mut Leaf7Subleaf0>(&mut self.0[i]) }),
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x7, 0x1))
                .ok()
                .map(|i| unsafe { transmute::<_, &mut Leaf7Subleaf1>(&mut self.0[i]) }),
        )
    }
}

index_leaf!(0x9, Leaf9);

index_leaf!(0xA, LeafA);

impl IndexLeaf<0xB> for IntelCpuid {
    type Output<'a> = LeafB<'a>;
    fn index_leaf<'a>(&'a self) -> Self::Output<'a> {
        // Get 1st leaf 0x4
        let start = match self.0.binary_search_by(|a| a.cmp_leaf(0xB)) {
            Ok(i) => i,
            Err(i) => i,
        };
        // Gets 1st leaf 0xC, which will directly follow the last subleaf for 0xB.
        let end = match self.0.binary_search_by(|a| a.cmp_leaf(0xC)) {
            Ok(i) => i,
            Err(i) => i,
        };
        LeafB(unsafe { transmute::<_, &[LeafBSubleaf]>(&self.0[start..end]) })
    }
}
impl IndexLeafMut<0xB> for IntelCpuid {
    type Output<'a> = LeafBMut<'a>;
    fn index_leaf_mut<'a>(&'a mut self) -> Self::Output<'a> {
        // Get 1st leaf 0x4
        let start = match self.0.binary_search_by(|a| a.cmp_leaf(0xB)) {
            Ok(i) => i,
            Err(i) => i,
        };
        // Gets 1st leaf 0xC, which will directly follow the last subleaf for 0xB.
        let end = match self.0.binary_search_by(|a| a.cmp_leaf(0xC)) {
            Ok(i) => i,
            Err(i) => i,
        };
        LeafBMut(unsafe { transmute::<_, &mut [LeafBSubleaf]>(&mut self.0[start..end]) })
    }
}

impl IndexLeaf<0xF> for IntelCpuid {
    type Output<'a> = LeafF<'a>;
    fn index_leaf<'a>(&'a self) -> Self::Output<'a> {
        LeafF(
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0xF, 0x0))
                .ok()
                .map(|i| unsafe { transmute::<_, &LeafFSubleaf0>(&self.0[i]) }),
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0xF, 0x1))
                .ok()
                .map(|i| unsafe { transmute::<_, &LeafFSubleaf1>(&self.0[i]) }),
        )
    }
}
impl IndexLeafMut<0xF> for IntelCpuid {
    type Output<'a> = LeafFMut<'a>;
    fn index_leaf_mut<'a>(&'a mut self) -> Self::Output<'a> {
        LeafFMut(
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0xF, 0x0))
                .ok()
                .map(|i| unsafe { transmute::<_, &mut LeafFSubleaf0>(&mut self.0[i]) }),
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0xF, 0x1))
                .ok()
                .map(|i| unsafe { transmute::<_, &mut LeafFSubleaf1>(&mut self.0[i]) }),
        )
    }
}

impl IndexLeaf<0x10> for IntelCpuid {
    type Output<'a> = Leaf10<'a>;
    fn index_leaf<'a>(&'a self) -> Self::Output<'a> {
        Leaf10(
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x10, 0x0))
                .ok()
                .map(|i| unsafe { transmute::<_, &Leaf10Subleaf0>(&self.0[i]) }),
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x10, 0x1))
                .ok()
                .map(|i| unsafe { transmute::<_, &Leaf10Subleaf1>(&self.0[i]) }),
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x10, 0x2))
                .ok()
                .map(|i| unsafe { transmute::<_, &Leaf10Subleaf2>(&self.0[i]) }),
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x10, 0x3))
                .ok()
                .map(|i| unsafe { transmute::<_, &Leaf10Subleaf3>(&self.0[i]) }),
        )
    }
}
impl IndexLeafMut<0x10> for IntelCpuid {
    type Output<'a> = Leaf10Mut<'a>;
    fn index_leaf_mut<'a>(&'a mut self) -> Self::Output<'a> {
        Leaf10Mut(
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x10, 0x0))
                .ok()
                .map(|i| unsafe { transmute::<_, &mut Leaf10Subleaf0>(&mut self.0[i]) }),
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x10, 0x1))
                .ok()
                .map(|i| unsafe { transmute::<_, &mut Leaf10Subleaf1>(&mut self.0[i]) }),
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x10, 0x2))
                .ok()
                .map(|i| unsafe { transmute::<_, &mut Leaf10Subleaf2>(&mut self.0[i]) }),
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x10, 0x3))
                .ok()
                .map(|i| unsafe { transmute::<_, &mut Leaf10Subleaf3>(&mut self.0[i]) }),
        )
    }
}

impl IndexLeaf<0x12> for IntelCpuid {
    type Output<'a> = Leaf12<'a>;
    fn index_leaf<'a>(&'a self) -> Self::Output<'a> {
        Leaf12(
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x12, 0x0))
                .ok()
                .map(|i| unsafe { transmute::<_, &Leaf12Subleaf0>(&self.0[i]) }),
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x12, 0x1))
                .ok()
                .map(|i| unsafe { transmute::<_, &Leaf12Subleaf1>(&self.0[i]) }),
            {
                // Get 3rd leaf 0x12
                let start = match self.0.binary_search_by(|a| a.cmp_subleaf(0x12, 0x2)) {
                    Ok(i) => i,
                    Err(i) => i,
                };
                // Gets 1st leaf 0x13, which will directly follow the last subleaf for 0x12.
                let end = match self.0.binary_search_by(|a| a.cmp_leaf(0x13)) {
                    Ok(i) => i,
                    Err(i) => i,
                };
                unsafe { transmute::<_, &[Leaf12SubleafGt1]>(&self.0[start..end]) }
            },
        )
    }
}
impl IndexLeafMut<0x12> for IntelCpuid {
    type Output<'a> = Leaf12Mut<'a>;
    fn index_leaf_mut<'a>(&'a mut self) -> Self::Output<'a> {
        Leaf12Mut(
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x12, 0x0))
                .ok()
                .map(|i| unsafe { transmute::<_, &mut Leaf12Subleaf0>(&mut self.0[i]) }),
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x12, 0x1))
                .ok()
                .map(|i| unsafe { transmute::<_, &mut Leaf12Subleaf1>(&mut self.0[i]) }),
            {
                // Get 3rd leaf 0x12
                let start = match self.0.binary_search_by(|a| a.cmp_subleaf(0x12, 0x2)) {
                    Ok(i) => i,
                    Err(i) => i,
                };
                // Gets 1st leaf 0x13, which will directly follow the last subleaf for 0x12.
                let end = match self.0.binary_search_by(|a| a.cmp_leaf(0x13)) {
                    Ok(i) => i,
                    Err(i) => i,
                };
                unsafe { transmute::<_, &mut [Leaf12SubleafGt1]>(&mut self.0[start..end]) }
            },
        )
    }
}

impl IndexLeaf<0x14> for IntelCpuid {
    type Output<'a> = Leaf14<'a>;
    fn index_leaf<'a>(&'a self) -> Self::Output<'a> {
        Leaf14(
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x14, 0x0))
                .ok()
                .map(|i| unsafe { transmute::<_, &Leaf14Subleaf0>(&self.0[i]) }),
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x14, 0x1))
                .ok()
                .map(|i| unsafe { transmute::<_, &Leaf14Subleaf1>(&self.0[i]) }),
        )
    }
}
impl IndexLeafMut<0x14> for IntelCpuid {
    type Output<'a> = Leaf14Mut<'a>;
    fn index_leaf_mut<'a>(&'a mut self) -> Self::Output<'a> {
        Leaf14Mut(
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x14, 0x0))
                .ok()
                .map(|i| unsafe { transmute::<_, &mut Leaf14Subleaf0>(&mut self.0[i]) }),
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x14, 0x1))
                .ok()
                .map(|i| unsafe { transmute::<_, &mut Leaf14Subleaf1>(&mut self.0[i]) }),
        )
    }
}

index_leaf!(0x15, Leaf15);

index_leaf!(0x16, Leaf16);

impl IndexLeaf<0x17> for IntelCpuid {
    type Output<'a> = Leaf17<'a>;
    fn index_leaf<'a>(&'a self) -> Self::Output<'a> {
        Leaf17(
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x17, 0x0))
                .ok()
                .map(|i| unsafe { transmute::<_, &Leaf17Subleaf0>(&self.0[i]) }),
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x17, 0x1))
                .ok()
                .map(|i| unsafe { transmute::<_, &Leaf17Subleaf1>(&self.0[i]) }),
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x17, 0x2))
                .ok()
                .map(|i| unsafe { transmute::<_, &Leaf17Subleaf2>(&self.0[i]) }),
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x17, 0x3))
                .ok()
                .map(|i| unsafe { transmute::<_, &Leaf17Subleaf3>(&self.0[i]) }),
        )
    }
}
impl IndexLeafMut<0x17> for IntelCpuid {
    type Output<'a> = Leaf17Mut<'a>;
    fn index_leaf_mut<'a>(&'a mut self) -> Self::Output<'a> {
        Leaf17Mut(
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x17, 0x0))
                .ok()
                .map(|i| unsafe { transmute::<_, &mut Leaf17Subleaf0>(&mut self.0[i]) }),
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x17, 0x1))
                .ok()
                .map(|i| unsafe { transmute::<_, &mut Leaf17Subleaf1>(&mut self.0[i]) }),
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x17, 0x2))
                .ok()
                .map(|i| unsafe { transmute::<_, &mut Leaf17Subleaf2>(&mut self.0[i]) }),
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x17, 0x3))
                .ok()
                .map(|i| unsafe { transmute::<_, &mut Leaf17Subleaf3>(&mut self.0[i]) }),
        )
    }
}

impl IndexLeaf<0x18> for IntelCpuid {
    type Output<'a> = Leaf18<'a>;
    fn index_leaf<'a>(&'a self) -> Self::Output<'a> {
        Leaf18(
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x18, 0x0))
                .ok()
                .map(|i| unsafe { transmute(&self.0[i]) }),
            {
                // Get 2nd leaf 0x18
                let start = match self.0.binary_search_by(|a| a.cmp_subleaf(0x18, 0x1)) {
                    Ok(i) => i,
                    Err(i) => i,
                };
                // Gets 1st leaf 0x19, which will directly follow the last subleaf for 0x18.
                let end = match self.0.binary_search_by(|a| a.cmp_leaf(0x19)) {
                    Ok(i) => i,
                    Err(i) => i,
                };
                unsafe { transmute::<_, &[Leaf18SubleafGt0]>(&self.0[start..end]) }
            },
        )
    }
}
impl IndexLeafMut<0x18> for IntelCpuid {
    type Output<'a> = Leaf18Mut<'a>;
    fn index_leaf_mut<'a>(&'a mut self) -> Self::Output<'a> {
        Leaf18Mut(
            self.0
                .binary_search_by(|a| a.cmp_subleaf(0x18, 0x0))
                .ok()
                .map(|i| unsafe { transmute(&mut self.0[i]) }),
            {
                // Get 2nd leaf 0x18
                let start = match self.0.binary_search_by(|a| a.cmp_subleaf(0x18, 0x1)) {
                    Ok(i) => i,
                    Err(i) => i,
                };
                // Gets 1st leaf 0x19, which will directly follow the last subleaf for 0x18.
                let end = match self.0.binary_search_by(|a| a.cmp_leaf(0x19)) {
                    Ok(i) => i,
                    Err(i) => i,
                };
                unsafe { transmute::<_, &mut [Leaf18SubleafGt0]>(&mut self.0[start..end]) }
            },
        )
    }
}

index_leaf!(0x19, Leaf19);

index_leaf!(0x1A, Leaf1A);

index_leaf!(0x1B, Leaf1B);

index_leaf!(0x1C, Leaf1C);

impl IndexLeaf<0x1F> for IntelCpuid {
    type Output<'a> = Leaf1F<'a>;
    fn index_leaf<'a>(&'a self) -> Self::Output<'a> {
        // Get 1st leaf 0x1F
        let start = match self.0.binary_search_by(|a| a.cmp_leaf(0x1F)) {
            Ok(i) => i,
            Err(i) => i,
        };
        // Gets 1st leaf 0x20, which will directly follow the last subleaf for 0x1F.
        let end = match self.0.binary_search_by(|a| a.cmp_leaf(0x20)) {
            Ok(i) => i,
            Err(i) => i,
        };
        Leaf1F(unsafe { transmute::<_, &[Leaf1FSubleaf]>(&self.0[start..end]) })
    }
}
impl IndexLeafMut<0x1F> for IntelCpuid {
    type Output<'a> = Leaf1FMut<'a>;
    fn index_leaf_mut<'a>(&'a mut self) -> Self::Output<'a> {
        // Get 1st leaf 0x1F
        let start = match self.0.binary_search_by(|a| a.cmp_leaf(0x1F)) {
            Ok(i) => i,
            Err(i) => i,
        };
        // Gets 1st leaf 0x20, which will directly follow the last subleaf for 0x1F.
        let end = match self.0.binary_search_by(|a| a.cmp_leaf(0x20)) {
            Ok(i) => i,
            Err(i) => i,
        };
        Leaf1FMut(unsafe { transmute::<_, &mut [Leaf1FSubleaf]>(&mut self.0[start..end]) })
    }
}

index_leaf!(0x20, Leaf20);
index_leaf!(0x80000000, Leaf80000000);
index_leaf!(0x80000001, Leaf80000001);
index_leaf!(0x80000002, Leaf80000002);
index_leaf!(0x80000003, Leaf80000003);
index_leaf!(0x80000004, Leaf80000004);
index_leaf!(0x80000005, Leaf80000005);
index_leaf!(0x80000006, Leaf80000006);
index_leaf!(0x80000007, Leaf80000007);
index_leaf!(0x80000008, Leaf80000008);
