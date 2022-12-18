// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#[cfg(cpuid)]
use bit_fields::CheckedAssignError;

/// Error type for [`AmdCpuid::normalize`].
#[cfg(cpuid)]
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum NormalizeCpuidError {
    /// Provided `cpu_bits` is >=8.
    #[error("Provided `cpu_bits` is >=8: {0}.")]
    CpuBits(u8),
    /// Missing leaf 0x80000000.
    #[error("Missing leaf 0x80000000.")]
    MissingLeaf0x80000000,
    /// Missing leaf 0x80000001.
    #[error("Missing leaf 0x80000001.")]
    MissingLeaf0x80000001,
    /// Failed to set feature entry leaf.
    #[error("Failed to set feature entry leaf: {0}")]
    FeatureEntry(#[from] FeatureEntryError),
    /// Failed to set extended cache topology leaf.
    #[error("Failed to set extended cache topology leaf: {0}")]
    ExtendedCacheTopology(#[from] ExtendedCacheTopologyError),
    /// Failed to set extended APIC ID leaf.
    #[error("Failed to set extended APIC ID leaf: {0}")]
    ExtendedApicId(#[from] ExtendedApicIdError),
    /// Failed to set brand string.
    #[error("Failed to set brand string: {0}")]
    BrandString(crate::MissingBrandStringLeaves),
}
/// Error type for setting leaf 0x80000008 section of [`AmdCpuid::normalize`].
#[cfg(cpuid)]
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum FeatureEntryError {
    /// Missing leaf 0x80000008.
    #[error("Missing leaf 0x80000008.")]
    MissingLeaf0x80000008,
    /// Failed to set `nt` (number of physical threads) due to overflow.
    #[error("Failed to set `nt` (number of physical threads) due to overflow.")]
    NumberOfPhysicalThreadsOverflow,
    /// Failed to set `nt` (number of physical threads).
    #[error("Failed to set `nt` (number of physical threads).")]
    NumberOfPhysicalThreads(CheckedAssignError),
}

/// Error type for setting leaf 0x8000001d section of [`AmdCpuid::normalize`].
#[cfg(cpuid)]
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum ExtendedCacheTopologyError {
    /// Missing leaf 0x8000001d.
    #[error("Missing leaf 0x8000001d.")]
    MissingLeaf0x8000001d,
    /// Failed to set `num_sharing_cache` due to overflow.
    #[error("Failed to set `num_sharing_cache` due to overflow.")]
    NumSharingCacheOverflow,
    /// Failed to set `num_sharing_cache`.
    #[error("Failed to set `num_sharing_cache`: {0}")]
    NumSharingCache(CheckedAssignError),
}

/// Error type for setting leaf 0x8000001e section of [`AmdCpuid::normalize`].
#[cfg(cpuid)]
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum ExtendedApicIdError {
    /// Missing leaf 0x8000001d.
    #[error("Missing leaf 0x8000001e.")]
    MissingLeaf0x8000001e,
    /// Failed to set `extended_apic_id`.
    #[error("Failed to set `extended_apic_id`: {0}")]
    ExtendedApicId(CheckedAssignError),
    /// Failed to set `compute_unit_id`.
    #[error("Failed to set `compute_unit_id`: {0}")]
    ComputeUnitId(CheckedAssignError),
    /// Failed to set `threads_per_compute_unit`.
    #[error("Failed to set `threads_per_compute_unit`: {0}")]
    ThreadPerComputeUnit(CheckedAssignError),
}
