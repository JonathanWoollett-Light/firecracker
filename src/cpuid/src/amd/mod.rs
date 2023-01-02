// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
#![warn(clippy::pedantic)]
#![allow(
    clippy::similar_names,
    clippy::unreadable_literal,
    clippy::module_name_repetitions
)]

#[cfg(cpuid)]
use super::{CCpuidResult, KvmCpuidFlags};
use super::{CpuidEntry, CpuidKey, CpuidTrait, RawCpuid, RawKvmCpuidEntry, Supports};

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

/// A structure containing the information as described in the AMD CPUID specification as described
/// in
/// [AMD64 Architecture Programmer’s Manual Volume 3: General-Purpose and System Instructions](https://www.amd.com/system/files/TechDocs/24594.pdf)
/// .
///
/// # Notes
///
/// We not do not currently check AMD features on snapshot restore.
#[derive(Debug, Clone, Eq, PartialEq, construct::Inline)]
#[repr(C)]
pub struct AmdCpuid(pub std::collections::BTreeMap<CpuidKey, CpuidEntry>);

impl CpuidTrait for AmdCpuid {
    /// Gets a given sub-leaf.
    #[inline]
    fn get(&self, key: &CpuidKey) -> Option<&CpuidEntry> {
        self.0.get(key)
    }
    /// Gets a given sub-leaf.
    #[inline]
    fn get_mut(&mut self, key: &CpuidKey) -> Option<&mut CpuidEntry> {
        self.0.get_mut(key)
    }
}

impl AmdCpuid {
    /// We always use this brand string.
    pub const DEFAULT_BRAND_STRING: &[u8; super::BRAND_STRING_LENGTH] =
        b"AMD EPYC\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";

