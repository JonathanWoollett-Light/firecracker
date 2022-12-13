// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//
// Portions Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the THIRD-PARTY file.
#![warn(clippy::pedantic)]
#![allow(
    clippy::unsafe_derive_deserialize,
    clippy::unreadable_literal,
    clippy::doc_markdown,
    clippy::similar_names
)]
#![warn(missing_docs)]
#![warn(clippy::ptr_as_ptr)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(clippy::cast_lossless)]
//! Utility for configuring the CPUID (CPU identification) for the guest microVM.

use std::convert::TryFrom;

pub use amd::AmdCpuid;
use bit_fields::Equal;
pub use cpuid_ffi::*;
pub use intel::IntelCpuid;

/// cpuid utility functions.
pub mod common;

/// Indexing implementations (shared between AMD and Intel).
mod indexing;
pub use indexing::*;

/// Register bit fields (shared between AMD and Intel).
mod registers;
pub use registers::*;

/// Leaf structs (shared between AMD and Intel).
mod leaves;
pub use leaves::*;

/// Contains helper methods for bit operations.
pub mod bit_helper;
/// T2S Intel template
#[cfg(cpuid)]
pub mod t2s;

/// AMD CPUID specification handling.
pub mod amd;
/// Raw CPUID specification handling.
mod cpuid_ffi;
/// Intel CPUID specification handling.
pub mod intel;

/// Errors associated with processing the CPUID leaves.
#[derive(Debug, Clone)]
pub enum Error {
    /// A FamStructWrapper operation has failed
    FamError(utils::fam::Error),
    /// A call to an internal helper method failed
    InternalError(common::Error),
    /// The operation is not permitted for the current vendor
    InvalidVendor,
    /// The maximum number of addressable logical CPUs cannot be stored in an `u8`.
    VcpuCountOverflow,
}

impl From<utils::fam::Error> for Error {
    fn from(x: utils::fam::Error) -> Self {
        Self::FamError(x)
    }
}

impl From<common::Error> for Error {
    fn from(x: common::Error) -> Self {
        Self::InternalError(x)
    }
}

/// CPUID information
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, construct::Inline)]
#[repr(C)]
pub enum Cpuid {
    /// Intel CPUID specific information.
    Intel(IntelCpuid),
    /// AMD CPUID specific information.
    Amd(AmdCpuid),
}

