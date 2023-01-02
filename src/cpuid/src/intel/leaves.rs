// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
#![allow(clippy::similar_names, clippy::module_name_repetitions, missing_docs)]
use std::cmp::Ordering;
use std::fmt;

use bit_fields::Equal;

#[allow(clippy::wildcard_imports)]
use super::*;
use crate::{warn_support, Leaf, Supports};

/// Cache and TLB infomation keywords.
#[allow(clippy::non_ascii_literal)]
static KEYWORDS: phf::Map<u8, &'static str> = phf::phf_map! {
    0x00u8 => "Null descriptor, this byte contains no information",
    0x01u8 => "Instruction TLB: 4 KByte pages, 4-way set associative, 32 entries",
    0x02u8 => "Instruction TLB: 4 MByte pages, fully associative, 2 entries",
    0x03u8 => "Data TLB: 4 KByte pages, 4-way set associative, 64 entries",
    0x04u8 => "Data TLB: 4 MByte pages, 4-way set associative, 8 entries",
    0x05u8 => "Data TLB1: 4 MByte pages, 4-way set associative, 32 entries",
    0x06u8 => "1st-level instruction cache: 8 KBytes, 4-way set associative, 32 byte line size",
    0x08u8 => "1st-level instruction cache: 16 KBytes, 4-way set associative, 32 byte line size",
    0x09u8 => "1st-level instruction cache: 32KBytes, 4-way set associative, 64 byte line size",
    0x0Au8 => "1st-level data cache: 8 KBytes, 2-way set associative, 32 byte line size",
    0x0Bu8 => "Instruction TLB: 4 MByte pages, 4-way set associative, 4 entries",
    0x0Cu8 => "1st-level data cache: 16 KBytes, 4-way set associative, 32 byte line size",
    0x0Du8 => "1st-level data cache: 16 KBytes, 4-way set associative, 64 byte line size",
    0x0Eu8 => "1st-level data cache: 24 KBytes, 6-way set associative, 64 byte line size",
    0x1Du8 => "2nd-level cache: 128 KBytes, 2-way set associative, 64 byte line size",
    0x21u8 => "2nd-level cache: 256 KBytes, 8-way set associative, 64 byte line size",
    0x22u8 => "3rd-level cache: 512 KBytes, 4-way set associative, 64 byte line size, 2 lines per sector",
    0x23u8 => "3rd-level cache: 1 MBytes, 8-way set associative, 64 byte line size, 2 lines per sector",
    0x24u8 => "2nd-level cache: 1 MBytes, 16-way set associative, 64 byte line size",
    0x25u8 => "3rd-level cache: 2 MBytes, 8-way set associative, 64 byte line size, 2 lines per sector",
    0x29u8 => "3rd-level cache: 4 MBytes, 8-way set associative, 64 byte line size, 2 lines per sector",
    0x2Cu8 => "1st-level data cache: 32 KBytes, 8-way set associative, 64 byte line size",
    0x30u8 => "1st-level instruction cache: 32 KBytes, 8-way set associative, 64 byte line size",
    0x40u8 => "No 2nd-level cache or, if processor contains a valid 2nd-level cache, no 3rd-level cache",
    0x41u8 => "2nd-level cache: 128 KBytes, 4-way set associative, 32 byte line size",
    0x42u8 => "2nd-level cache: 256 KBytes, 4-way set associative, 32 byte line size",
    0x43u8 => "2nd-level cache: 512 KBytes, 4-way set associative, 32 byte line size",
    0x44u8 => "2nd-level cache: 1 MByte, 4-way set associative, 32 byte line size",
    0x45u8 => "2nd-level cache: 2 MByte, 4-way set associative, 32 byte line size",
    0x46u8 => "3rd-level cache: 4 MByte, 4-way set associative, 64 byte line size",
    0x47u8 => "3rd-level cache: 8 MByte, 8-way set associative, 64 byte line size",
    0x48u8 => "2nd-level cache: 3MByte, 12-way set associative, 64 byte line size",
    0x49u8 => "3rd-level cache: 4MB, 16-way set associative, 64-byte line size (Intel Xeon processor MP, Family 0FH, Model 06H);\n2nd-level cache: 4 MByte, 16-way set associative, 64 byte line size",
    0x4Au8 => "3rd-level cache: 6MByte, 12-way set associative, 64 byte line size",
    0x4Bu8 => "3rd-level cache: 8MByte, 16-way set associative, 64 byte line size",
    0x4Cu8 => "3rd-level cache: 12MByte, 12-way set associative, 64 byte line size",
    0x4Du8 => "3rd-level cache: 16MByte, 16-way set associative, 64 byte line size",
    0x4Eu8 => "2nd-level cache: 6MByte, 24-way set associative, 64 byte line size",
    0x4Fu8 => "Instruction TLB: 4 KByte pages, 32 entries",
    0x50u8 => "Instruction TLB: 4 KByte and 2-MByte or 4-MByte pages, 64 entries",
    0x51u8 => "Instruction TLB: 4 KByte and 2-MByte or 4-MByte pages, 128 entries",
    0x52u8 => "Instruction TLB: 4 KByte and 2-MByte or 4-MByte pages, 256 entries",
    0x55u8 => "Instruction TLB: 2-MByte or 4-MByte pages, fully associative, 7 entries",
    0x56u8 => "Data TLB0: 4 MByte pages, 4-way set associative, 16 entries",
    0x57u8 => "Data TLB0: 4 KByte pages, 4-way associative, 16 entries",
    0x59u8 => "Data TLB0: 4 KByte pages, fully associative, 16 entries",
    0x5Au8 => "Data TLB0: 2 MByte or 4 MByte pages, 4-way set associative, 32 entries",
    0x5Bu8 => "Data TLB: 4 KByte and 4 MByte pages, 64 entries",
    0x5Cu8 => "Data TLB: 4 KByte and 4 MByte pages, 128 entries",
    0x5Du8 => "Data TLB: 4 KByte and 4 MByte pages, 256 entries",
    0x60u8 => "1st-level data cache: 16 KByte, 8-way set associative, 64 byte line size",
    0x61u8 => "Instruction TLB: 4 KByte pages, fully associative, 48 entries",
    0x63u8 => "Data TLB: 2 MByte or 4 MByte pages, 4-way set associative, 32 entries and a separate array with 1 GByte pages, 4-way set associative, 4 entries",
    0x64u8 => "Data TLB: 4 KByte pages, 4-way set associative, 512 entries",
    0x66u8 => "1st-level data cache: 8 KByte, 4-way set associative, 64 byte line size",
    0x67u8 => "1st-level data cache: 16 KByte, 4-way set associative, 64 byte line size",
    0x68u8 => "1st-level data cache: 32 KByte, 4-way set associative, 64 byte line size",
    0x6Au8 => "uTLB: 4 KByte pages, 8-way set associative, 64 entries",
    0x6Bu8 => "DTLB: 4 KByte pages, 8-way set associative, 256 entries",
    0x6Cu8 => "DTLB: 2M/4M pages, 8-way set associative, 128 entries",
    0x6Du8 => "DTLB: 1 GByte pages, fully associative, 16 entries",
    0x70u8 => "Trace cache: 12 K-μop, 8-way set associative",
    0x71u8 => "Trace cache: 16 K-μop, 8-way set associative",
    0x72u8 => "Trace cache: 32 K-μop, 8-way set associative",
    0x76u8 => "Instruction TLB: 2M/4M pages, fully associative, 8 entries",
    0x78u8 => "2nd-level cache: 1 MByte, 4-way set associative, 64byte line size",
    0x79u8 => "2nd-level cache: 128 KByte, 8-way set associative, 64 byte line size, 2 lines per sector",
    0x7Au8 => "2nd-level cache: 256 KByte, 8-way set associative, 64 byte line size, 2 lines per sector",
    0x7Bu8 => "2nd-level cache: 512 KByte, 8-way set associative, 64 byte line size, 2 lines per sector",
    0x7Cu8 => "2nd-level cache: 1 MByte, 8-way set associative, 64 byte line size, 2 lines per sector",
    0x7Du8 => "2nd-level cache: 2 MByte, 8-way set associative, 64byte line size",
    0x7Fu8 => "2nd-level cache: 512 KByte, 2-way set associative, 64-byte line size",
    0x80u8 => "2nd-level cache: 512 KByte, 8-way set associative, 64-byte line size",
    0x82u8 => "2nd-level cache: 256 KByte, 8-way set associative, 32 byte line size",
    0x83u8 => "2nd-level cache: 512 KByte, 8-way set associative, 32 byte line size",
    0x84u8 => "2nd-level cache: 1 MByte, 8-way set associative, 32 byte line size",
    0x85u8 => "2nd-level cache: 2 MByte, 8-way set associative, 32 byte line size",
    0x86u8 => "2nd-level cache: 512 KByte, 4-way set associative, 64 byte line size",
    0x87u8 => "2nd-level cache: 1 MByte, 8-way set associative, 64 byte line size",
    0xA0u8 => "DTLB: 4k pages, fully associative, 32 entries",
    0xB0u8 => "Instruction TLB: 4 KByte pages, 4-way set associative, 128 entries",
    0xB1u8 => "Instruction TLB: 2M pages, 4-way, 8 entries or 4M pages, 4-way, 4 entries",
    0xB2u8 => "Instruction TLB: 4KByte pages, 4-way set associative, 64 entries",
    0xB3u8 => "Data TLB: 4 KByte pages, 4-way set associative, 128 entries",
    0xB4u8 => "Data TLB1: 4 KByte pages, 4-way associative, 256 entries",
    0xB5u8 => "Instruction TLB: 4KByte pages, 8-way set associative, 64 entries",
    0xB6u8 => "Instruction TLB: 4KByte pages, 8-way set associative, 128 entries",
    0xBAu8 => "Data TLB1: 4 KByte pages, 4-way associative, 64 entries",
    0xC0u8 => "Data TLB: 4 KByte and 4 MByte pages, 4-way associative, 8 entries",
    0xC1u8 => "Shared 2nd-Level TLB: 4 KByte / 2 MByte pages, 8-way associative, 1024 entries",
    0xC2u8 => "DTLB: 4 KByte/2 MByte pages, 4-way associative, 16 entries",
    0xC3u8 => "Shared 2nd-Level TLB: 4 KByte / 2 MByte pages, 6-way associative, 1536 entries. Also 1GBbyte pages, 4-way, 16 entries.",
    0xC4u8 => "DTLB: 2M/4M Byte pages, 4-way associative, 32 entries",
    0xCAu8 => "Shared 2nd-Level TLB: 4 KByte pages, 4-way associative, 512 entries",
    0xD0u8 => "3rd-level cache: 512 KByte, 4-way set associative, 64 byte line size",
    0xD1u8 => "3rd-level cache: 1 MByte, 4-way set associative, 64 byte line size",
    0xD2u8 => "3rd-level cache: 2 MByte, 4-way set associative, 64 byte line size",
    0xD6u8 => "3rd-level cache: 1 MByte, 8-way set associative, 64 byte line size",
    0xD7u8 => "3rd-level cache: 2 MByte, 8-way set associative, 64 byte line size",
    0xD8u8 => "3rd-level cache: 4 MByte, 8-way set associative, 64 byte line size",
    0xDCu8 => "3rd-level cache: 1.5 MByte, 12-way set associative, 64 byte line size",
    0xDDu8 => "3rd-level cache: 3 MByte, 12-way set associative, 64 byte line size",
    0xDEu8 => "3rd-level cache: 6 MByte, 12-way set associative, 64 byte line size",
    0xE2u8 => "3rd-level cache: 2 MByte, 16-way set associative, 64 byte line size",
    0xE3u8 => "3rd-level cache: 4 MByte, 16-way set associative, 64 byte line size",
    0xE4u8 => "3rd-level cache: 8 MByte, 16-way set associative, 64 byte line size",
    0xEAu8 => "3rd-level cache: 12MByte, 24-way set associative, 64 byte line size",
    0xEBu8 => "3rd-level cache: 18MByte, 24-way set associative, 64 byte line size",
    0xECu8 => "3rd-level cache: 24MByte, 24-way set associative, 64 byte line size",
    0xF0u8 => "64-Byte prefetching",
    0xF1u8 => "128-Byte prefetching",
    0xFEu8 => "CPUID leaf 2 does not report TLB descriptor information; use CPUID leaf 18H to query TLB and other address translation parameters.",
    0xFFu8 => "CPUID leaf 2 does not report cache descriptor information, use CPUID leaf 4 to query cache parameters"
};
impl fmt::Display for Leaf2 {
    #[allow(clippy::unwrap_used, clippy::unwrap_in_result)]
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", <Vec<&'static str>>::try_from(self).unwrap())
    }
}

