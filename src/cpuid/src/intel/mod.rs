// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
#![allow(
    clippy::similar_names,
    clippy::module_name_repetitions,
    clippy::unreadable_literal,
    clippy::unsafe_derive_deserialize,
    clippy::needless_lifetimes
)]

#[cfg(cpuid)]
use std::convert::TryFrom;

/// Error types.
mod errors;
pub use errors::*;

/// Register bit fields.
mod registers;
pub use registers::*;

/// Leaf structs.
mod leaves;
pub use leaves::*;

/// Indexing implementations.
mod indexing;
pub use indexing::*;

use super::{CpuidEntry, CpuidKey, IndexLeaf, IndexLeafMut, RawCpuid, RawKvmCpuidEntry, Supports};

/// Macro to log warnings on unchecked leaves when validating support.
macro_rules! warn_support {
    ($($x:literal),*) => {
        $(
            log::warn!("Could not validate support for Intel CPUID leaf {}.",$x);
        )*

    }
}

// -------------------------------------------------------------------------------------------------
// Intel cpuid structure
// -------------------------------------------------------------------------------------------------

/// A structure matching supporting the  Intel CPUID specification as described in
/// [Intel® 64 and IA-32 Architectures Software Developer's Manual Combined Volumes 2A, 2B, 2C, and 2D: Instruction Set Reference, A-Z](https://cdrdv2.intel.com/v1/dl/getContent/671110)
/// .
#[derive(Debug, Clone, Eq, PartialEq, construct::Inline)]
pub struct IntelCpuid(pub std::collections::BTreeMap<CpuidKey, CpuidEntry>);

