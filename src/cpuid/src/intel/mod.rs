// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
#![allow(
    clippy::similar_names,
    clippy::module_name_repetitions,
    clippy::unreadable_literal,
    clippy::unsafe_derive_deserialize
)]
#[cfg(cpuid)]
use core::arch::x86_64::CpuidResult;
use std::cmp::{Ord, PartialOrd};
use std::convert::TryFrom;

use log_derive::{logfn, logfn_inputs};

/// Error types.
mod errors;
pub use errors::*;
/// Register bitfields.
mod registers;
pub use registers::*;
/// Leaf structs.
mod leaves;
pub use leaves::*;
/// Indexing implementations.
mod indexing;
pub use indexing::*;

use crate::{
    cascade_cpo, FeatureComparison, FeatureRelation, FixedString, Padding, RawCpuid,
    RawKvmCpuidEntry,
};

/// Cascades the `cpo()` function.
///
/// E.g. `cascade_cpo!(a,b,c,d) == cpo(cpo(cpo(a,b),c),d)`
#[macro_export]
macro_rules! cascade_cpo {
    ($($x:expr),*) => {
        {
            $crate::cascade!(Some(FeatureRelation::Equal),cpo$(,$x)*)
        }
    }
}

/// Cascades a function, e.g. `add(add(1,2),add(3,4))` can be written `cascade!(0,add,1,2,3,4)`.
#[macro_export]
macro_rules! cascade {
    ($s:expr,$f:expr,$($x:expr),*) => {
        {
            let temp = $s;
            $(
                let temp = $f(temp,$x);
            )*
            temp
        }
    }
}

/// Macro to log warnings on unchecked leaves when validating support.
macro_rules! warn_support {
    ($($x:literal),*) => {
        $(
            log::warn!("Could not validate support for Intel CPUID leaf {}.",$x);
        )*

    }
}

/// Combine Partial Ordering
#[must_use]
pub fn cpo(a: Option<FeatureRelation>, b: Option<FeatureRelation>) -> Option<FeatureRelation> {
    use FeatureRelation::{Equal, Subset, Superset};

    let (x, y) = match (a, b) {
        (Some(x), Some(y)) => (x, y),
        (_, _) => return None,
    };

    match (x, y) {
        (Equal, Equal) => Some(Equal),

        (Superset | Equal, Superset) | (Superset, Equal) => Some(Superset),

        (Subset, _) | (_, Subset) => Some(Subset),
    }
}

// -------------------------------------------------------------------------------------------------
// Intel cpuid structure
// -------------------------------------------------------------------------------------------------