/// Error type for [`<[&'static str; 16] as TryFrom<&Leaf2>>::try_from`].
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
#[error("Unknown cache and TLB information keyword: {0}")]
pub struct UnknownKeyword(u8);

/// Most significant bit
fn most_significant_bit(x: u8) -> bool {
    // SAFETY: 1 is less than the number of bits in `u8`.
    let mask = unsafe { u8::MAX.checked_shr(1).unwrap_unchecked() };
    (x & mask) == 1
}

impl TryFrom<&Leaf2> for Vec<&'static str> {
    type Error = UnknownKeyword;
    #[inline]
    fn try_from(leaf: &Leaf2) -> Result<Self, Self::Error> {
        let (a_opt, b_opt, c_opt, d_opt) = <(
            Option<[&'static str; 3]>,
            Option<[&'static str; 4]>,
            Option<[&'static str; 4]>,
            Option<[&'static str; 4]>,
        )>::try_from(leaf)?;
        let mut vec = Vec::new();
        if let Some(a) = a_opt {
            vec.extend(a);
        }
        if let Some(b) = b_opt {
            vec.extend(b);
        }
        if let Some(c) = c_opt {
            vec.extend(c);
        }
        if let Some(d) = d_opt {
            vec.extend(d);
        }
        Ok(vec)
    }
}
// - The least-significant-byte of eax always returns 01h, this value should be ignored.
// - The most significant bit is set to 0 when the register contains valid 1-byte descriptors.
impl TryFrom<&Leaf2>
    for (
        Option<[&'static str; 3]>,
        Option<[&'static str; 4]>,
        Option<[&'static str; 4]>,
        Option<[&'static str; 4]>,
    )
{
    type Error = UnknownKeyword;
    #[inline]
    fn try_from(leaf: &Leaf2) -> Result<Self, Self::Error> {
        Ok((
            if most_significant_bit(leaf.eax[3]) {
                None
            } else {
                Some([
                    KEYWORDS
                        .get(&leaf.eax[1])
                        .ok_or(UnknownKeyword(leaf.eax[1]))?,
                    KEYWORDS
                        .get(&leaf.eax[2])
                        .ok_or(UnknownKeyword(leaf.eax[2]))?,
                    KEYWORDS
                        .get(&leaf.eax[3])
                        .ok_or(UnknownKeyword(leaf.eax[3]))?,
                ])
            },
            if most_significant_bit(leaf.ebx[3]) {
                None
            } else {
                Some([
                    KEYWORDS
                        .get(&leaf.ebx[0])
                        .ok_or(UnknownKeyword(leaf.ebx[0]))?,
                    KEYWORDS
                        .get(&leaf.ebx[1])
                        .ok_or(UnknownKeyword(leaf.ebx[1]))?,
                    KEYWORDS
                        .get(&leaf.ebx[2])
                        .ok_or(UnknownKeyword(leaf.ebx[2]))?,
                    KEYWORDS
                        .get(&leaf.ebx[3])
                        .ok_or(UnknownKeyword(leaf.ebx[3]))?,
                ])
            },
            if most_significant_bit(leaf.ecx[3]) {
                None
            } else {
                Some([
                    KEYWORDS
                        .get(&leaf.ecx[0])
                        .ok_or(UnknownKeyword(leaf.ecx[0]))?,
                    KEYWORDS
                        .get(&leaf.ecx[1])
                        .ok_or(UnknownKeyword(leaf.ecx[1]))?,
                    KEYWORDS
                        .get(&leaf.ecx[2])
                        .ok_or(UnknownKeyword(leaf.ecx[2]))?,
                    KEYWORDS
                        .get(&leaf.ecx[3])
                        .ok_or(UnknownKeyword(leaf.ecx[3]))?,
                ])
            },
            if most_significant_bit(leaf.edx[3]) {
                None
            } else {
                Some([
                    KEYWORDS
                        .get(&leaf.edx[0])
                        .ok_or(UnknownKeyword(leaf.edx[0]))?,
                    KEYWORDS
                        .get(&leaf.edx[1])
                        .ok_or(UnknownKeyword(leaf.edx[1]))?,
                    KEYWORDS
                        .get(&leaf.edx[2])
                        .ok_or(UnknownKeyword(leaf.edx[2]))?,
                    KEYWORDS
                        .get(&leaf.edx[3])
                        .ok_or(UnknownKeyword(leaf.edx[3]))?,
                ])
            },
        ))
    }
}
impl From<(u32, u32, u32, u32)> for Leaf2 {
    #[inline]
    fn from((eax, ebx, ecx, edx): (u32, u32, u32, u32)) -> Self {
        Self {
            eax: eax.to_ne_bytes(),
            ebx: ebx.to_ne_bytes(),
            ecx: ecx.to_ne_bytes(),
            edx: edx.to_ne_bytes(),
        }
    }
}

// -------------------------------------------------------------------------------------------------
// Leaf types
// -------------------------------------------------------------------------------------------------

/// Leaf 02H
pub type Leaf2 = Leaf<[u8; 4], [u8; 4], [u8; 4], [u8; 4]>;

/// Leaf 03H
pub type Leaf3 = Leaf<Leaf3Eax, Leaf3Ebx, Leaf3Ecx, Leaf3Edx>;

/// Leaf 04H
#[derive(Debug, PartialEq, Eq)]
pub struct Leaf4<'a>(pub Vec<&'a Leaf4Subleaf>);
/// Leaf 04H
#[derive(Debug, PartialEq, Eq)]
pub struct Leaf4Mut<'a>(pub Vec<&'a mut Leaf4Subleaf>);
/// Leaf 04H subleaf
pub type Leaf4Subleaf = Leaf<Leaf4Eax, Leaf4Ebx, Leaf4Ecx, Leaf4Edx>;

/// Leaf 05H
pub type Leaf5 = Leaf<Leaf5Eax, Leaf5Ebx, Leaf5Ecx, Leaf5Edx>;

/// Leaf 06H
pub type Leaf6 = Leaf<Leaf6Eax, Leaf6Ebx, Leaf6Ecx, Leaf6Edx>;

/// Leaf 07H
#[derive(Debug, PartialEq, Eq)]
pub struct Leaf7<'a>(pub Option<&'a Leaf7Subleaf0>, pub Option<&'a Leaf7Subleaf1>);
/// Leaf 07H
#[derive(Debug, PartialEq, Eq)]
pub struct Leaf7Mut<'a>(
    pub Option<&'a mut Leaf7Subleaf0>,
    pub Option<&'a mut Leaf7Subleaf1>,
);
/// Leaf 07H subleaf 0
pub type Leaf7Subleaf0 =
    Leaf<Leaf7Subleaf0Eax, Leaf7Subleaf0Ebx, Leaf7Subleaf0Ecx, Leaf7Subleaf0Edx>;
/// Leaf 07H subleaf 1
pub type Leaf7Subleaf1 =
    Leaf<Leaf7Subleaf1Eax, Leaf7Subleaf1Ebx, Leaf7Subleaf1Ecx, Leaf7Subleaf1Edx>;

/// Leaf 09H
pub type Leaf9 = Leaf<Leaf9Eax, Leaf9Ebx, Leaf9Ecx, Leaf9Edx>;

/// Leaf 0AH
pub type LeafA = Leaf<LeafAEax, LeafAEbx, LeafAEcx, LeafAEdx>;

/// Leaf 0BH
#[derive(Debug, PartialEq, Eq)]
pub struct LeafB<'a>(pub Vec<&'a LeafBSubleaf>);
/// Leaf 0BH
#[derive(Debug, PartialEq, Eq)]
pub struct LeafBMut<'a>(pub Vec<&'a mut LeafBSubleaf>);
/// Leaf 0BH subleaf
pub type LeafBSubleaf = Leaf<LeafBEax, LeafBEbx, LeafBEcx, LeafBEdx>;

/// Leaf 0FH
#[derive(Debug, PartialEq, Eq)]
pub struct LeafF<'a>(pub Option<&'a LeafFSubleaf0>, pub Option<&'a LeafFSubleaf1>);
/// Leaf 0FH
#[derive(Debug, PartialEq, Eq)]
pub struct LeafFMut<'a>(
    pub Option<&'a mut LeafFSubleaf0>,
    pub Option<&'a mut LeafFSubleaf1>,
);
/// Leaf 0FH subleaf 0
pub type LeafFSubleaf0 =
    Leaf<LeafFSubleaf0Eax, LeafFSubleaf0Ebx, LeafFSubleaf0Ecx, LeafFSubleaf0Edx>;
/// Leaf 0FH subleaf 1
pub type LeafFSubleaf1 =
    Leaf<LeafFSubleaf1Eax, LeafFSubleaf1Ebx, LeafFSubleaf1Ecx, LeafFSubleaf1Edx>;

/// Leaf 10H
#[derive(Debug, PartialEq, Eq)]
pub struct Leaf10<'a>(
    pub Option<&'a Leaf10Subleaf0>,
    pub Option<&'a Leaf10Subleaf1>,
    pub Option<&'a Leaf10Subleaf2>,
    pub Option<&'a Leaf10Subleaf3>,
);
/// Leaf 10H
#[derive(Debug, PartialEq, Eq)]
pub struct Leaf10Mut<'a>(
    pub Option<&'a mut Leaf10Subleaf0>,
    pub Option<&'a mut Leaf10Subleaf1>,
    pub Option<&'a mut Leaf10Subleaf2>,
    pub Option<&'a mut Leaf10Subleaf3>,
);
/// Leaf 10H subleaf 0
pub type Leaf10Subleaf0 =
    Leaf<Leaf10Subleaf0Eax, Leaf10Subleaf0Ebx, Leaf10Subleaf0Ecx, Leaf10Subleaf0Edx>;