/// Error type for [`Cpuid::kvm_get_supported_cpuid`].
#[cfg(cpuid)]
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum KvmGetSupportedCpuidError {
    /// Could not access KVM.
    #[error("Could not access KVM: {0}")]
    KvmAccess(#[from] utils::errno::Error),
    /// Failed to create CPUID structure.
    #[error("Failed to create CPUID structure: {0}")]
    CpuidFromRaw(CpuidTryFromRawCpuid),
}

/// Error type for [`Cpuid::normalize`].
#[cfg(cpuid)]
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum NormalizeCpuid {
    /// Failed to apply modifications to Intel CPUID.
    #[error("Failed to apply modifications to Intel CPUID: {0}")]
    Intel(#[from] intel::NormalizeCpuid),
    /// Failed to apply modifications to AMD CPUID.
    #[error("Failed to apply modifications to AMD CPUID: {0}")]
    Amd(#[from] amd::NormalizeCpuid),
}

impl Cpuid {
    /// When a microVM is started without a template, we use this brand string.
    pub const DEFUALT_INTEL_BRAND_STRING: &[u8] = b"Intel(R) Xeon(R) Processor";
    /// When a microVM is started without a template, we use this brand string.
    pub const DEFAULT_AMD_BRAND_STRING: &[u8] = b"AMD EPYC";

    /// Returns the CPUID manufacturers ID  (e.g. `GenuineIntel` or `AuthenticAMD`).
    #[cfg(cpuid)]
    #[must_use]
    pub fn host_manufacturer_id() -> [u8; 12] {
        // SAFETY: The `cpuid` feature guarantees CPUID is supported thus this is safe.
        let leaf_0 = unsafe { std::arch::x86_64::__cpuid_count(0, 0) };

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

        arr
    }
    /// Gets supported CPUID by KVM.
    ///
    /// # Errors
    ///
    /// When failed to access KVM.
    #[cfg(cpuid)]
    pub fn kvm_get_supported_cpuid() -> std::result::Result<Self, KvmGetSupportedCpuidError> {
        let supported_kvm_cpuid =
            kvm_ioctls::Kvm::new()?.get_supported_cpuid(kvm_bindings::KVM_MAX_CPUID_ENTRIES)?;
        let supported_raw_cpuid = RawCpuid::from(supported_kvm_cpuid);
        Cpuid::try_from(supported_raw_cpuid).map_err(KvmGetSupportedCpuidError::CpuidFromRaw)
    }
    /// Returns `Some(&IntelCpuid)` if `Self == Self::Intel(_)` else returns `None`.
    #[must_use]
    pub fn intel(&self) -> Option<&IntelCpuid> {
        match self {
            Self::Intel(intel) => Some(intel),
            Self::Amd(_) => None,
        }
    }
    /// Returns `Some(&AmdCpuid)` if `Self == Self::Amd(_)` else returns `None`.
    #[must_use]
    pub fn amd(&self) -> Option<&AmdCpuid> {
        match self {
            Self::Intel(_) => None,
            Self::Amd(amd) => Some(amd),
        }
    }
    /// Returns the CPUID manufacturers ID (e.g. `GenuineIntel` or `AuthenticAMD`) or `None` if it
    /// cannot be found in CPUID (e.g. leaf 0x0 is missing).
    #[must_use]
    pub fn manufacturer_id(&self) -> Option<[u8; 12]> {
        match self {
            Self::Intel(intel) => intel.manufacturer_id(),
            Self::Amd(amd) => amd.manufacturer_id(),
        }
    }
    /// Applies required modifications to CPUID respective of a vCPU.
    ///
    /// # Errors
    ///
    /// When failing:
    /// - [`Cpuid::IntelCpuid::normalize`].
    /// - [`Cpuid::AmdCpuid::normalize`].
    #[cfg(cpuid)]
    pub fn normalize(
        &mut self,
        // The index of the current logical CPU in the range [0..cpu_count].
        cpu_index: u8,
        // The total number of logical CPUs.
        cpu_count: u8,
        // The number of bits needed to enumerate logical CPUs per core.
        cpu_bits: u8,
    ) -> Result<(), NormalizeCpuid> {
        match self {
            Self::Intel(intel) => intel
                .normalize(cpu_index, cpu_count, cpu_bits)
                .map_err(NormalizeCpuid::Intel),
            Self::Amd(amd) => amd
                .normalize(cpu_index, cpu_count, cpu_bits)
                .map_err(NormalizeCpuid::Amd),
        }
    }
    /// Compares `self` to `other` ignoring undefined bits.
    #[must_use]
    pub fn equal(&self, other: &Self) -> bool {
        match (self, other) {
            (Cpuid::Intel(a), Cpuid::Intel(b)) => a.equal(b),
            (Cpuid::Amd(a), Cpuid::Amd(b)) => a == b,
            _ => false,
        }
    }

    /// Get immutable reference to leaf.
    #[must_use]
    pub fn leaf<'a, const N: usize>(&'a self) -> <Self as IndexLeaf<N>>::Output<'a>
    where
        Self: IndexLeaf<N>,
    {
        <Self as IndexLeaf<N>>::index_leaf(self)
    }

    /// Get mutable reference to leaf.
    #[must_use]
    pub fn leaf_mut<'a, const N: usize>(&'a mut self) -> <Self as IndexLeafMut<N>>::Output<'a>
    where
        Self: IndexLeafMut<N>,
    {
        <Self as IndexLeafMut<N>>::index_leaf_mut(self)
    }

    /// Gets a given sub-leaf.
    pub fn get(&mut self, key: &CpuidKey) -> Option<&CpuidEntry> {
        match self {
            Self::Intel(intel_cpuid) => intel_cpuid.get(key),
            Self::Amd(amd_cpuid) => amd_cpuid.get(key),
        }
    }

    /// Gets a given sub-leaf.
    pub fn get_mut(&mut self, key: &CpuidKey) -> Option<&mut CpuidEntry> {
        match self {
            Self::Intel(intel_cpuid) => intel_cpuid.get_mut(key),
            Self::Amd(amd_cpuid) => amd_cpuid.get_mut(key),
        }
    }
}

/// Error type for [`<Cpuid as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum CpuidNotSupported {
    /// Intel.
    #[error("Intel: {0}")]
    Intel(intel::IntelCpuidNotSupported),
    /// Amd.
    #[error("Amd: {0}")]
    Amd(amd::AmdCpuidNotSupported),
    /// Different manufacturer IDs.
    #[error("Different manufacturer IDs.")]
    Incompatible,
}

impl Supports for Cpuid {
    type Error = CpuidNotSupported;
    /// Compare support of `self` to support `other`.
    ///
    /// For checking if a process from an environment with cpuid `other` could be continued in the
    /// environment with the cpuid `self`.
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        match (self, other) {
            (Self::Intel(a), Self::Intel(b)) => a.supports(b).map_err(CpuidNotSupported::Intel),
            (Self::Amd(a), Self::Amd(b)) => a.supports(b).map_err(CpuidNotSupported::Amd),
            _ => Err(CpuidNotSupported::Incompatible),
        }
    }
}

