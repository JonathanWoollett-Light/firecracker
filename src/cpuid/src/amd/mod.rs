// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
#![warn(clippy::pedantic)]
#![allow(clippy::similar_names, clippy::module_name_repetitions)]

use super::{CpuidEntry, CpuidKey, IndexLeaf, IndexLeafMut, RawCpuid, RawKvmCpuidEntry};

/// Error type for [`AmdCpuid::normalize`].
#[cfg(cpuid)]
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
#[error("Failed to apply `VmSpec`.")]
pub struct NormalizeCpuid;

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

impl AmdCpuid {
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

    /// Applies `vm_spec` to `self`.
    ///
    /// # Errors
    ///
    /// Never.
    #[cfg(cpuid)]
    #[allow(clippy::unused_self)]
    pub fn normalize(
        &mut self,
        // The index of the current logical CPU in the range [0..cpu_count].
        _cpu_index: u8,
        // The total number of logical CPUs.
        _cpu_count: u8,
        // The number of bits needed to enumerate logical CPUs per core.
        _cpu_bits: u8,
    ) -> Result<(), NormalizeCpuid> {
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

impl From<RawCpuid> for AmdCpuid {
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
    fn from(amd_cpuid: AmdCpuid) -> Self {
        let entries = amd_cpuid
            .0
            .into_iter()
            .map(RawKvmCpuidEntry::from)
            .collect::<Vec<_>>();
        Self::from(entries)
    }
}