/// Leaf 10H subleaf 1
pub type Leaf10Subleaf1 =
    Leaf<Leaf10Subleaf1Eax, Leaf10Subleaf1Ebx, Leaf10Subleaf1Ecx, Leaf10Subleaf1Edx>;
/// Leaf 10H subleaf 2
pub type Leaf10Subleaf2 =
    Leaf<Leaf10Subleaf2Eax, Leaf10Subleaf2Ebx, Leaf10Subleaf2Ecx, Leaf10Subleaf2Edx>;
/// Leaf 10H subleaf 3
pub type Leaf10Subleaf3 =
    Leaf<Leaf10Subleaf3Eax, Leaf10Subleaf3Ebx, Leaf10Subleaf3Ecx, Leaf10Subleaf3Edx>;

/// Leaf 12H
#[derive(Debug, PartialEq, Eq)]
pub struct Leaf12<'a>(
    pub Option<&'a Leaf12Subleaf0>,
    pub Option<&'a Leaf12Subleaf1>,
    pub Vec<&'a Leaf12SubleafGt1>,
);
/// Leaf 12H
#[derive(Debug, PartialEq, Eq)]
pub struct Leaf12Mut<'a>(
    pub Option<&'a mut Leaf12Subleaf0>,
    pub Option<&'a mut Leaf12Subleaf1>,
    pub Vec<&'a mut Leaf12SubleafGt1>,
);
/// Leaf 12H subleaf 0
pub type Leaf12Subleaf0 =
    Leaf<Leaf12Subleaf0Eax, Leaf12Subleaf0Ebx, Leaf12Subleaf0Ecx, Leaf12Subleaf0Edx>;
/// Leaf 12H subleaf 1
pub type Leaf12Subleaf1 =
    Leaf<Leaf12Subleaf1Eax, Leaf12Subleaf1Ebx, Leaf12Subleaf1Ecx, Leaf12Subleaf1Edx>;