/// A structure matching supporting the  Intel CPUID specification as described in
/// [Intel® 64 and IA-32 Architectures Software Developer's Manual Combined Volumes 2A, 2B, 2C, and 2D: Instruction Set Reference, A-Z](https://cdrdv2.intel.com/v1/dl/getContent/671110)
/// .
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, construct::Inline)]
pub struct IntelCpuid(Vec<CpuidEntry>);
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

    /// Returns the CPUID manufacturers ID. E.g. `GenuineIntel` or `AuthenticAMD`.
    #[must_use]
    pub fn manufacturer_id(&self) -> Result<FixedString<12>, MissingLeaf0> {
        let leaf0: &Leaf0 = self.leaf::<0x0>().ok_or(MissingLeaf0)?;
        Ok(FixedString::from((
            leaf0.ebx.clone(),
            leaf0.ecx.clone(),
            leaf0.edx.clone(),
        )))
    }
    /// Applies `vm_spec` to `self`.
    ///
    /// # Errors
    ///
    /// When failing to set:
    /// - Feature infomation leaf.
    /// - Deterministic cache leaf
    /// - Extended topology leaf
    #[cfg(cpuid)]
    #[allow(clippy::too_many_lines)]
    pub fn apply_vm_spec(&mut self, vm_spec: &crate::VmSpec) -> Result<(), ApplyVmSpecError> {
        // Update feature infomation entry
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
                .ok_or(FeatireInfomationError::MissingLeaf1)?;

            // X86 hypervisor feature
            leaf_1.ecx.tsc_deadline_mut().on();
            // Hypervisor bit
            leaf_1.ecx.bit_mut::<31>().on();

            leaf_1
                .ebx
                .initial_apic_id_mut()
                .checked_assign(u32::from(vm_spec.cpu_index))
                .map_err(FeatireInfomationError::InitialApicId)?;
            leaf_1
                .ebx
                .clflush_mut()
                .checked_assign(EBX_CLFLUSH_CACHELINE)
                .map_err(FeatireInfomationError::Clflush)?;
            let max_cpus_per_package = u32::from(
                get_max_cpus_per_package(vm_spec.cpu_count)
                    .map_err(FeatireInfomationError::GetMaxCpusPerPackage)?,
            );
            leaf_1
                .ebx
                .max_addressable_logical_processor_ids_mut()
                .checked_assign(max_cpus_per_package)
                .map_err(FeatireInfomationError::SetMaxCpusPerPackage)?;

            // A value of 1 for HTT indicates the value in CPUID.1.EBX[23:16]
            // (the Maximum number of addressable IDs for logical processors in this package)
            // is valid for the package
            leaf_1.edx.htt_mut().set(vm_spec.cpu_count > 1);
        }

        // Update deterministic cache entry
        {
            let leaf_4: Leaf4Mut = self.leaf_mut::<0x4>();
            for subleaf in leaf_4.0.iter_mut() {
                match u32::from(&subleaf.eax.cache_level()) {
                    // L1 & L2 Cache
                    // The L1 & L2 cache is shared by at most 2 hyperthreads
                    1 | 2 => subleaf
                        .eax
                        .max_num_addressable_ids_for_logical_processors_sharing_this_cache_mut()
                        .checked_assign(u32::from(vm_spec.cpus_per_core() - 1))
                        .map_err(DeterministicCacheError::MaxCpusPerCore)?,
                    // L3 Cache
                    // The L3 cache is shared among all the logical threads
                    3 => subleaf
                        .eax
                        .max_num_addressable_ids_for_logical_processors_sharing_this_cache_mut()
                        .checked_assign(u32::from(vm_spec.cpu_count - 1))
                        .map_err(DeterministicCacheError::MaxCpusPerCore)?,
                    _ => (),
                }
                // Put all the cores in the same socket
                subleaf
                    .eax
                    .max_num_addressable_ids_for_processor_cores_in_physical_package_mut()
                    .checked_assign(u32::from(vm_spec.cpu_count / vm_spec.cpus_per_core()) - 1)
                    .map_err(DeterministicCacheError::MaxCorePerPackage)?;
            }
        }

        // Update power management entry
        {
            let leaf_6: &mut Leaf6 = self
                .leaf_mut::<0x6>()
                .ok_or(ApplyVmSpecError::MissingLeaf6)?;
            leaf_6.eax.intel_turbo_boost_technology_mut().off();
            // Clear X86 EPB feature. No frequency selection in the hypervisor.
            leaf_6.ecx.performance_energy_bias_mut().off();
        }

        // Update performance monitoring entry
        {
            let leaf_a: &mut LeafA = self
                .leaf_mut::<0xA>()
                .ok_or(ApplyVmSpecError::MissingLeafA)?;
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
            for (index, subleaf) in leaf_b.0.iter_mut().enumerate() {
                // reset eax, ebx, ecx
                subleaf.eax.0 = 0;
                subleaf.ebx.0 = 0;
                subleaf.ecx.0 = 0;
                // EDX bits 31..0 contain x2APIC ID of current logical processor
                // x2APIC increases the size of the APIC ID from 8 bits to 32 bits
                subleaf.edx.0 = u32::from(vm_spec.cpu_index);

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
                            .checked_assign(u32::from(vm_spec.cpu_bits))
                            .map_err(ExtendedTopologyError::ApicId)?;
                        // When cpu_count == 1 or HT is disabled, there is 1 logical core at this
                        // level Otherwise there are 2
                        subleaf
                            .ebx
                            .logical_processors_mut()
                            .checked_assign(u32::from(vm_spec.cpus_per_core()))
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
                            .checked_assign(u32::from(vm_spec.cpu_count))
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
impl std::ops::IndexMut<usize> for IntelCpuid {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}
impl std::ops::Index<usize> for IntelCpuid {
    type Output = CpuidEntry;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
impl FeatureComparison for IntelCpuid {
    /// Checks if `self` is a able to support `other`.
    ///
    /// Checks if a process from an environment with CPUID `other` could be continued in an
    /// environment with the CPUID `self`.
    #[logfn(Trace)]
    #[logfn_inputs(Trace)]
    fn feature_cmp(&self, other: &Self) -> Option<FeatureRelation> {
        let a = self.leaf::<0x00>().feature_cmp(&other.leaf::<0x00>());
        let b = self.leaf::<0x01>().feature_cmp(&other.leaf::<0x01>());
        let c = self.leaf::<0x05>().feature_cmp(&other.leaf::<0x05>());
        let d = self.leaf::<0x06>().feature_cmp(&other.leaf::<0x06>());
        let e = self.leaf::<0x07>().feature_cmp(&other.leaf::<0x07>());
        let f = self.leaf::<0x0A>().feature_cmp(&other.leaf::<0x0A>());
        let g = self.leaf::<0x0F>().feature_cmp(&other.leaf::<0x0F>());
        let h = self.leaf::<0x10>().feature_cmp(&other.leaf::<0x10>());
        let i = self.leaf::<0x14>().feature_cmp(&other.leaf::<0x14>());
        let j = self.leaf::<0x18>().feature_cmp(&other.leaf::<0x18>());
        let k = self.leaf::<0x19>().feature_cmp(&other.leaf::<0x19>());
        let l = self.leaf::<0x1C>().feature_cmp(&other.leaf::<0x1C>());
        let m = self.leaf::<0x20>().feature_cmp(&other.leaf::<0x20>());
        let n = self
            .leaf::<0x80000000>()
            .feature_cmp(&other.leaf::<0x80000000>());
        let o = self
            .leaf::<0x80000001>()
            .feature_cmp(&other.leaf::<0x80000001>());
        let p = self
            .leaf::<0x80000007>()
            .feature_cmp(&other.leaf::<0x80000007>());
        let q = self
            .leaf::<0x80000008>()
            .feature_cmp(&other.leaf::<0x80000008>());

        let rtn = cascade_cpo!(a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q);
        // let rtn = cascade_cpo!(a ,b, e);

        #[rustfmt::skip]
        warn_support!(
            0x2,0x3,0x4,0x9,0xB,0xD,0x12,0x15,0x16,0x17,0x18,0x1A,0x1B,0x1F,0x80000002_u64,
            0x80000003_u64,0x80000004_u64,0x80000005_u64,0x80000006_u64
        );

        rtn
    }
}
impl From<RawCpuid> for IntelCpuid {
    fn from(raw_cpuid: RawCpuid) -> Self {
        let mut vec = raw_cpuid
            .into_iter()
            .map(|entry| CpuidEntry::from(entry.clone()))
            .collect::<Vec<_>>();
        // To enable fast access we sort the underlying entries.
        vec.sort();
        Self(vec)
    }
}
impl TryFrom<IntelCpuid> for RawCpuid {
    type Error = UnknownLeaf;
    fn try_from(intel_cpuid: IntelCpuid) -> Result<Self, Self::Error> {
        let entries = intel_cpuid
            .0
            .into_iter()
            .map(|leaf| RawKvmCpuidEntry::try_from(leaf))
            .collect::<Result<Vec<RawKvmCpuidEntry>, UnknownLeaf>>()?;
        Ok(Self::from(entries))
    }
}

// -------------------------------------------------------------------------------------------------
// Intel cpuid leaves
// -------------------------------------------------------------------------------------------------

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
impl From<CpuidResult> for CCpuidResult {
    fn from(CpuidResult { eax, ebx, ecx, edx }: CpuidResult) -> Self {
        Self { eax, ebx, ecx, edx }
    }
}
/// CPUID entry infomation stored for each leaf of [`IntelCpuid`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, construct::Inline)]
pub struct CpuidEntry {
    leaf: u32,
    subleaf: u32,
    result: CCpuidResult,
}
impl CpuidEntry {
    /// Equivalent to [`CpuidEntry::cmp_subleaf`] with a `subleaf` of 0.
    pub fn cmp_leaf(&self, leaf: u32) -> std::cmp::Ordering {
        self.cmp_subleaf(leaf, 0)
    }
    /// Compares `self.leaf` and `self.subleaf` to a given `leaf` and `subleaf` returning an ordering.
    pub fn cmp_subleaf(&self, leaf: u32, subleaf: u32) -> std::cmp::Ordering {
        self.leaf.cmp(&leaf).then(self.subleaf.cmp(&subleaf))
    }
}
impl std::cmp::PartialOrd for CpuidEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            self.leaf
                .cmp(&other.leaf)
                .then(self.subleaf.cmp(&other.subleaf)),
        )
    }
}
impl std::cmp::Ord for CpuidEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(&other).unwrap()
    }
}

/// Unknown CPUID leaf.
#[derive(Debug, thiserror::Error)]
#[error("Unknown CPUID leaf.")]
pub struct UnknownLeaf;

impl TryFrom<CpuidEntry> for RawKvmCpuidEntry {
    type Error = UnknownLeaf;
    fn try_from(
        CpuidEntry {
            leaf,
            subleaf,
            result,
        }: CpuidEntry,
    ) -> Result<Self, Self::Error> {
        let CCpuidResult { eax, ebx, ecx, edx } = result;
        Ok(Self {
            function: leaf,
            index: subleaf,
            flags: *KVM_CPUID_LEAF_FLAGS.get(&leaf).ok_or(UnknownLeaf)?,
            eax,
            ebx,
            ecx,
            edx,
            padding: Padding::default(),
        })
    }
}
impl From<RawKvmCpuidEntry> for CpuidEntry {
    fn from(
        RawKvmCpuidEntry {
            function,
            index,
            eax,
            ebx,
            ecx,
            edx,
            ..
        }: RawKvmCpuidEntry,
    ) -> Self {
        Self {
            leaf: function,
            subleaf: index,
            result: CCpuidResult { eax, ebx, ecx, edx },
        }
    }
}
