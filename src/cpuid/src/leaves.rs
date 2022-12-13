// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use construct::Inline;

use super::*;

/// A generic leaf formed of 4 members `eax`, `ebx`, `ecx` and `edx`.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(C)]
pub struct Leaf<A, B, C, D> {
    /// EAX register.
    pub eax: A,
    /// EBX register.
    pub ebx: B,
    /// ECX register.
    pub ecx: C,
    /// EDX register.
    pub edx: D,
}
impl<A: Inline, B: Inline, C: Inline, D: Inline> Inline for Leaf<A, B, C, D> {
    fn inline(&self) -> construct::TokenStream {
        let (a, b, c, d) = (
            self.eax.inline(),
            self.ebx.inline(),
            self.ecx.inline(),
            self.edx.inline(),
        );
        construct::quote! {
            Leaf {
                eax: #a,
                ebx: #b,
                ecx: #c,
                edx: #d
            }
        }
    }
}

// #[cfg(cpuid)]
// impl<A: From<u32> ,B: From<u32> ,C: From<u32>, D: From<u32>> From<CpuidResult> for Leaf<A, B, C,
// D> {     fn from(CpuidResult { eax, ebx, ecx, edx }: CpuidResult) -> Self {
//         Self(Leaf { eax: A::from(eax), ebx: ebx::from(ebx), ecx: ecx::from(ecx), edx:
// edx::from(edx) })     }
// }

impl<A, B, C, D> From<(A, B, C, D)> for Leaf<A, B, C, D> {
    fn from((a, b, c, d): (A, B, C, D)) -> Self {
        Leaf {
            eax: a,
            ebx: b,
            ecx: c,
            edx: d,
        }
    }
}

#[cfg(cpuid)]
impl<A: From<u32>, B: From<u32>, C: From<u32>, D: From<u32>> From<std::arch::x86_64::CpuidResult>
    for Leaf<A, B, C, D>
{
    fn from(
        std::arch::x86_64::CpuidResult { eax, ebx, ecx, edx }: std::arch::x86_64::CpuidResult,
    ) -> Self {
        Leaf {
            eax: A::from(eax),
            ebx: B::from(ebx),
            ecx: C::from(ecx),
            edx: D::from(edx),
        }
    }
}
impl<A: From<u32>, B: From<u32>, C: From<u32>, D: From<u32>> From<&RawKvmCpuidEntry>
    for Leaf<A, B, C, D>
{
    fn from(
        &RawKvmCpuidEntry {
            eax, ebx, ecx, edx, ..
        }: &RawKvmCpuidEntry,
    ) -> Self {
        Leaf {
            eax: A::from(eax),
            ebx: B::from(ebx),
            ecx: C::from(ecx),
            edx: D::from(edx),
        }
    }
}

impl<A: Equal, B: Equal, C: Equal, D: Equal> Equal for Leaf<A, B, C, D> {
    fn equal(&self, other: &Self) -> bool {
        self.eax.equal(&other.eax)
            && self.ebx.equal(&other.ebx)
            && self.ecx.equal(&other.ecx)
            && self.edx.equal(&other.edx)
    }
}

// -------------------------------------------------------------------------------------------------
// Shared leaf types
// -------------------------------------------------------------------------------------------------

/// Leaf 00H
pub type Leaf0 = Leaf<u32, u32, u32, u32>;

/// Leaf 80000002H
pub type Leaf80000002 = Leaf<Leaf80000002Eax, Leaf80000002Ebx, Leaf80000002Ecx, Leaf80000002Edx>;

/// Leaf 80000003H
pub type Leaf80000003 = Leaf80000002;

/// Leaf 80000004H
pub type Leaf80000004 = Leaf80000002;

// -------------------------------------------------------------------------------------------------
// Supports
// -------------------------------------------------------------------------------------------------

/// Logs a warning depending on which registers where not fully checked within a leaf.
// TODO Remove this macro export
#[macro_export]
macro_rules! warn_support {
    ($a:literal, $eax:literal, $ebx:literal, $ecx:literal, $edx:literal) => {
        if let Some(msg) = crate::support_warn($eax, $ebx, $ecx, $edx) {
            log::warn!(
                "Could not fully validate support for Intel CPUID leaf {} due to being unable to \
                 fully compare register/s: {}.",
                $a,
                msg
            );
        }
    };
}
/// Returns a static string depending the register boolean.
#[allow(clippy::fn_params_excessive_bools)]
pub(crate) const fn support_warn(
    eax: bool,
    ebx: bool,
    ecx: bool,
    edx: bool,
) -> Option<&'static str> {
    match (eax, ebx, ecx, edx) {
        (true, true, true, true) => None,
        (false, true, true, true) => Some("EAX"),
        (true, false, true, true) => Some("EBX"),
        (true, true, false, true) => Some("ECX"),
        (true, true, true, false) => Some("EDX"),
        (false, false, true, true) => Some("EAX and EBX"),
        (false, true, false, true) => Some("EAX and ECX"),
        (false, true, true, false) => Some("EAX and EDX"),
        (true, false, false, true) => Some("EBX and ECX"),
        (true, false, true, false) => Some("EBX and EDX"),
        (true, true, false, false) => Some("ECX and EDX"),
        (false, false, false, true) => Some("EAX, EBX and ECX"),
        (false, false, true, false) => Some("EAX, EBX and EDX"),
        (false, true, false, false) => Some("EAX, ECX and EDX"),
        (true, false, false, false) => Some("EBX, ECX and EDX"),
        (false, false, false, false) => Some("EAX, EBX, ECX and EDX"),
    }
}

/// Error type for [`<Leaf0 as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Leaf0NotSupported {
    /// Maximum input value.
    #[error("Maximum input value: {0} < {1}.")]
    MaximumInputValue(u32, u32),
    /// Manufacturer ID.
    #[error("Manufacturer ID: {0:?} != {1:?}.")]
    ManufacturerId([u32; 3], [u32; 3]),
}

impl Supports for Leaf0 {
    type Error = Leaf0NotSupported;
    /// We check the manufacturer id e.g. 'GenuineIntel' is an exact match and that
    /// 'Maximum Input Value for Basic CPUID Information.' is >=
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        warn_support!("0x0", true, true, true, true);

        if !(self.ebx == other.ebx && self.ecx == other.ecx && self.edx == other.edx) {
            return Err(Leaf0NotSupported::ManufacturerId(
                [self.ebx, self.ecx, self.edx],
                [other.ebx, other.ecx, other.edx],
            ));
        }
        if self.eax < other.eax {
            return Err(Leaf0NotSupported::MaximumInputValue(self.eax, other.eax));
        }

        Ok(())
    }
}
