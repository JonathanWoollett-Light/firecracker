// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#[cfg(cpuid)]
use bit_fields::CheckedAssignError;

/// Error type for [`IntelCpuid::normalize`].
#[cfg(cpuid)]
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum NormalizeCpuidError {
    /// Provided `cpu_bits` is >=8.
    #[error("Provided `cpu_bits` is >=8: {0}.")]
    CpuBits(u8),
    /// Failed to set deterministic cache leaf.
    #[error("Failed to set deterministic cache leaf: {0}")]
    DeterministicCache(#[from] DeterministicCacheError),
    /// Leaf 0x6 is missing from CPUID.
    #[error("Leaf 0x6 is missing from CPUID.")]
    MissingLeaf6,
    /// Leaf 0xA is missing from CPUID.
    #[error("Leaf 0xA is missing from CPUID.")]
    MissingLeafA,
    /// Failed to set extended topology leaf.
    #[error("Failed to set extended topology leaf: {0}")]
    ExtendedTopology(#[from] ExtendedTopologyError),
    /// Failed to get brand string.
    #[error("Failed to get brand string: {0}")]
    GetBrandString(DefaultBrandStringError),
    /// Failed to set brand string.
    #[error("Failed to set brand string: {0}")]
    ApplyBrandString(crate::MissingBrandStringLeaves),
}

/// Error type for setting leaf 4 section of `IntelCpuid::normalize`.
#[cfg(cpuid)]
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum DeterministicCacheError {
    /// Failed to set `Maximum number of addressable IDs for logical processors sharing this
    /// cache` due to underflow in cpu count.
    #[error(
        "Failed to set `Maximum number of addressable IDs for logical processors sharing this \
         cache` due to underflow in cpu count."
    )]
    MaxCpusPerCoreUnderflow,
    /// Failed to set `Maximum number of addressable IDs for logical processors sharing this
    /// cache`.
    #[error(
        "Failed to set `Maximum number of addressable IDs for logical processors sharing this \
         cache`: {0}"
    )]
    MaxCpusPerCore(CheckedAssignError),
    /// Failed to set `Maximum number of addressable IDs for processor cores in the physical
    /// package` due to underflow in cores
    #[error(
        "Failed to set `Maximum number of addressable IDs for processor cores in the physical \
         package` due to underflow in cores."
    )]
    MaxCorePerPackageUnderflow,
    /// Failed to set `Maximum number of addressable IDs for processor cores in the physical
    /// package`.
    #[error(
        "Failed to set `Maximum number of addressable IDs for processor cores in the physical \
         package`: {0}"
    )]
    MaxCorePerPackage(CheckedAssignError),
}

/// Error type for setting leaf b section of `IntelCpuid::normalize`.
#[cfg(cpuid)]
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum ExtendedTopologyError {
    /// Failed to set `Number of bits to shift right on x2APIC ID to get a unique topology ID of
    /// the next level type`.
    #[error(
        "Failed to set `Number of bits to shift right on x2APIC ID to get a unique topology ID of \
         the next level type`: {0}"
    )]
    ApicId(CheckedAssignError),
    /// Failed to set `Number of logical processors at this level type`.
    #[error("Failed to set `Number of logical processors at this level type`: {0}")]
    LogicalProcessors(CheckedAssignError),
    /// Failed to set `Level Type`.
    #[error("Failed to set `Level Type`: {0}")]
    LevelType(CheckedAssignError),
    /// Failed to set `Level Number`.
    #[error("Failed to set `Level Number`: {0}")]
    LevelNumber(CheckedAssignError),
    /// Failed to set all leaves, as more than `u32::MAX` sub-leaves are present.
    #[error("Failed to set all leaves, as more than `u32::MAX` sub-leaves are present: {0}")]
    Overflow(<u32 as TryFrom<usize>>::Error),
}

/// Error type for [`IntelCpuid::default_brand_string`].
#[cfg(cpuid)]
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum DefaultBrandStringError {
    /// Missing frequency.
    #[error("Missing frequency: {0:?}.")]
    MissingFreqeuncy([u8; crate::BRAND_STRING_LENGTH]),
    /// Missing space.
    #[error("Missing space: {0:?}.")]
    MissingSpace([u8; crate::BRAND_STRING_LENGTH]),
    /// Insufficient space in brand string.
    #[error("Insufficient space in brand string.")]
    Overflow,
}