impl IntelCpuid {
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
        self.0.get(key)
    }
    /// Gets a given sub-leaf.
    pub fn get_mut(&mut self, key: &CpuidKey) -> Option<&mut CpuidEntry> {
        self.0.get_mut(key)
    }

    /// Returns the CPUID manufacturers ID. E.g. `GenuineIntel` or `AuthenticAMD`.
    #[must_use]
    pub fn manufacturer_id(&self) -> Option<[u8; 12]> {
        let leaf_0 = self.leaf::<0x0>()?;

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
    /// Applies required modifications to CPUID respective of a vCPU.
    ///
    /// # Errors
    ///
    /// When failing to set:
    /// - Feature information leaf.
    /// - Deterministic cache leaf
    /// - Extended topology leaf
    #[cfg(cpuid)]
    #[allow(clippy::too_many_lines)]
    pub fn normalize(
        &mut self,
        // The index of the current logical CPU in the range [0..cpu_count].
        cpu_index: u8,
        // The total number of logical CPUs.
        cpu_count: u8,
        // The number of bits needed to enumerate logical CPUs per core.
        cpu_bits: u8,
    ) -> Result<(), NormalizeCpuid> {
        let cpus_per_core = 1 << cpu_bits;

        // Update feature information entry
        {
            /// Flush a cache line size.
            const EBX_CLFLUSH_CACHELINE: u32 = 8;

            /// The maximum number of logical processors per package is computed as the closest
            /// power of 2 higher or equal to the CPU count configured by the user.
            const fn get_max_cpus_per_package(
                cpu_count: u8,
            ) -> Result<u8, GetMaxCpusPerPackageError> {
                // This match is better than but approximately equivalent to
                // `2.pow((cpu_count as f32).log2().ceil() as u8)` (`2^ceil(log_2(c))`).
                match cpu_count {
                    0 => Err(GetMaxCpusPerPackageError::Underflow),
                    1 => Ok(1),
                    2 => Ok(2),
                    3..=4 => Ok(4),
                    5..=8 => Ok(8),
                    9..=16 => Ok(16),
                    17..=32 => Ok(32),
                    33..=64 => Ok(64),
                    65..=128 => Ok(128),
                    129..=u8::MAX => Err(GetMaxCpusPerPackageError::Overflow),
                }
            }

            let leaf_1: &mut Leaf1 = self
                .leaf_mut::<0x1>()
                .ok_or(FeatureInformationError::MissingLeaf1)?;

            // X86 hypervisor feature
            leaf_1.ecx.tsc_deadline_mut().on();
            // Hypervisor bit
            leaf_1.ecx.bit_mut::<31>().on();

            leaf_1
                .ebx
                .initial_apic_id_mut()
                .checked_assign(u32::from(cpu_index))
                .map_err(FeatureInformationError::InitialApicId)?;
            leaf_1
                .ebx
                .clflush_mut()
                .checked_assign(EBX_CLFLUSH_CACHELINE)
                .map_err(FeatureInformationError::Clflush)?;
            let max_cpus_per_package = u32::from(
                get_max_cpus_per_package(cpu_count)
                    .map_err(FeatureInformationError::GetMaxCpusPerPackage)?,
            );
            leaf_1
                .ebx
                .max_addressable_logical_processor_ids_mut()
                .checked_assign(max_cpus_per_package)
                .map_err(FeatureInformationError::SetMaxCpusPerPackage)?;

            // A value of 1 for HTT indicates the value in CPUID.1.EBX[23:16]
            // (the Maximum number of addressable IDs for logical processors in this package)
            // is valid for the package
            leaf_1.edx.htt_mut().set(cpu_count > 1);
        }

        // Update deterministic cache entry
        {
            let leaf_4: Leaf4Mut = self.leaf_mut::<0x4>();
            for subleaf in leaf_4.0 {
                match u32::from(&subleaf.eax.cache_level()) {
                    // L1 & L2 Cache
                    // The L1 & L2 cache is shared by at most 2 hyperthreads
                    1 | 2 => subleaf
                        .eax
                        .max_num_addressable_ids_for_logical_processors_sharing_this_cache_mut()
                        .checked_assign(u32::from(cpus_per_core - 1))
                        .map_err(DeterministicCacheError::MaxCpusPerCore)?,
                    // L3 Cache
                    // The L3 cache is shared among all the logical threads
                    3 => subleaf
                        .eax
                        .max_num_addressable_ids_for_logical_processors_sharing_this_cache_mut()
                        .checked_assign(u32::from(cpu_count - 1))
                        .map_err(DeterministicCacheError::MaxCpusPerCore)?,
                    _ => (),
                }
                // Put all the cores in the same socket
                subleaf
                    .eax
                    .max_num_addressable_ids_for_processor_cores_in_physical_package_mut()
                    .checked_assign(u32::from(cpu_count / cpus_per_core) - 1)
                    .map_err(DeterministicCacheError::MaxCorePerPackage)?;
            }
        }

        // Update power management entry
        {
            let leaf_6: &mut Leaf6 = self.leaf_mut::<0x6>().ok_or(NormalizeCpuid::MissingLeaf6)?;
            leaf_6.eax.intel_turbo_boost_technology_mut().off();
            // Clear X86 EPB feature. No frequency selection in the hypervisor.
            leaf_6.ecx.performance_energy_bias_mut().off();
        }

        // Update performance monitoring entry
        {
            let leaf_a: &mut LeafA = self.leaf_mut::<0xA>().ok_or(NormalizeCpuid::MissingLeafA)?;
            *leaf_a = LeafA::from((
                LeafAEax::from(0),
                LeafAEbx::from(0),
                LeafAEcx::from(0),
                LeafAEdx::from(0),
            ));
        }

        // Update extended topology entry
        #[allow(clippy::doc_markdown)]
        {
            /// Level type used for setting thread level processor topology.
            pub const LEVEL_TYPE_THREAD: u32 = 1;
            /// Level type used for setting core level processor topology.
            pub const LEVEL_TYPE_CORE: u32 = 2;
            /// The APIC ID shift in leaf 0xBh specifies the number of bits to shit the x2APIC ID to
            /// get a unique topology of the next level. This allows 128 logical
            /// processors/package.
            const LEAFBH_INDEX1_APICID: u32 = 7;

            let leaf_b: LeafBMut = self.leaf_mut::<0xB>();
            for (index, subleaf) in leaf_b.0.into_iter().enumerate() {
                // reset eax, ebx, ecx
                subleaf.eax.0 = 0;
                subleaf.ebx.0 = 0;
                subleaf.ecx.0 = 0;
                // EDX bits 31..0 contain x2APIC ID of current logical processor
                // x2APIC increases the size of the APIC ID from 8 bits to 32 bits
                subleaf.edx.0 = u32::from(cpu_index);

                // "If SMT is not present in a processor implementation but CPUID leaf 0BH is
                // supported, CPUID.EAX=0BH, ECX=0 will return EAX = 0, EBX = 1 and
                // level type = 1. Number of logical processors at the core level is
                // reported at level type = 2." (Intel® 64 Architecture x2APIC
                // Specification, Ch. 2.8)
                match index {
                    // Thread Level Topology; index = 0
                    0 => {
                        // To get the next level APIC ID, shift right with at most 1 because we have
                        // maximum 2 hyperthreads per core that can be represented by 1 bit.
                        subleaf
                            .eax
                            .bit_shifts_right_2x_apic_id_unique_topology_id_mut()
                            .checked_assign(u32::from(cpu_bits))
                            .map_err(ExtendedTopologyError::ApicId)?;
                        // When cpu_count == 1 or HT is disabled, there is 1 logical core at this
                        // level Otherwise there are 2
                        subleaf
                            .ebx
                            .logical_processors_mut()
                            .checked_assign(u32::from(cpus_per_core))
                            .map_err(ExtendedTopologyError::LogicalProcessors)?;

                        subleaf
                            .ecx
                            .level_type_mut()
                            .checked_assign(LEVEL_TYPE_THREAD)
                            .map_err(ExtendedTopologyError::LevelType)?;
                    }
                    // Core Level Processor Topology; index = 1
                    1 => {
                        subleaf
                            .eax
                            .bit_shifts_right_2x_apic_id_unique_topology_id_mut()
                            .checked_assign(LEAFBH_INDEX1_APICID)
                            .map_err(ExtendedTopologyError::ApicId)?;
                        subleaf
                            .ebx
                            .logical_processors_mut()
                            .checked_assign(u32::from(cpu_count))
                            .map_err(ExtendedTopologyError::LogicalProcessors)?;
                        // We expect here as this is an extremely rare case that is unlikely to ever
                        // occur. It would require manual editing of the CPUID structure to push
                        // more than 2^32 subleaves.
                        subleaf
                            .ecx
                            .level_number_mut()
                            .checked_assign(
                                u32::try_from(index)
                                    .expect("Failed to convert sub-leaf index to u32."),
                            )
                            .map_err(ExtendedTopologyError::LevelNumber)?;
                        subleaf
                            .ecx
                            .level_type_mut()
                            .checked_assign(LEVEL_TYPE_CORE)
                            .map_err(ExtendedTopologyError::LevelType)?;
                    }
                    // Core Level Processor Topology; index >=2
                    // No other levels available; This should already be set correctly,
                    // and it is added here as a "re-enforcement" in case we run on
                    // different hardware
                    level => {
                        // We expect here as this is an extremely rare case that is unlikely to ever
                        // occur. It would require manual editing of the CPUID structure to push
                        // more than 2^32 subleaves.
                        subleaf.ecx.0 =
                            u32::try_from(level).expect("Failed to convert sub-leaf index to u32.");
                    }
                }
            }
        }

        Ok(())
    }
}

