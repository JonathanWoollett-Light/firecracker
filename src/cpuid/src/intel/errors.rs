// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use bit_fields::CheckedAssignError;

/// Error type for `get_max_cpus_per_package`.
#[derive(Debug, thiserror::Error)]
pub enum GetMaxCpusPerPackageError {
    /// Failed to get max CPUs per package as `cpu_count == 0`.
    #[error("Failed to get max CPUs per package as `cpu_count == 0`")]
    Underflow,
    /// Failed to get max CPUs per package as `cpu_count > 128`.
    #[error("Failed to get max CPUs per package as `cpu_count > 128`")]
    Overflow,
}

/// Error type for [`IntelCpuid::apply_vm_spec`].
#[cfg(cpuid)]
#[derive(Debug, thiserror::Error)]
pub enum ApplyVmSpecError {
    /// Failed to set feature infomation leaf.
    #[error("Failed to set feature infomation leaf: {0}")]
    FeatureInfomation(#[from] FeatireInfomationError),
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
}

// TODO These `MissingLeafX` types can be `MissingLeaf<const N:u8>;`
/// Leaf 0x0 is missing from CPUID.
#[derive(Debug, thiserror::Error)]
#[error("Leaf 0x0 is missing from CPUID.")]
pub struct MissingLeaf0;

/// Error type for setting leaf 1 section of `IntelCpuid::apply_vm_spec`.
#[derive(Debug, thiserror::Error)]
pub enum FeatireInfomationError {
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

/// Error type for setting leaf 4 section of `IntelCpuid::apply_vm_spec`.
#[derive(Debug, thiserror::Error)]
pub enum DeterministicCacheError {
    /// Failed to set `Maximum number of addressable IDs for logical processors sharing this
    /// cache`.
    #[error(
        "Failed to set `Maximum number of addressable IDs for logical processors sharing this \
         cache`: {0}"
    )]
    MaxCpusPerCore(CheckedAssignError),
    /// Failed to set `Maximum number of addressable IDs for processor cores in the physical
    /// package`.
    #[error(
        "Failed to set `Maximum number of addressable IDs for processor cores in the physical \
         package`: {0}"
    )]
    MaxCorePerPackage(CheckedAssignError),
}

/// Error type for setting leaf b section of `IntelCpuid::apply_vm_spec`.
#[derive(Debug, thiserror::Error)]
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
}

/// Error type for [`<IntelCpuid as TryFrom<RawCpuid>>::from()`].
#[derive(Debug, thiserror::Error)]
#[error("Leaf {leaf:x} subleaf {subleaf:x} not found in the given `RawCpuid`.")]
pub struct MissingLeaf {
    leaf: u32,
    subleaf: u32,
}
impl From<u32> for MissingLeaf {
    fn from(leaf: u32) -> Self {
        Self { leaf, subleaf: 0 }
    }
}
impl From<(u32, u32)> for MissingLeaf {
    fn from((leaf, subleaf): (u32, u32)) -> Self {
        Self { leaf, subleaf }
    }
}