/// Leaf 12H subleaf >1
pub type Leaf12SubleafGt1 =
    Leaf<Leaf12SubleafGt1Eax, Leaf12SubleafGt1Ebx, Leaf12SubleafGt1Ecx, Leaf12SubleafGt1Edx>;

/// Leaf 14H
#[derive(Debug, PartialEq, Eq)]
pub struct Leaf14<'a>(
    pub Option<&'a Leaf14Subleaf0>,
    pub Option<&'a Leaf14Subleaf1>,
);
/// Leaf 14H
#[derive(Debug, PartialEq, Eq)]
pub struct Leaf14Mut<'a>(
    pub Option<&'a mut Leaf14Subleaf0>,
    pub Option<&'a mut Leaf14Subleaf1>,
);
/// Leaf 14H subleaf 0
pub type Leaf14Subleaf0 =
    Leaf<Leaf14Subleaf0Eax, Leaf14Subleaf0Ebx, Leaf14Subleaf0Ecx, Leaf14Subleaf0Edx>;
/// Leaf 14H subleaf 1
pub type Leaf14Subleaf1 =
    Leaf<Leaf14Subleaf1Eax, Leaf14Subleaf1Ebx, Leaf14Subleaf1Ecx, Leaf14Subleaf1Edx>;

/// Leaf 15H
pub type Leaf15 = Leaf<Leaf15Eax, Leaf15Ebx, Leaf15Ecx, Leaf15Edx>;

/// Leaf 16H
pub type Leaf16 = Leaf<Leaf16Eax, Leaf16Ebx, Leaf16Ecx, Leaf16Edx>;

/// Leaf 17H
#[derive(Debug, PartialEq, Eq)]
pub struct Leaf17<'a>(
    pub Option<&'a Leaf17Subleaf0>,
    pub Option<&'a Leaf17Subleaf1>,
    pub Option<&'a Leaf17Subleaf2>,
    pub Option<&'a Leaf17Subleaf3>,
);
/// Leaf 17H
#[derive(Debug, PartialEq, Eq)]
pub struct Leaf17Mut<'a>(
    pub Option<&'a mut Leaf17Subleaf0>,
    pub Option<&'a mut Leaf17Subleaf1>,
    pub Option<&'a mut Leaf17Subleaf2>,
    pub Option<&'a mut Leaf17Subleaf3>,
);
/// Leaf 18H subleaf 0
pub type Leaf17Subleaf0 =
    Leaf<Leaf17Subleaf0Eax, Leaf17Subleaf0Ebx, Leaf17Subleaf0Ecx, Leaf17Subleaf0Edx>;
/// Leaf 17H subleaf 1
pub type Leaf17Subleaf1 =
    Leaf<Leaf17Subleaf1Eax, Leaf17Subleaf1Ebx, Leaf17Subleaf1Ecx, Leaf17Subleaf1Edx>;
/// Leaf 17H subleaf 2
pub type Leaf17Subleaf2 = Leaf17Subleaf1;
/// Leaf 17H subleaf 3
pub type Leaf17Subleaf3 = Leaf17Subleaf1;

/// Leaf 18H
#[derive(Debug, PartialEq, Eq)]
pub struct Leaf18<'a>(
    pub Option<&'a Leaf18Subleaf0>,
    pub Vec<&'a Leaf18SubleafGt0>,
);
/// Leaf 18H
#[derive(Debug, PartialEq, Eq)]
pub struct Leaf18Mut<'a>(
    pub Option<&'a mut Leaf18Subleaf0>,
    pub Vec<&'a mut Leaf18SubleafGt0>,
);
/// Leaf 18H subleaf 0
pub type Leaf18Subleaf0 =
    Leaf<Leaf18Subleaf0Eax, Leaf18Subleaf0Ebx, Leaf18Subleaf0Ecx, Leaf18Subleaf0Edx>;
/// Leaf 18H subleaf 1
pub type Leaf18SubleafGt0 =
    Leaf<Leaf18SubleafGt0Eax, Leaf18SubleafGt0Ebx, Leaf18SubleafGt0Ecx, Leaf18SubleafGt0Edx>;

/// Leaf 19H
pub type Leaf19 = Leaf<Leaf19Eax, Leaf19Ebx, Leaf19Ecx, Leaf19Edx>;

/// Leaf 1AH
pub type Leaf1A = Leaf<Leaf1AEax, Leaf1AEbx, Leaf1AEcx, Leaf1AEdx>;

/// Leaf 1CH
pub type Leaf1C = Leaf<Leaf1CEax, Leaf1CEbx, Leaf1CEcx, Leaf1CEdx>;

/// Leaf 1FH
#[derive(Debug, PartialEq, Eq)]
pub struct Leaf1F<'a>(pub Vec<&'a Leaf1FSubleaf>);
/// Leaf 1FH
#[derive(Debug, PartialEq, Eq)]
pub struct Leaf1FMut<'a>(pub Vec<&'a mut Leaf1FSubleaf>);
/// Leaf 1F subleaf 1
pub type Leaf1FSubleaf = Leaf<Leaf1FEax, Leaf1FEbx, Leaf1FEcx, Leaf1FEdx>;

/// Leaf 20H
pub type Leaf20 = Leaf<Leaf20Eax, Leaf20Ebx, Leaf20Ecx, Leaf20Edx>;

/// Leaf 80000000H
pub type Leaf80000000 = Leaf<Leaf80000000Eax, Leaf80000000Ebx, Leaf80000000Ecx, Leaf80000000Edx>;

/// Leaf 80000001H
pub type Leaf80000001 = Leaf<Leaf80000001Eax, Leaf80000001Ebx, Leaf80000001Ecx, Leaf80000001Edx>;

/// Leaf 80000005H
pub type Leaf80000005 = Leaf<Leaf80000005Eax, Leaf80000005Ebx, Leaf80000005Ecx, Leaf80000005Edx>;

/// Leaf 80000006H
pub type Leaf80000006 = Leaf<Leaf80000006Eax, Leaf80000006Ebx, Leaf80000006Ecx, Leaf80000006Edx>;

/// Leaf 80000007H
pub type Leaf80000007 = Leaf<Leaf80000007Eax, Leaf80000007Ebx, Leaf80000007Ecx, Leaf80000007Edx>;

/// Leaf 80000008H
pub type Leaf80000008 = Leaf<Leaf80000008Eax, Leaf80000008Ebx, Leaf80000008Ecx, Leaf80000008Edx>;

// -------------------------------------------------------------------------------------------------
// Equal
// -------------------------------------------------------------------------------------------------

impl Equal for Leaf4<'_> {
    /// Compares `self` to `other` ignoring undefined bits.
    #[inline]
    #[must_use]
    fn equal(&self, other: &Self) -> bool {
        self.0.equal(&other.0)
    }
}
impl Equal for Leaf4Mut<'_> {
    /// Compares `self` to `other` ignoring undefined bits.
    #[inline]
    #[must_use]
    fn equal(&self, other: &Self) -> bool {
        self.0.equal(&other.0)
    }
}
impl Equal for Leaf7<'_> {
    /// Compares `self` to `other` ignoring undefined bits.
    #[inline]
    #[must_use]
    fn equal(&self, other: &Self) -> bool {
        self.0.equal(&other.0) && self.1.equal(&other.1)
    }
}
impl Equal for Leaf7Mut<'_> {
    /// Compares `self` to `other` ignoring undefined bits.
    #[inline]
    #[must_use]
    fn equal(&self, other: &Self) -> bool {
        self.0.equal(&other.0) && self.1.equal(&other.1)
    }
}
impl Equal for LeafB<'_> {
    /// Compares `self` to `other` ignoring undefined bits.
    #[inline]
    #[must_use]
    fn equal(&self, other: &Self) -> bool {
        self.0.equal(&other.0)
    }
}
impl Equal for LeafBMut<'_> {
    /// Compares `self` to `other` ignoring undefined bits.
    #[inline]
    #[must_use]
    fn equal(&self, other: &Self) -> bool {
        self.0.equal(&other.0)
    }
}