/// Trait defining if a CPUID component supports another.
pub trait Supports {
    /// Error type.
    type Error;
    /// Returns `Ok(())` if `self` supports `other` or `Err(reason)` if it does not.
    ///
    /// # Errors
    ///
    /// When `self` does not support `other`.
    fn supports(&self, other: &Self) -> Result<(), Self::Error>;
}

/// Error type for [`<Cpuid as TryFrom<RawCpuid>>::try_from`].
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum CpuidTryFromRawCpuid {
    /// Leaf 0 not found in the given `RawCpuid`..
    #[error("Leaf 0 not found in the given `RawCpuid`.")]
    MissingLeaf0,
    /// Unsupported CPUID manufacturer id.
    #[error(
        "Unsupported CPUID manufacturer id: \"{0:?}\" (only 'GenuineIntel' and 'AuthenticAMD' are \
         supported)."
    )]
    UnsupportedManufacturer([u8; 12]),
    /// Custom
    #[error("Custom: {0}")]
    Custom(String),
}

#[allow(warnings)]
impl TryFrom<RawCpuid> for Cpuid {
    type Error = CpuidTryFromRawCpuid;
    fn try_from(raw_cpuid: RawCpuid) -> Result<Self, Self::Error> {
        let leaf_0 = raw_cpuid
            .get(0, 0)
            .ok_or(CpuidTryFromRawCpuid::MissingLeaf0)?;
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

        // Temp check
        // return Err(CpuidTryFromRawCpuid::UnsupportedManufacturer(arr));

        // TODO: Need to double check this is safe.
        // SAFETY: If we attempt to use the array directly in the match, comparing `&arr` to
        // `b"GenuineIntel"` this results in a compiler error.
        // To workaround this we use `&str` we do not care if it is valid utf8 thus we uncheck
        // convert.
        let manufacturer_str: &str = unsafe { std::str::from_utf8_unchecked(&arr) };

        match manufacturer_str {
            "GenuineIntel" => Ok(Cpuid::Intel(IntelCpuid::from(raw_cpuid))),
            "AuthenticAMD" => Ok(Cpuid::Amd(AmdCpuid::from(raw_cpuid))),
            _ => Err(CpuidTryFromRawCpuid::UnsupportedManufacturer(arr)),
        }
    }
}

impl From<Cpuid> for RawCpuid {
    fn from(cpuid: Cpuid) -> Self {
        match cpuid {
            Cpuid::Intel(intel_cpuid) => RawCpuid::from(intel_cpuid),
            Cpuid::Amd(amd_cpuid) => RawCpuid::from(amd_cpuid),
        }
    }
}

#[cfg(cpuid)]
impl From<Cpuid> for kvm_bindings::CpuId {
    fn from(cpuid: Cpuid) -> Self {
        let raw_cpuid = RawCpuid::from(cpuid);
        Self::from(raw_cpuid)
    }
}

/// CPUID index values `leaf` and `subleaf`.
#[derive(Debug, Clone, Default, PartialEq, Eq, construct::Inline)]
pub struct CpuidKey {
    /// CPUID leaf.
    pub leaf: u32,
    /// CPUID subleaf.
    pub subleaf: u32,
}

impl CpuidKey {
    /// `CpuidKey { leaf, subleaf: 0 }`
    #[must_use]
    pub fn leaf(leaf: u32) -> Self {
        Self { leaf, subleaf: 0 }
    }
    /// `CpuidKey { leaf, subleaf }`
    #[must_use]
    pub fn subleaf(leaf: u32, subleaf: u32) -> Self {
        Self { leaf, subleaf }
    }
}

impl std::cmp::PartialOrd for CpuidKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            self.leaf
                .cmp(&other.leaf)
                .then(self.subleaf.cmp(&other.subleaf)),
        )
    }
}

impl std::cmp::Ord for CpuidKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