impl bit_fields::Equal for IntelCpuid {
    /// Compares `self` to `other` ignoring undefined bits.
    #[must_use]
    fn equal(&self, other: &Self) -> bool {
        self.leaf::<0x0>().equal(&other.leaf::<0x0>())
            && self.leaf::<0x01>().equal(&other.leaf::<0x01>())
            && self.leaf::<0x02>().equal(&other.leaf::<0x02>())
            && self.leaf::<0x03>().equal(&other.leaf::<0x03>())
            && self.leaf::<0x04>().equal(&other.leaf::<0x04>())
            && self.leaf::<0x05>().equal(&other.leaf::<0x05>())
            && self.leaf::<0x06>().equal(&other.leaf::<0x06>())
            && self.leaf::<0x07>().equal(&other.leaf::<0x07>())
            && self.leaf::<0x09>().equal(&other.leaf::<0x09>())
            && self.leaf::<0x0A>().equal(&other.leaf::<0x0A>())
            && self.leaf::<0x0B>().equal(&other.leaf::<0x0B>())
            && self.leaf::<0x0F>().equal(&other.leaf::<0x0F>())
            && self.leaf::<0x10>().equal(&other.leaf::<0x10>())
            && self.leaf::<0x14>().equal(&other.leaf::<0x14>())
            && self.leaf::<0x15>().equal(&other.leaf::<0x15>())
            && self.leaf::<0x16>().equal(&other.leaf::<0x16>())
            && self.leaf::<0x17>().equal(&other.leaf::<0x17>())
            && self.leaf::<0x18>().equal(&other.leaf::<0x18>())
            && self.leaf::<0x19>().equal(&other.leaf::<0x19>())
            && self.leaf::<0x1A>().equal(&other.leaf::<0x1A>())
            && self.leaf::<0x1B>().equal(&other.leaf::<0x1B>())
            && self.leaf::<0x1C>().equal(&other.leaf::<0x1C>())
            && self.leaf::<0x1F>().equal(&other.leaf::<0x1F>())
            && self.leaf::<0x20>().equal(&other.leaf::<0x20>())
            && self.leaf::<0x80000000>().equal(&other.leaf::<0x80000000>())
            && self.leaf::<0x80000001>().equal(&other.leaf::<0x80000001>())
            && self.leaf::<0x80000002>().equal(&other.leaf::<0x80000002>())
            && self.leaf::<0x80000003>().equal(&other.leaf::<0x80000003>())
            && self.leaf::<0x80000004>().equal(&other.leaf::<0x80000004>())
            && self.leaf::<0x80000005>().equal(&other.leaf::<0x80000005>())
            && self.leaf::<0x80000006>().equal(&other.leaf::<0x80000006>())
            && self.leaf::<0x80000007>().equal(&other.leaf::<0x80000007>())
            && self.leaf::<0x80000008>().equal(&other.leaf::<0x80000008>())
    }
}