    /// Applies `vm_spec` to `self`.
    ///
    /// # Errors
    ///
    /// When attempting to access misisng leaves or set fields within leaves to values that don't
    /// fit.
    ///
    /// # Panics
    ///
    /// Never.
    // As we pass through host frequency, we require CPUID and thus `cfg(cpuid)`.
    #[allow(clippy::too_many_lines)]
    #[cfg(cpuid)]
    #[inline]
    pub fn normalize(
        &mut self,
        // The index of the current logical CPU in the range [0..cpu_count].
        cpu_index: u8,
        // The total number of logical CPUs.
        cpu_count: u8,
        // The number of bits needed to enumerate logical CPUs per core.
        cpu_bits: u8,
    ) -> Result<(), NormalizeCpuidError> {
        let cpus_per_core = 1u8
            .checked_shl(u32::from(cpu_bits))
            .ok_or(NormalizeCpuidError::CpuBits(cpu_bits))?;

        // Process CPUID
        {
            // Some versions of kernel may return the 0xB leaf for AMD even if this is an
            // Intel-specific leaf. Remove it.
            self.0.remove(&CpuidKey::leaf(0xB));

            // Pass-through host CPUID for leaves 0x8000001e and 0x8000001d.
            {
                // 0x8000001e
                self.0.insert(
                    CpuidKey::leaf(0x8000001e),
                    CpuidEntry {
                        flags: KvmCpuidFlags::empty(),
                        // SAFETY: Safe as `cfg(cpuid)` ensure CPUID is supported.
                        result: CCpuidResult::from(unsafe {
                            core::arch::x86_64::__cpuid(0x8000001e)
                        }),
                    },
                );

                // 0x8000001d
                for subleaf in 0.. {
                    // SAFETY: Safe as `cfg(cpuid)` ensure CPUID is supported.
                    let result = CCpuidResult::from(unsafe {
                        core::arch::x86_64::__cpuid_count(0x8000001d, subleaf)
                    });
                    if Leaf8000001dEax::from(result.eax).cache_type() == 0 {
                        break;
                    }
                    self.0.insert(
                        CpuidKey::subleaf(0x8000001d, subleaf),
                        CpuidEntry {
                            flags: KvmCpuidFlags::SIGNIFICANT_INDEX,
                            result,
                        },
                    );
                }
            }
        }

        // Update largest extended fn entry
        {
            // KVM sets the largest extended function to 0x80000000. Change it to 0x8000001f
            // Since we also use the leaf 0x8000001d (Extended Cache Topology).
            let leaf_80000000 = self
                .leaf_mut::<0x80000000>()
                .ok_or(NormalizeCpuidError::MissingLeaf0x80000000)?;
            // SAFETY: Safe, as `0x8000_001f` is within the known range.
            unsafe {
                leaf_80000000
                    .eax
                    .l_func_ext_mut()
                    .unchecked_assign(0x8000_001f);
            }
        }

        // Updated extended feature fn entry
        {
            // set the Topology Extension bit since we use the Extended Cache Topology leaf
            let leaf_80000001 = self
                .leaf_mut::<0x80000001>()
                .ok_or(NormalizeCpuidError::MissingLeaf0x80000001)?;
            leaf_80000001.ecx.topology_extensions_mut().on();
        }

        // Update AMD feature entry
        {
            // We don't support more then 128 threads right now.
            // It's safe to put them all on the same processor.
            let leaf_80000008 = self
                .leaf_mut::<0x80000008>()
                .ok_or(FeatureEntryError::MissingLeaf0x80000008)?;
            // This value allows at most 64 logical threads within a package.
            // SAFETY: `7` is within the known range and always safe.
            unsafe {
                leaf_80000008.ecx.apic_id_size_mut().unchecked_assign(7);
            }
            leaf_80000008
                .ecx
                .nt_mut()
                .checked_assign(u32::from(
                    cpu_count
                        .checked_sub(1)
                        .ok_or(FeatureEntryError::NumberOfPhysicalThreadsOverflow)?,
                ))
                .map_err(FeatureEntryError::NumberOfPhysicalThreads)?;
        }

        // Update extended cache topology entry
        {
            let leaf_8000001d: Leaf8000001dMut = self.leaf_mut::<0x8000001d>();
            for subleaf in leaf_8000001d.0 {
                match u32::from(&subleaf.eax.cache_level()) {
                    // L1 & L2 Cache
                    // The L1 & L2 cache is shared by at most 2 hyper-threads
                    1 | 2 => subleaf
                        .eax
                        .num_sharing_cache_mut()
                        // SAFETY: We know `cpus_per_core > 0` therefore this is always safe.
                        .checked_assign(u32::from(unsafe {
                            cpus_per_core.checked_sub(1).unwrap_unchecked()
                        }))
                        .map_err(ExtendedCacheTopologyError::NumSharingCache)?,
                    // L3 Cache
                    // The L3 cache is shared among all the logical threads
                    3 => subleaf
                        .eax
                        .num_sharing_cache_mut()
                        .checked_assign(u32::from(
                            cpu_count
                                .checked_sub(1)
                                .ok_or(ExtendedCacheTopologyError::NumSharingCacheOverflow)?,
                        ))
                        .map_err(ExtendedCacheTopologyError::NumSharingCache)?,
                    _ => (),
                }
            }
        }

        // Update extended apic id entry
        {
            // When hyper-threading is enabled each pair of 2 consecutive logical CPUs
            // will have the same core id since they represent 2 threads in the same core.
            // For Example:
            // logical CPU 0 -> core id: 0
            // logical CPU 1 -> core id: 0
            // logical CPU 2 -> core id: 1
            // logical CPU 3 -> core id: 1
            let core_id =
                // SAFETY: We know `cpus_per_core != 0` therefore this is always safe.
                unsafe { u32::from(cpu_index.checked_div(cpus_per_core).unwrap_unchecked()) };

            let leaf_8000001e = self
                .leaf_mut::<0x8000001e>()
                .ok_or(ExtendedApicIdError::MissingLeaf0x8000001e)?;
            leaf_8000001e
                .eax
                .extended_apic_id_mut()
                .checked_assign(u32::from(cpu_index))
                .map_err(ExtendedApicIdError::ExtendedApicId)?;

            leaf_8000001e
                .ebx
                .compute_unit_id_mut()
                .checked_assign(core_id)
                .map_err(ExtendedApicIdError::ComputeUnitId)?;
            leaf_8000001e
                .ebx
                .threads_per_compute_unit_mut()
                // SAFETY: We know `cpus_per_core > 0` therefore this is always safe.
                .checked_assign(u32::from(unsafe {
                    cpus_per_core.checked_sub(1).unwrap_unchecked()
                }))
                .map_err(ExtendedApicIdError::ThreadPerComputeUnit)?;

            // SAFETY: We know the value always fits within the range and thus is always safe.
            unsafe {
                // This value means there is 1 node per processor.
                leaf_8000001e
                    .ecx
                    .nodes_per_processor_mut()
                    .unchecked_assign(0);
                // Put all the cpus in the same node.
                leaf_8000001e.ecx.node_id_mut().unchecked_assign(0);
            }
        }

        // Update brand string entry
        self.apply_brand_string(Self::DEFAULT_BRAND_STRING)
            .map_err(NormalizeCpuidError::BrandString)?;

        Ok(())
    }
}

/// Error type for [`<AmdCpuidNotSupported as Supports>::supports`].
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
#[error("AmdCpuidNotSupported.")]
pub struct AmdCpuidNotSupported;

impl Supports for AmdCpuid {
    type Error = AmdCpuidNotSupported;
    /// Checks if `self` is a able to support `other`.
    ///
    /// Checks if a process from an environment with CPUID `other` could be continued in an
    /// environment with the CPUID `self`.
    #[inline]
    fn supports(&self, _other: &Self) -> Result<(), Self::Error> {
        // We don't currently implement support checking for AMD, therefore we allow all templates,
        // presuming support.
        Ok(())
    }
}

impl From<RawCpuid> for AmdCpuid {
    #[inline]
    fn from(raw_cpuid: RawCpuid) -> Self {
        let map = raw_cpuid
            .iter()
            .cloned()
            .map(<(CpuidKey, CpuidEntry)>::from)
            .collect();
        Self(map)
    }
}

impl From<AmdCpuid> for RawCpuid {
    #[inline]
    fn from(amd_cpuid: AmdCpuid) -> Self {
        let entries = amd_cpuid
            .0
            .into_iter()
            .map(RawKvmCpuidEntry::from)
            .collect::<Vec<_>>();
        Self::from(entries)
    }
}
