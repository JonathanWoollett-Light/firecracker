// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#[cfg(cpuid)]
use bit_fields::CheckedAssignError;

/// Errors associated with processing the CPUID leaves.
#[derive(Debug, Clone)]
pub enum Error {
    /// A FamStructWrapper operation has failed
    FamError(utils::fam::Error),
    /// A call to an internal helper method failed
    InternalError(crate::common::Error),
    /// The operation is not permitted for the current vendor
    InvalidVendor,
    /// The maximum number of addressable logical CPUs cannot be stored in an `u8`.
    VcpuCountOverflow,
}

impl From<utils::fam::Error> for Error {
    #[inline]
    fn from(x: utils::fam::Error) -> Self {
        Self::FamError(x)
    }
}

impl From<crate::common::Error> for Error {
    #[inline]
    fn from(x: crate::common::Error) -> Self {
        Self::InternalError(x)
    }
}

/// Error type for [`<Cpuid as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum CpuidNotSupported {
    /// Intel.
    #[error("Intel: {0}")]
    Intel(crate::intel::IntelCpuidNotSupported),
    /// Amd.
    #[error("Amd: {0}")]
    Amd(crate::amd::AmdCpuidNotSupported),
    /// Different manufacturer IDs.
    #[error("Different manufacturer IDs.")]
    Incompatible,
}

/// Error type for [`apply_brand_string`].
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
#[error("Missing brand string leaves 0x80000002, 0x80000003 and 0x80000004.")]
pub struct MissingBrandStringLeaves;

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
pub enum NormalizeCpuidError {
    /// Failed to apply modifications to Intel CPUID.
    #[error("Failed to apply modifications to Intel CPUID: {0}")]
    Intel(#[from] crate::intel::NormalizeCpuidError),
    /// Failed to apply modifications to AMD CPUID.
    #[error("Failed to apply modifications to AMD CPUID: {0}")]
    Amd(#[from] crate::amd::NormalizeCpuidError),
    /// Failed to set feature information leaf.
    #[error("Failed to set feature information leaf: {0}")]
    FeatureInfomation(#[from] FeatureInformationError),
}

/// Error type for setting leaf 1 section of `IntelCpuid::normalize`.
#[cfg(cpuid)]
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum FeatureInformationError {
    /// Leaf 0x1 is missing from CPUID.
    #[error("Leaf 0x1 is missing from CPUID.")]
    MissingLeaf1,
    /// Failed to set `Initial APIC ID`.
    #[error("Failed to set `Initial APIC ID`: {0}")]
    InitialApicId(CheckedAssignError),
    /// Failed to set `CLFLUSH line size`.
    #[error("Failed to set `CLFLUSH line size`: {0}")]
    Clflush(CheckedAssignError),
    /// Failed to get max CPUs per package.
    #[error("Failed to get max CPUs per package: {0}")]
    GetMaxCpusPerPackage(GetMaxCpusPerPackageError),
    /// Failed to set max CPUs per package.
    #[error("Failed to set max CPUs per package: {0}")]
    SetMaxCpusPerPackage(CheckedAssignError),
}

/// Error type for `get_max_cpus_per_package`.
#[cfg(cpuid)]
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum GetMaxCpusPerPackageError {
    /// Failed to get max CPUs per package as `cpu_count == 0`.
    #[error("Failed to get max CPUs per package as `cpu_count == 0`")]
    Underflow,
    /// Failed to get max CPUs per package as `cpu_count > 128`.
    #[error("Failed to get max CPUs per package as `cpu_count > 128`")]
    Overflow,
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
}