/// Error type for [`<IntelCpuidNotSupported as Supports>::supports`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum IntelCpuidNotSupported {
    /// MissingLeaf0.
    #[error("MissingLeaf0.")]
    MissingLeaf0,
    /// Leaf0.
    #[error("Leaf0: {0}")]
    Leaf0(super::Leaf0NotSupported),
    /// MissingLeaf1.
    #[error("MissingLeaf1.")]
    MissingLeaf1,
    /// Leaf1.
    #[error("Leaf1: {0}")]
    Leaf1(Leaf1NotSupported),
    /// MissingLeaf5.
    #[error("MissingLeaf5.")]
    MissingLeaf5,
    /// Leaf5.
    #[error("Leaf5: {0}")]
    Leaf5(Leaf5NotSupported),
    /// MissingLeaf6.
    #[error("MissingLeaf6.")]
    MissingLeaf6,
    /// Leaf6.
    #[error("Leaf6: {0}")]
    Leaf6(Leaf6NotSupported),
    /// Leaf7
    #[error("Leaf7: {0}")]
    Leaf7(Leaf7NotSupported),
    /// MissingLeafA.
    #[error("MissingLeafA.")]
    MissingLeafA,
    /// LeafA.
    #[error("LeafA: {0}")]
    LeafA(LeafANotSupported),
    /// LeafF.
    #[error("LeafF: {0}")]
    LeafF(LeafFNotSupported),
    /// Leaf10.
    #[error("Leaf10: {0}")]
    Leaf10(Leaf10NotSupported),
    /// Leaf14.
    #[error("Leaf14: {0}")]
    Leaf14(Leaf14NotSupported),
    /// Leaf18.
    #[error("Leaf18: {0}")]
    Leaf18(Leaf18NotSupported),
    /// MissingLeaf19.
    #[error("MissingLeaf19.")]
    MissingLeaf19,
    /// Leaf19.
    #[error("Leaf19: {0}")]
    Leaf19(Leaf19NotSupported),
    /// MissingLeaf1C.
    #[error("MissingLeaf1C.")]
    MissingLeaf1C,
    /// Leaf1C.
    #[error("Leaf1C: {0}")]
    Leaf1C(Leaf1CNotSupported),
    /// MissingLeaf20.
    #[error("MissingLeaf20.")]
    MissingLeaf20,
    /// Leaf20.
    #[error("Leaf20: {0}")]
    Leaf20(Leaf20NotSupported),
    /// MissingLeaf80000000.
    #[error("MissingLeaf80000000.")]
    MissingLeaf80000000,
    /// Leaf80000000.
    #[error("Leaf80000000: {0}")]
    Leaf80000000(Leaf80000000NotSupported),
    /// MissingLeaf80000001.
    #[error("MissingLeaf80000001.")]
    MissingLeaf80000001,
    /// Leaf80000001.
    #[error("Leaf80000001: {0}")]
    Leaf80000001(Leaf80000001NotSupported),
    /// MissingLeaf80000007.
    #[error("MissingLeaf80000007.")]
    MissingLeaf80000007,
    /// Leaf80000007
    #[error("Leaf80000007: {0}")]
    Leaf80000007(Leaf80000007NotSupported),
    /// MissingLeaf80000008.
    #[error("MissingLeaf80000008.")]
    MissingLeaf80000008,
    /// Leaf80000008.
    #[error("Leaf80000008: {0}")]
    Leaf80000008(Leaf80000008NotSupported),
}