impl Equal for LeafF<'_> {
    /// Compares `self` to `other` ignoring undefined bits.
    #[inline]
    #[must_use]
    fn equal(&self, other: &Self) -> bool {
        self.0.equal(&other.0) && self.1.equal(&other.1)
    }
}
impl Equal for LeafFMut<'_> {
    /// Compares `self` to `other` ignoring undefined bits.
    #[inline]
    #[must_use]
    fn equal(&self, other: &Self) -> bool {
        self.0.equal(&other.0) && self.1.equal(&other.1)
    }
}

impl Equal for Leaf10<'_> {
    /// Compares `self` to `other` ignoring undefined bits.
    #[inline]
    #[must_use]
    fn equal(&self, other: &Self) -> bool {
        self.0.equal(&other.0)
            && self.1.equal(&other.1)
            && self.2.equal(&other.2)
            && self.3.equal(&other.3)
    }
}
impl Equal for Leaf10Mut<'_> {
    /// Compares `self` to `other` ignoring undefined bits.
    #[inline]
    #[must_use]
    fn equal(&self, other: &Self) -> bool {
        self.0.equal(&other.0)
            && self.1.equal(&other.1)
            && self.2.equal(&other.2)
            && self.3.equal(&other.3)
    }
}

impl Equal for Leaf12<'_> {
    /// Compares `self` to `other` ignoring undefined bits.
    #[inline]
    #[must_use]
    fn equal(&self, other: &Self) -> bool {
        self.0.equal(&other.0) && self.1.equal(&other.1) && self.2.equal(&other.2)
    }
}
impl Equal for Leaf12Mut<'_> {
    /// Compares `self` to `other` ignoring undefined bits.
    #[inline]
    #[must_use]
    fn equal(&self, other: &Self) -> bool {
        self.0.equal(&other.0) && self.1.equal(&other.1) && self.2.equal(&other.2)
    }
}
impl Equal for Leaf14<'_> {
    /// Compares `self` to `other` ignoring undefined bits.
    #[inline]
    #[must_use]
    fn equal(&self, other: &Self) -> bool {
        self.0.equal(&other.0) && self.1.equal(&other.1)
    }
}
impl Equal for Leaf14Mut<'_> {
    /// Compares `self` to `other` ignoring undefined bits.
    #[inline]
    #[must_use]
    fn equal(&self, other: &Self) -> bool {
        self.0.equal(&other.0) && self.1.equal(&other.1)
    }
}
impl Equal for Leaf17<'_> {
    /// Compares `self` to `other` ignoring undefined bits.
    #[inline]
    #[must_use]
    fn equal(&self, other: &Self) -> bool {
        self.0.equal(&other.0)
            && self.1.equal(&other.1)
            && self.2.equal(&other.2)
            && self.3.equal(&other.3)
    }
}
impl Equal for Leaf17Mut<'_> {
    /// Compares `self` to `other` ignoring undefined bits.
    #[inline]
    #[must_use]
    fn equal(&self, other: &Self) -> bool {
        self.0.equal(&other.0)
            && self.1.equal(&other.1)
            && self.2.equal(&other.2)
            && self.3.equal(&other.3)
    }
}

impl Equal for Leaf18<'_> {
    /// Compares `self` to `other` ignoring undefined bits.
    #[inline]
    #[must_use]
    fn equal(&self, other: &Self) -> bool {
        self.0.equal(&other.0) && self.1.equal(&other.1)
    }
}
impl Equal for Leaf18Mut<'_> {
    /// Compares `self` to `other` ignoring undefined bits.
    #[inline]
    #[must_use]
    fn equal(&self, other: &Self) -> bool {
        self.0.equal(&other.0) && self.1.equal(&other.1)
    }
}

impl Equal for Leaf1F<'_> {
    /// Compares `self` to `other` ignoring undefined bits.
    #[inline]
    #[must_use]
    fn equal(&self, other: &Self) -> bool {
        self.0.equal(&other.0)
    }
}
impl Equal for Leaf1FMut<'_> {
    /// Compares `self` to `other` ignoring undefined bits.
    #[inline]
    #[must_use]
    fn equal(&self, other: &Self) -> bool {
        self.0.equal(&other.0)
    }
}

// -------------------------------------------------------------------------------------------------
// Supports
// -------------------------------------------------------------------------------------------------

/// Error type for [`<Leaf5 as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Leaf5NotSupported {
    /// SmallestMonitorLineSize.
    #[error("SmallestMonitorLineSize.")]
    SmallestMonitorLineSize,
    /// LargestMonitorLineSize.
    #[error("LargestMonitorLineSize.")]
    LargestMonitorLineSize,
    /// Ecx.
    #[error("Ecx.")]
    Ecx,
}

impl Supports for Leaf5 {
    type Error = Leaf5NotSupported;
    /// We check everything here.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        warn_support!("0x5", true, true, true, false);
        // We compare `<=` therefore `Ordering::Less` corresponds  greater support and to
        // `Ordering::Greater` for support, thus we reverse the result of the comparison.
        if self.eax.smallest_monitor_line_size() > other.eax.smallest_monitor_line_size() {
            return Err(Leaf5NotSupported::SmallestMonitorLineSize);
        }
        if self.ebx.largest_monitor_line_size() < other.ebx.largest_monitor_line_size() {
            return Err(Leaf5NotSupported::LargestMonitorLineSize);
        }
        if matches!(self.ecx.cmp_flags(&other.ecx), Some(Ordering::Less) | None) {
            return Err(Leaf5NotSupported::Ecx);
        }

        Ok(())
    }
}

/// Error type for [`<Leaf6 as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Leaf6NotSupported {
    /// Eax.
    #[error("Eax.")]
    Eax,
    /// NumberOfInterruptThresholdsInDigitalThermalSensor.
    #[error("NumberOfInterruptThresholdsInDigitalThermalSensor.")]
    NumberOfInterruptThresholdsInDigitalThermalSensor,
    /// IntelThreadDirectorClasses.
    #[error("IntelThreadDirectorClasses.")]
    IntelThreadDirectorClasses,
    /// Ecx.
    #[error("Ecx.")]
    Ecx,
}

impl Supports for Leaf6 {
    type Error = Leaf6NotSupported;
    /// We do not currently check EDX.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        warn_support!("0x6", true, true, true, false);
        match self.eax.cmp_flags(&other.eax) {
            Some(Ordering::Greater | Ordering::Equal) => (),
            Some(Ordering::Less) | None => {
                return Err(Leaf6NotSupported::Eax);
            }
        }
        if self
            .ebx
            .number_of_interrupt_thresholds_in_digital_thermal_sensor()
            < other
                .ebx
                .number_of_interrupt_thresholds_in_digital_thermal_sensor()
        {
            return Err(Leaf6NotSupported::NumberOfInterruptThresholdsInDigitalThermalSensor);
        }
        if self.ecx.intel_thread_director_classes() < other.ecx.intel_thread_director_classes() {
            return Err(Leaf6NotSupported::IntelThreadDirectorClasses);
        }
        if matches!(self.ecx.cmp_flags(&other.ecx), Some(Ordering::Less) | None) {
            return Err(Leaf6NotSupported::Ecx);
        }

        Ok(())
    }
}

/// Error type for [`<Leaf7 as Supports>::supports`] and [`<Leaf7Mut as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Leaf7NotSupported {
    /// MissingSubleaf0.
    #[error("MissingSubleaf0.")]
    MissingSubleaf0,
    /// Subleaf0.
    #[error("Subleaf0: {0}")]
    Subleaf0(Leaf7Subleaf0NotSupported),
    /// MissingSubleaf1.
    #[error("MissingSubleaf1.")]
    MissingSubleaf1,
    /// Subleaf1.
    #[error("Subleaf1: {0}")]
    Subleaf1(Leaf7Subleaf1NotSupported),
}

