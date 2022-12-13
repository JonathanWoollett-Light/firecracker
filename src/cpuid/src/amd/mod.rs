// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use serde::{Deserialize, Serialize};

use crate::RawCpuid;

/// Error type for [`AmdCpuid::apply_vm_spec`].
#[cfg(cpuid)]
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
#[error("Failed to apply `VmSpec`.")]
pub struct ApplyVmSpecError;

/// A structure containing the information as described in the AMD CPUID specification as described
/// in
/// [AMD64 Architecture Programmer’s Manual Volume 3: General-Purpose and System Instructions](https://www.amd.com/system/files/TechDocs/24594.pdf)
/// .
///
/// # Notes
///
/// We not do not currently check AMD features on snapshot restore.
#[allow(clippy::unsafe_derive_deserialize, clippy::module_name_repetitions)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, construct::Inline)]
#[repr(C)]
pub struct AmdCpuid(pub RawCpuid);

// TODO: Replace checking of CPUID avaiblity with `x86` and `x86_64` check and
// [`std::arch_x86_64::has_cpuid()`] when this is stabilized. CPUID is supported when:
// - We are on an x86 archtecture with `sse` enabled and `sgx disabled`.
// - We are on an x86_64 architecture with `sgx` disabled
impl AmdCpuid {
    /// Alias for [`AmdCpuid::default`]
    #[must_use]
    pub fn new() -> Self {
        Self(RawCpuid::new())
    }
}
impl AmdCpuid {
    /// Returns the CPUID manufacturers ID (e.g. `GenuineIntel` or `AuthenticAMD`) or `None` if it
    /// cannot be found in CPUID (e.g. leaf 0x0 is missing).
    #[allow(clippy::similar_names)]
    #[must_use]
    pub fn manufacturer_id(&self) -> Option<[u8; 12]> {
        let leaf_0 = self.0.get(0, 0)?;

        // The ordering of the manufacturer string is ebx,edx,ecx this is not a mistake.
        let (ebx, edx, ecx) = (
            leaf_0.ebx.to_ne_bytes(),
            leaf_0.edx.to_ne_bytes(),
            leaf_0.ecx.to_ne_bytes(),
        );
        let arr: [u8; 12] = [
            ebx[0], ebx[1], ebx[2], ebx[3], edx[0], edx[1], edx[2], edx[3], ecx[0], ecx[1], ecx[2],
            ecx[3],
        ];

        Some(arr)
    }
    /// Applies `vm_spec` to `self`.
    ///
    /// # Errors
    ///
    /// Never.
    #[cfg(cpuid)]
    #[allow(clippy::unused_self)]
    pub fn apply_vm_spec(&mut self, _vm_spec: &crate::VmSpec) -> Result<(), ApplyVmSpecError> {
        // unimplemented
        Ok(())
    }
}

/// Error type for [`<AmdCpuidNotSupported as Supports>::supports`].
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
#[error("AmdCpuidNotSupported.")]
pub struct AmdCpuidNotSupported;

impl crate::Supports for AmdCpuid {
    type Error = AmdCpuidNotSupported;
    /// Checks if `self` is a able to support `other`.
    ///
    /// Checks if a process from an environment with CPUID `other` could be continued in an
    /// environment with the CPUID `self`.
    fn supports(&self, _other: &Self) -> Result<(), Self::Error> {
        Ok(())
    }
}
impl Default for AmdCpuid {
    /// Constructs new `Cpuid` via [`std::arch::x86_64::__cpuid_count`].
    ///
    /// # Note
    ///
    /// As we do not currently support the AMD CPUID specification this constructs an empty
    /// [`RawCpuid`].
    fn default() -> Self {
        Self(RawCpuid::new())
    }
}
impl From<RawCpuid> for AmdCpuid {
    fn from(raw_cpuid: RawCpuid) -> Self {
        Self(raw_cpuid)
    }
}
impl From<AmdCpuid> for RawCpuid {
    fn from(amd_cpuid: AmdCpuid) -> Self {
        amd_cpuid.0
    }
}