/// CPUID entry information stored for each leaf of [`IntelCpuid`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, construct::Inline)]
pub struct CpuidEntry {
    /// The KVM requires a `flags` parameter which indicates if a given CPUID leaf has sub-leaves.
    /// This does not change at runtime so we can save memory by not storing this under every
    /// sub-leaf and instead fetching from a map when converting back to the KVM CPUID
    /// structure. But for robustness we currently do store we do not use this approach.
    ///
    /// A map on flags would look like:
    /// ```ignore
    /// use cpuid::KvmCpuidFlags;
    /// #[allow(clippy::non_ascii_literal)]
    /// pub static KVM_CPUID_LEAF_FLAGS: phf::Map<u32, KvmCpuidFlags> = phf::phf_map! {
    ///     0x00u32 => KvmCpuidFlags::empty(),
    ///     0x01u32 => KvmCpuidFlags::empty(),
    ///     0x02u32 => KvmCpuidFlags::empty(),
    ///     0x03u32 => KvmCpuidFlags::empty(),
    ///     0x04u32 => KvmCpuidFlags::SignificantIndex,
    ///     0x05u32 => KvmCpuidFlags::empty(),
    ///     0x06u32 => KvmCpuidFlags::empty(),
    ///     0x07u32 => KvmCpuidFlags::SignificantIndex,
    ///     0x09u32 => KvmCpuidFlags::empty(),
    ///     0x0Au32 => KvmCpuidFlags::empty(),
    ///     0x0Bu32 => KvmCpuidFlags::SignificantIndex,
    ///     0x0Fu32 => KvmCpuidFlags::SignificantIndex,
    ///     0x10u32 => KvmCpuidFlags::SignificantIndex,
    ///     0x12u32 => KvmCpuidFlags::SignificantIndex,
    ///     0x14u32 => KvmCpuidFlags::SignificantIndex,
    ///     0x15u32 => KvmCpuidFlags::empty(),
    ///     0x16u32 => KvmCpuidFlags::empty(),
    ///     0x17u32 => KvmCpuidFlags::SignificantIndex,
    ///     0x18u32 => KvmCpuidFlags::SignificantIndex,
    ///     0x19u32 => KvmCpuidFlags::empty(),
    ///     0x1Au32 => KvmCpuidFlags::empty(),
    ///     0x1Bu32 => KvmCpuidFlags::empty(),
    ///     0x1Cu32 => KvmCpuidFlags::empty(),
    ///     0x1Fu32 => KvmCpuidFlags::SignificantIndex,
    ///     0x20u32 => KvmCpuidFlags::empty(),
    ///     0x80000000u32 => KvmCpuidFlags::empty(),
    ///     0x80000001u32 => KvmCpuidFlags::empty(),
    ///     0x80000002u32 => KvmCpuidFlags::empty(),
    ///     0x80000003u32 => KvmCpuidFlags::empty(),
    ///     0x80000004u32 => KvmCpuidFlags::empty(),
    ///     0x80000005u32 => KvmCpuidFlags::empty(),
    ///     0x80000006u32 => KvmCpuidFlags::empty(),
    ///     0x80000007u32 => KvmCpuidFlags::empty(),
    ///     0x80000008u32 => KvmCpuidFlags::empty(),
    /// };
    /// ```
    pub flags: crate::cpuid_ffi::KvmCpuidFlags,
    /// Register values.
    pub result: CCpuidResult,
}

/// To transmute this into leaves such that we can return mutable reference to it with leaf specific
/// accessors, requires this to have a consistent member ordering. [`core::arch::x86::CpuidResult`]
/// is not `repr(C)`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, construct::Inline)]
#[repr(C)]
pub struct CCpuidResult {
    /// EDX
    pub eax: u32,
    /// EBX
    pub ebx: u32,
    /// ECX
    pub ecx: u32,
    /// EDX
    pub edx: u32,
}

#[cfg(cpuid)]
impl From<core::arch::x86_64::CpuidResult> for CCpuidResult {
    fn from(
        core::arch::x86_64::CpuidResult { eax, ebx, ecx, edx }: core::arch::x86_64::CpuidResult,
    ) -> Self {
        Self { eax, ebx, ecx, edx }
    }
}
impl From<(CpuidKey, CpuidEntry)> for RawKvmCpuidEntry {
    fn from(
        (CpuidKey { leaf, subleaf }, CpuidEntry { flags, result }): (CpuidKey, CpuidEntry),
    ) -> Self {
        let CCpuidResult { eax, ebx, ecx, edx } = result;
        Self {
            function: leaf,
            index: subleaf,
            flags,
            eax,
            ebx,
            ecx,
            edx,
            padding: Padding::default(),
        }
    }
}

impl From<RawKvmCpuidEntry> for (CpuidKey, CpuidEntry) {
    fn from(
        RawKvmCpuidEntry {
            function,
            index,
            flags,
            eax,
            ebx,
            ecx,
            edx,
            ..
        }: RawKvmCpuidEntry,
    ) -> Self {
        (
            CpuidKey {
                leaf: function,
                subleaf: index,
            },
            CpuidEntry {
                flags,
                result: CCpuidResult { eax, ebx, ecx, edx },
            },
        )
    }
}