impl Supports for Leaf7<'_> {
    type Error = Leaf7NotSupported;
    /// We do not currently check EDX.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        match (self.0, other.0) {
            (_, None) => (),
            (None, Some(_)) => return Err(Leaf7NotSupported::MissingSubleaf0),
            (Some(a), Some(b)) => a.supports(b).map_err(Leaf7NotSupported::Subleaf0)?,
        }
        match (self.1, other.1) {
            (_, None) => (),
            (None, Some(_)) => return Err(Leaf7NotSupported::MissingSubleaf1),
            (Some(a), Some(b)) => a.supports(b).map_err(Leaf7NotSupported::Subleaf1)?,
        }

        Ok(())
    }
}
impl Supports for Leaf7Mut<'_> {
    type Error = Leaf7NotSupported;
    /// We do not currently check EDX.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        match (self.0.as_ref(), other.0.as_ref()) {
            (_, None) => (),
            (None, Some(_)) => return Err(Leaf7NotSupported::MissingSubleaf0),
            (Some(a), Some(b)) => a.supports(b).map_err(Leaf7NotSupported::Subleaf0)?,
        }
        match (self.1.as_ref(), other.1.as_ref()) {
            (_, None) => (),
            (None, Some(_)) => return Err(Leaf7NotSupported::MissingSubleaf1),
            (Some(a), Some(b)) => a.supports(b).map_err(Leaf7NotSupported::Subleaf1)?,
        }

        Ok(())
    }
}

/// Error type for [`<Leaf7Subleaf0 as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Leaf7Subleaf0NotSupported {
    /// MaxInputValueSubleaf.
    #[error("MaxInputValueSubleaf: {0} vs {1}.")]
    MaxInputValueSubleaf(u32, u32),
    /// Ebx.
    #[error("Ebx: {0} vs {1}.")]
    Ebx(u32, u32),
    /// Ecx.
    #[error("Ecx: {0} vs {1}.")]
    Ecx(u32, u32),
    /// Edx.
    #[error("Edx: {0} vs {1}.")]
    Edx(u32, u32),
}

impl Supports for Leaf7Subleaf0 {
    type Error = Leaf7Subleaf0NotSupported;
    /// We check everything here.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        debug_assert!(
            self.eax.max_input_value_subleaf() == 1 || self.eax.max_input_value_subleaf() == 0
        );
        debug_assert!(
            other.eax.max_input_value_subleaf() == 1 || other.eax.max_input_value_subleaf() == 0
        );
        warn_support!("0x7 sub-leaf 0", true, true, true, true);

        if self.eax.max_input_value_subleaf() < other.eax.max_input_value_subleaf() {
            return Err(Leaf7Subleaf0NotSupported::MaxInputValueSubleaf(
                self.eax.max_input_value_subleaf().read(),
                other.eax.max_input_value_subleaf().read(),
            ));
        }
        if matches!(self.ebx.cmp_flags(&other.ebx), Some(Ordering::Less) | None) {
            return Err(Leaf7Subleaf0NotSupported::Ebx(self.ebx.0, other.ebx.0));
        }

        // KVM automtically sets OSPKE as active, but will return that it is not supported,
        // therefore we mask it out when comparing KMV CPUID support.
        let mask = !Leaf7Subleaf0Ecx::OSPKE;
        if matches!(
            (self.ecx & mask).cmp_flags(&(other.ecx & mask)),
            Some(Ordering::Less) | None
        ) {
            return Err(Leaf7Subleaf0NotSupported::Ecx(self.ecx.0, other.ecx.0));
        }
        if matches!(self.edx.cmp_flags(&other.edx), Some(Ordering::Less) | None) {
            return Err(Leaf7Subleaf0NotSupported::Edx(self.edx.0, other.edx.0));
        }

        Ok(())
    }
}

/// Error type for [`<Leaf7Subleaf1 as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Leaf7Subleaf1NotSupported {
    /// Eax.
    #[error("Eax.")]
    Eax,
    /// Ebx.
    #[error("Ebx.")]
    Ebx,
}

impl Supports for Leaf7Subleaf1 {
    type Error = Leaf7Subleaf1NotSupported;
    /// We check everything here.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        warn_support!("0x7 sub-leaf 1", true, true, true, true);
        if matches!(self.eax.cmp_flags(&other.eax), Some(Ordering::Less) | None) {
            return Err(Leaf7Subleaf1NotSupported::Eax);
        }
        if matches!(self.ebx.cmp_flags(&other.ebx), Some(Ordering::Less) | None) {
            return Err(Leaf7Subleaf1NotSupported::Ebx);
        }
        Ok(())
    }
}

/// Error type for [`<LeafA as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum LeafANotSupported {
    /// Ebx.
    #[error("Ebx.")]
    Ebx,
}

impl Supports for LeafA {
    type Error = LeafANotSupported;
    /// We do not currently check EAX, ECX and EDX.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        warn_support!("0xA", false, true, false, false);
        if matches!(self.ebx.cmp_flags(&other.ebx), Some(Ordering::Less) | None) {
            return Err(LeafANotSupported::Ebx);
        }
        Ok(())
    }
}

/// Error type for [`<LeafF as Supports>::supports`] and [`<LeafFMut as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum LeafFNotSupported {
    /// MissingSubleaf0.
    #[error("MissingSubleaf0.")]
    MissingSubleaf0,
    /// Subleaf0.
    #[error("Subleaf0: {0}")]
    Subleaf0(LeafFSubleaf0NotSupported),
    /// MissingSubleaf1.
    #[error("MissingSubleaf1.")]
    MissingSubleaf1,
    /// Subleaf1.
    #[error("Subleaf1: {0}")]
    Subleaf1(LeafFSubleaf1NotSupported),
}

impl Supports for LeafF<'_> {
    type Error = LeafFNotSupported;
    /// We check sub-leaves 0 and 1.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        match (self.0, other.0) {
            (_, None) => (),
            (None, Some(_)) => return Err(LeafFNotSupported::MissingSubleaf0),
            (Some(a), Some(b)) => a.supports(b).map_err(LeafFNotSupported::Subleaf0)?,
        }
        match (self.1, other.1) {
            (_, None) => (),
            (None, Some(_)) => return Err(LeafFNotSupported::MissingSubleaf1),
            (Some(a), Some(b)) => a.supports(b).map_err(LeafFNotSupported::Subleaf1)?,
        }
        Ok(())
    }
}

impl Supports for LeafFMut<'_> {
    type Error = LeafFNotSupported;
    /// We check sub-leaves 0 and 1.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        match (self.0.as_ref(), other.0.as_ref()) {
            (_, None) => (),
            (None, Some(_)) => return Err(LeafFNotSupported::MissingSubleaf0),
            (Some(a), Some(b)) => a.supports(b).map_err(LeafFNotSupported::Subleaf0)?,
        }
        match (self.1.as_ref(), other.1.as_ref()) {
            (_, None) => (),
            (None, Some(_)) => return Err(LeafFNotSupported::MissingSubleaf1),
            (Some(a), Some(b)) => a.supports(b).map_err(LeafFNotSupported::Subleaf1)?,
        }
        Ok(())
    }
}

/// Error type for [`<LeafFSubleaf0 as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum LeafFSubleaf0NotSupported {
    /// MaxRmidRange.
    #[error("MaxRmidRange.")]
    MaxRmidRange,
    /// Edx.
    #[error("Edx.")]
    Edx,
}

impl Supports for LeafFSubleaf0 {
    type Error = LeafFSubleaf0NotSupported;
    /// We check everything here.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        warn_support!("0xF sub-leaf 0", true, true, true, true);
        if self.ebx.max_rmid_range() < other.ebx.max_rmid_range() {
            return Err(LeafFSubleaf0NotSupported::MaxRmidRange);
        }
        if matches!(self.edx.cmp_flags(&other.edx), Some(Ordering::Less) | None) {
            return Err(LeafFSubleaf0NotSupported::Edx);
        }

        Ok(())
    }
}

/// Error type for [`<LeafFSubleaf1 as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum LeafFSubleaf1NotSupported {
    /// RmidMax.
    #[error("RmidMax.")]
    RmidMax,
    /// Edx.
    #[error("Edx.")]
    Edx,
}