impl Supports for IntelCpuid {
    type Error = IntelCpuidNotSupported;
    /// Checks if `self` is a able to support `other`.
    ///
    /// Checks if a process from an environment with CPUID `other` could be continued in an
    /// environment with the CPUID `self`.
    fn supports(&self, other: &Self) -> Result<(), Self::Error> {
        match (self.leaf::<0x00>(), other.leaf::<0x00>()) {
            (_, None) => (),
            (None, Some(_)) => return Err(IntelCpuidNotSupported::MissingLeaf0),
            (Some(a), Some(b)) => a.supports(b).map_err(IntelCpuidNotSupported::Leaf0)?,
        }
        match (self.leaf::<0x01>(), other.leaf::<0x01>()) {
            (_, None) => (),
            (None, Some(_)) => return Err(IntelCpuidNotSupported::MissingLeaf1),
            (Some(a), Some(b)) => a.supports(b).map_err(IntelCpuidNotSupported::Leaf1)?,
        }
        match (self.leaf::<0x05>(), other.leaf::<0x05>()) {
            (_, None) => (),
            (None, Some(_)) => return Err(IntelCpuidNotSupported::MissingLeaf5),
            (Some(a), Some(b)) => a.supports(b).map_err(IntelCpuidNotSupported::Leaf5)?,
        }
        match (self.leaf::<0x06>(), other.leaf::<0x06>()) {
            (_, None) => (),
            (None, Some(_)) => return Err(IntelCpuidNotSupported::MissingLeaf6),
            (Some(a), Some(b)) => a.supports(b).map_err(IntelCpuidNotSupported::Leaf6)?,
        }
        self.leaf::<0x7>()
            .supports(&other.leaf::<0x7>())
            .map_err(IntelCpuidNotSupported::Leaf7)?;

        match (self.leaf::<0x0A>(), other.leaf::<0x0A>()) {
            (_, None) => (),
            (None, Some(_)) => return Err(IntelCpuidNotSupported::MissingLeafA),
            (Some(a), Some(b)) => a.supports(b).map_err(IntelCpuidNotSupported::LeafA)?,
        }

        self.leaf::<0x0F>()
            .supports(&other.leaf::<0x0F>())
            .map_err(IntelCpuidNotSupported::LeafF)?;

        self.leaf::<0x10>()
            .supports(&other.leaf::<0x10>())
            .map_err(IntelCpuidNotSupported::Leaf10)?;

        self.leaf::<0x14>()
            .supports(&other.leaf::<0x14>())
            .map_err(IntelCpuidNotSupported::Leaf14)?;

        self.leaf::<0x18>()
            .supports(&other.leaf::<0x18>())
            .map_err(IntelCpuidNotSupported::Leaf18)?;

        match (self.leaf::<0x19>(), other.leaf::<0x19>()) {
            (_, None) => (),
            (None, Some(_)) => return Err(IntelCpuidNotSupported::MissingLeaf19),
            (Some(a), Some(b)) => a.supports(b).map_err(IntelCpuidNotSupported::Leaf19)?,
        }
        match (self.leaf::<0x1C>(), other.leaf::<0x1C>()) {
            (_, None) => (),
            (None, Some(_)) => return Err(IntelCpuidNotSupported::MissingLeaf1C),
            (Some(a), Some(b)) => a.supports(b).map_err(IntelCpuidNotSupported::Leaf1C)?,
        }
        match (self.leaf::<0x20>(), other.leaf::<0x20>()) {
            (_, None) => (),
            (None, Some(_)) => return Err(IntelCpuidNotSupported::MissingLeaf20),
            (Some(a), Some(b)) => a.supports(b).map_err(IntelCpuidNotSupported::Leaf20)?,
        }
        match (self.leaf::<0x80000000>(), other.leaf::<0x80000000>()) {
            (_, None) => (),
            (None, Some(_)) => return Err(IntelCpuidNotSupported::MissingLeaf80000000),
            (Some(a), Some(b)) => a
                .supports(b)
                .map_err(IntelCpuidNotSupported::Leaf80000000)?,
        }
        match (self.leaf::<0x80000001>(), other.leaf::<0x80000001>()) {
            (_, None) => (),
            (None, Some(_)) => return Err(IntelCpuidNotSupported::MissingLeaf80000001),
            (Some(a), Some(b)) => a
                .supports(b)
                .map_err(IntelCpuidNotSupported::Leaf80000001)?,
        }
        match (self.leaf::<0x80000007>(), other.leaf::<0x80000007>()) {
            (_, None) => (),
            (None, Some(_)) => return Err(IntelCpuidNotSupported::MissingLeaf80000007),
            (Some(a), Some(b)) => a
                .supports(b)
                .map_err(IntelCpuidNotSupported::Leaf80000007)?,
        }
        match (self.leaf::<0x80000008>(), other.leaf::<0x80000008>()) {
            (_, None) => (),
            (None, Some(_)) => return Err(IntelCpuidNotSupported::MissingLeaf80000008),
            (Some(a), Some(b)) => a
                .supports(b)
                .map_err(IntelCpuidNotSupported::Leaf80000008)?,
        }

        #[rustfmt::skip]
        warn_support!(
            0x2,0x3,0x4,0x9,0xB,0xD,0x12,0x15,0x16,0x17,0x18,0x1A,0x1B,0x1F,0x80000002_u64,
            0x80000003_u64,0x80000004_u64,0x80000005_u64,0x80000006_u64
        );

        Ok(())
    }
}

impl From<RawCpuid> for IntelCpuid {
    fn from(raw_cpuid: RawCpuid) -> Self {
        let map = raw_cpuid
            .iter()
            .cloned()
            .map(<(CpuidKey, CpuidEntry)>::from)
            .collect();
        Self(map)
    }
}

impl From<IntelCpuid> for RawCpuid {
    fn from(intel_cpuid: IntelCpuid) -> Self {
        let entries = intel_cpuid
            .0
            .into_iter()
            .map(RawKvmCpuidEntry::from)
            .collect::<Vec<_>>();
        Self::from(entries)
    }
}
