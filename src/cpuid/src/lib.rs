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
// This is unused (emits a `dead_code` warning) when cpuid is not supported.
#[cfg(cpuid)]
use common::GetCpuidError;
pub use cpuid_ffi::*;
pub use intel::IntelCpuid;

/// cpuid utility functions.
pub mod common;

/// Contains helper methods for bit operations.
pub mod bit_helper;
#[cfg(cpuid)]
mod brand_string;
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

/// Structure containing the specifications of the VM
#[cfg(cpuid)]
pub struct VmSpec {
    /// The vendor id of the CPU
    cpu_vendor_id: [u8; 12],
    /// The desired brand string for the guest.
    #[allow(dead_code)]
    brand_string: brand_string::BrandString,
    /// The index of the current logical CPU in the range [0..cpu_count].
    cpu_index: u8,
    /// The total number of logical CPUs.
    cpu_count: u8,
    /// The number of bits needed to enumerate logical CPUs per core.
    cpu_bits: u8,
}

#[cfg(cpuid)]
impl VmSpec {
    /// Creates a new instance of [`VmSpec`] with the specified parameters
    /// The brand string is deduced from the `vendor_id`.
    ///
    /// # Errors
    ///
    /// When CPUID leaf 0 is not supported.
    pub fn new(cpu_index: u8, cpu_count: u8, smt: bool) -> Result<VmSpec, GetCpuidError> {
        let cpu_vendor_id = common::get_vendor_id_from_host()?;
        Ok(VmSpec {
            cpu_vendor_id,
            cpu_index,
            cpu_count,
            cpu_bits: u8::from(cpu_count > 1 && smt),
            brand_string: brand_string::BrandString::from_vendor_id(&cpu_vendor_id),
        })
    }

    /// Returns an immutable reference to `cpu_vendor_id`.
    #[must_use]
    pub fn cpu_vendor_id(&self) -> &[u8; 12] {
        &self.cpu_vendor_id
    }

    /// Returns the number of cpus per core
    #[must_use]
    pub fn cpus_per_core(&self) -> u8 {
        1 << self.cpu_bits
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

/// Error type for [`Cpuid::apply_vm_spec`].
#[cfg(cpuid)]
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum ApplyVmSpecError {
    /// Failed to apply VmSpec to Intel CPUID.
    #[error("Failed to apply VmSpec to Intel CPUID: {0}")]
    Intel(#[from] intel::ApplyVmSpecError),
    /// Failed to apply VmSpec to AMD CPUID.
    #[error("Failed to apply VmSpec to AMD CPUID: {0}")]
    Amd(#[from] amd::ApplyVmSpecError),
}

impl Cpuid {
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
    /// Applies `vm_spec` to `self`.
    ///
    /// # Errors
    ///
    /// When failing:
    /// - [`Cpuid::IntelCpuid::apply_vm_spec`].
    /// - [`Cpuid::AmdCpuid::apply_vm_spec`].
    #[cfg(cpuid)]
    pub fn apply_vm_spec(&mut self, vm_spec: &VmSpec) -> Result<(), ApplyVmSpecError> {
        match self {
            Self::Intel(intel) => intel
                .apply_vm_spec(vm_spec)
                .map_err(ApplyVmSpecError::Intel),
            Self::Amd(amd) => amd.apply_vm_spec(vm_spec).map_err(ApplyVmSpecError::Amd),
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