impl Supports for LeafFSubleaf1 {
    type Error = LeafFSubleaf1NotSupported;
    /// We do not check EBX.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        warn_support!("0xF sub-leaf 1", true, false, true, true);
        if self.ecx.rmid_max() < other.ecx.rmid_max() {
            return Err(LeafFSubleaf1NotSupported::RmidMax);
        }
        if matches!(self.edx.cmp_flags(&other.edx), Some(Ordering::Less) | None) {
            return Err(LeafFSubleaf1NotSupported::Edx);
        }

        Ok(())
    }
}

/// Error type for [`<Leaf10 as Supports>::supports`] and [`<Leaf10Mut as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Leaf10NotSupported {
    /// MissingSubleaf0.
    #[error("MissingSubleaf0.")]
    MissingSubleaf0,
    /// Subleaf0.
    #[error("Subleaf0: {0}")]
    Subleaf0(Leaf10Subleaf0NotSupported),
    /// MissingSubleaf1.
    #[error("MissingSubleaf1.")]
    MissingSubleaf1,
    /// Subleaf1.
    #[error("Subleaf1: {0}")]
    Subleaf1(Leaf10Subleaf1NotSupported),
    /// MissingSubleaf3.
    #[error("MissingSubleaf3.")]
    MissingSubleaf3,
    /// Subleaf3.
    #[error("Subleaf3: {0}")]
    Subleaf3(Leaf10Subleaf3NotSupported),
}

impl Supports for Leaf10<'_> {
    type Error = Leaf10NotSupported;
    /// We check sub-leaves 0 and 1.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        log::warn!(
            "Could not fully validate support for Intel CPUID leaf 0x10 due to being unable to \
             validate sub-leaf 2."
        );
        match (self.0, other.0) {
            (_, None) => (),
            (None, Some(_)) => return Err(Leaf10NotSupported::MissingSubleaf0),
            (Some(a), Some(b)) => a.supports(b).map_err(Leaf10NotSupported::Subleaf0)?,
        }
        match (self.1, other.1) {
            (_, None) => (),
            (None, Some(_)) => return Err(Leaf10NotSupported::MissingSubleaf1),
            (Some(a), Some(b)) => a.supports(b).map_err(Leaf10NotSupported::Subleaf1)?,
        }
        match (self.3, other.3) {
            (_, None) => (),
            (None, Some(_)) => return Err(Leaf10NotSupported::MissingSubleaf3),
            (Some(a), Some(b)) => a.supports(b).map_err(Leaf10NotSupported::Subleaf3)?,
        }
        Ok(())
    }
}

impl Supports for Leaf10Mut<'_> {
    type Error = Leaf10NotSupported;
    /// We check sub-leaves 0 and 1.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        log::warn!(
            "Could not fully validate support for Intel CPUID leaf 0x10 due to being unable to \
             validate sub-leaf 2."
        );
        match (self.0.as_ref(), other.0.as_ref()) {
            (_, None) => (),
            (None, Some(_)) => return Err(Leaf10NotSupported::MissingSubleaf0),
            (Some(a), Some(b)) => a.supports(b).map_err(Leaf10NotSupported::Subleaf0)?,
        }
        match (self.1.as_ref(), other.1.as_ref()) {
            (_, None) => (),
            (None, Some(_)) => return Err(Leaf10NotSupported::MissingSubleaf1),
            (Some(a), Some(b)) => a.supports(b).map_err(Leaf10NotSupported::Subleaf1)?,
        }
        match (self.3.as_ref(), other.3.as_ref()) {
            (_, None) => (),
            (None, Some(_)) => return Err(Leaf10NotSupported::MissingSubleaf3),
            (Some(a), Some(b)) => a.supports(b).map_err(Leaf10NotSupported::Subleaf3)?,
        }
        Ok(())
    }
}

/// Error type for [`<Leaf10Subleaf0 as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Leaf10Subleaf0NotSupported {
    /// Ebx.
    #[error("Ebx.")]
    Ebx,
}

impl Supports for Leaf10Subleaf0 {
    type Error = Leaf10Subleaf0NotSupported;
    /// We check everything here.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        warn_support!("0x10 sub-leaf 0", true, true, true, true);
        if matches!(self.ebx.cmp_flags(&other.ebx), Some(Ordering::Less) | None) {
            return Err(Leaf10Subleaf0NotSupported::Ebx);
        }
        Ok(())
    }
}

/// Error type for [`<Leaf10Subleaf1 as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Leaf10Subleaf1NotSupported {
    /// Ecx.
    #[error("Ecx.")]
    Ecx,
}

impl Supports for Leaf10Subleaf1 {
    type Error = Leaf10Subleaf1NotSupported;
    /// We only check ECX.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        warn_support!("0x10 sub-leaf 1", false, false, true, false);
        if matches!(self.ecx.cmp_flags(&other.ecx), Some(Ordering::Less) | None) {
            return Err(Leaf10Subleaf1NotSupported::Ecx);
        }
        Ok(())
    }
}

/// Error type for [`<Leaf10Subleaf3 as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Leaf10Subleaf3NotSupported {
    /// Ecx.
    #[error("Ecx.")]
    Ecx,
}

impl Supports for Leaf10Subleaf3 {
    type Error = Leaf10Subleaf3NotSupported;
    /// We only check ECX.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        warn_support!("0x10 sub-leaf 3", false, false, true, false);
        if matches!(self.ecx.cmp_flags(&other.ecx), Some(Ordering::Less) | None) {
            return Err(Leaf10Subleaf3NotSupported::Ecx);
        }
        Ok(())
    }
}

/// Error type for [`<Leaf14 as Supports>::supports`] and [`<Leaf14Mut as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Leaf14NotSupported {
    /// MissingSubleaf0.
    #[error("MissingSubleaf0.")]
    MissingSubleaf0,
    /// Subleaf0.
    #[error("Subleaf0: {0}")]
    Subleaf0(Leaf14Subleaf0NotSupported),
}

impl Supports for Leaf14<'_> {
    type Error = Leaf14NotSupported;
    /// Only checks subleaf 1.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        log::warn!(
            "Could not fully validate support for Intel CPUID leaf 0x14 due to being unable to \
             validate sub-leaf 1."
        );
        match (self.0, other.0) {
            (_, None) => (),
            (None, Some(_)) => return Err(Leaf14NotSupported::MissingSubleaf0),
            (Some(a), Some(b)) => a.supports(b).map_err(Leaf14NotSupported::Subleaf0)?,
        }
        Ok(())
    }
}

impl Supports for Leaf14Mut<'_> {
    type Error = Leaf14NotSupported;
    /// Only checks subleaf 1.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        log::warn!(
            "Could not fully validate support for Intel CPUID leaf 0x14 due to being unable to \
             validate sub-leaf 1."
        );
        match (self.0.as_ref(), other.0.as_ref()) {
            (_, None) => (),
            (None, Some(_)) => return Err(Leaf14NotSupported::MissingSubleaf0),
            (Some(a), Some(b)) => a.supports(b).map_err(Leaf14NotSupported::Subleaf0)?,
        }
        Ok(())
    }
}

/// Error type for [`<Leaf14Subleaf0 as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Leaf14Subleaf0NotSupported {
    /// MaxSubleaf.
    #[error("MaxSubleaf.")]
    MaxSubleaf,
    /// Ebx.
    #[error("Ebx.")]
    Ebx,
    /// Ecx.
    #[error("Ecx.")]
    Ecx,
}

impl Supports for Leaf14Subleaf0 {
    type Error = Leaf14Subleaf0NotSupported;
    /// We check everything here.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        warn_support!("0x14 sub-leaf 0", true, true, true, true);
        if self.eax.max_subleaf() < other.eax.max_subleaf() {
            return Err(Leaf14Subleaf0NotSupported::MaxSubleaf);
        }
        if matches!(self.ebx.cmp_flags(&other.ebx), Some(Ordering::Less) | None) {
            return Err(Leaf14Subleaf0NotSupported::Ebx);
        }
        if matches!(self.ecx.cmp_flags(&other.ecx), Some(Ordering::Less) | None) {
            return Err(Leaf14Subleaf0NotSupported::Ebx);
        }
        Ok(())
    }
}

/// Error type for [`<Leaf18 as Supports>::supports`] and [`<Leaf18Mut as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Leaf18NotSupported {
    /// MissingSubleaf0.
    #[error("MissingSubleaf0.")]
    MissingSubleaf0,
    /// Subleaf0.
    #[error("Subleaf0: {0}")]
    Subleaf0(Leaf18Subleaf0NotSupported),
}

impl Supports for Leaf18<'_> {
    type Error = Leaf18NotSupported;
    /// Only checks subleaf 1.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        log::warn!(
            "Could not fully validate support for Intel CPUID leaf 0x18 due to being unable to \
             validate sub-leaf 1."
        );
        match (self.0.as_ref(), other.0.as_ref()) {
            (_, None) => (),
            (None, Some(_)) => return Err(Leaf18NotSupported::MissingSubleaf0),
            (Some(a), Some(b)) => a.supports(b).map_err(Leaf18NotSupported::Subleaf0)?,
        }
        Ok(())
    }
}

impl Supports for Leaf18Mut<'_> {
    type Error = Leaf18NotSupported;
    /// Only checks subleaf 1.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        log::warn!(
            "Could not fully validate support for Intel CPUID leaf 0x18 due to being unable to \
             validate sub-leaf 1."
        );
        match (self.0.as_ref(), other.0.as_ref()) {
            (_, None) => (),
            (None, Some(_)) => return Err(Leaf18NotSupported::MissingSubleaf0),
            (Some(a), Some(b)) => a.supports(b).map_err(Leaf18NotSupported::Subleaf0)?,
        }
        Ok(())
    }
}

/// Error type for [`<Leaf18Subleaf0 as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Leaf18Subleaf0NotSupported {
    /// MissingSubleaf0.
    #[error("MissingSubleaf0.")]
    MaxSubleaf,
    /// Ebx.
    #[error("Ebx.")]
    Ebx,
}

impl Supports for Leaf18Subleaf0 {
    type Error = Leaf18Subleaf0NotSupported;
    /// We do not check ECX or EDX.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        warn_support!("0x18 sub-leaf 0", true, true, false, false);
        if self.eax.max_subleaf() < other.eax.max_subleaf() {
            return Err(Leaf18Subleaf0NotSupported::MaxSubleaf);
        }
        if matches!(self.ebx.cmp_flags(&other.ebx), Some(Ordering::Less) | None) {
            return Err(Leaf18Subleaf0NotSupported::Ebx);
        }
        Ok(())
    }
}

/// Error type for [`<Leaf19 as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Leaf19NotSupported {
    /// Eax.
    #[error("Eax.")]
    Eax,
    /// Ebx.
    #[error("Ebx.")]
    Ebx,
    /// Ecx.
    #[error("Ecx.")]
    Ecx,
}

impl Supports for Leaf19 {
    type Error = Leaf19NotSupported;
    /// We check everything here.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        warn_support!("0x19", true, true, true, true);
        if matches!(self.eax.cmp_flags(&other.eax), Some(Ordering::Less) | None) {
            return Err(Leaf19NotSupported::Eax);
        }
        if matches!(self.ebx.cmp_flags(&other.ebx), Some(Ordering::Less) | None) {
            return Err(Leaf19NotSupported::Ebx);
        }
        if matches!(self.ecx.cmp_flags(&other.ecx), Some(Ordering::Less) | None) {
            return Err(Leaf19NotSupported::Ecx);
        }
        Ok(())
    }
}

/// Error type for [`<Leaf1C as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Leaf1CNotSupported {
    /// Eax.
    #[error("Eax.")]
    Eax,
    /// Ebx.
    #[error("Ebx.")]
    Ebx,
    /// Ecx.
    #[error("Ecx.")]
    Ecx,
}

impl Supports for Leaf1C {
    type Error = Leaf1CNotSupported;
    /// We do not check EAX.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        warn_support!("0x1C", true, true, true, true);
        if matches!(self.eax.cmp_flags(&other.eax), Some(Ordering::Less) | None) {
            return Err(Leaf1CNotSupported::Eax);
        }
        if matches!(self.ebx.cmp_flags(&other.ebx), Some(Ordering::Less) | None) {
            return Err(Leaf1CNotSupported::Ebx);
        }
        if matches!(self.ecx.cmp_flags(&other.ecx), Some(Ordering::Less) | None) {
            return Err(Leaf1CNotSupported::Ecx);
        }
        Ok(())
    }
}

/// Error type for [`<Leaf20 as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Leaf20NotSupported {
    /// MaxSubleaves.
    #[error("MaxSubleaves.")]
    MaxSubleaves,
    /// Ebx.
    #[error("Ebx.")]
    Ebx,
}

impl Supports for Leaf20 {
    type Error = Leaf20NotSupported;
    /// We do not check EBX.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        debug_assert_eq!(self.eax.max_subleaves(), 1);
        debug_assert_eq!(other.eax.max_subleaves(), 1);
        warn_support!("0x1C", true, true, true, true);

        if self.eax.max_subleaves() < other.eax.max_subleaves() {
            return Err(Leaf20NotSupported::MaxSubleaves);
        }
        if matches!(self.ebx.cmp_flags(&other.ebx), Some(Ordering::Less) | None) {
            return Err(Leaf20NotSupported::Ebx);
        }
        Ok(())
    }
}

/// Error type for [`<Leaf80000000 as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Leaf80000000NotSupported {
    /// MaxExtendedFunctionInput.
    #[error("MaxExtendedFunctionInput.")]
    MaxExtendedFunctionInput,
}

impl Supports for Leaf80000000 {
    type Error = Leaf80000000NotSupported;
    /// We check everything here.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        warn_support!("0x80000000", true, true, true, true);

        if self.eax.max_extend_function_input() < other.eax.max_extend_function_input() {
            return Err(Leaf80000000NotSupported::MaxExtendedFunctionInput);
        }
        Ok(())
    }
}

/// Error type for [`<Leaf80000001 as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Leaf80000001NotSupported {
    /// Ecx.
    #[error("Ecx.")]
    Ecx,
    /// Edx.
    #[error("Edx.")]
    Edx,
}

impl Supports for Leaf80000001 {
    type Error = Leaf80000001NotSupported;
    /// We do not check EAX.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        warn_support!("0x80000001", true, true, true, true);

        if matches!(self.ecx.cmp_flags(&other.ecx), Some(Ordering::Less) | None) {
            return Err(Leaf80000001NotSupported::Ecx);
        }
        if matches!(self.edx.cmp_flags(&other.edx), Some(Ordering::Less) | None) {
            return Err(Leaf80000001NotSupported::Edx);
        }
        Ok(())
    }
}

/// Error type for [`<Leaf80000007 as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Leaf80000007NotSupported {
    /// Edx.
    #[error("Edx.")]
    Edx,
}

impl Supports for Leaf80000007 {
    type Error = Leaf80000007NotSupported;
    /// We check everything here.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        warn_support!("0x80000007", true, true, true, true);

        if matches!(self.edx.cmp_flags(&other.edx), Some(Ordering::Less) | None) {
            return Err(Leaf80000007NotSupported::Edx);
        }
        Ok(())
    }
}

/// Error type for [`<Leaf80000008 as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Leaf80000008NotSupported {
    /// PhysicalAddressBits.
    #[error("PhysicalAddressBits.")]
    PhysicalAddressBits,
    /// LinearAddressBits.
    #[error("LinearAddressBits.")]
    LinearAddressBits,
    /// Ebx.
    #[error("Ebx.")]
    Ebx,
}

impl Supports for Leaf80000008 {
    type Error = Leaf80000008NotSupported;
    /// We check everything here.
    #[inline]
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        warn_support!("0x80000008", true, true, true, true);

        if self.eax.physical_address_bits() < other.eax.physical_address_bits() {
            return Err(Leaf80000008NotSupported::PhysicalAddressBits);
        }
        if self.eax.linear_address_bits() < other.eax.linear_address_bits() {
            return Err(Leaf80000008NotSupported::LinearAddressBits);
        }
        if matches!(self.ebx.cmp_flags(&other.ebx), Some(Ordering::Less) | None) {
            return Err(Leaf80000008NotSupported::Ebx);
        }
        Ok(())
    }
}
