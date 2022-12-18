// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
#![allow(clippy::restriction)]

#[cfg(cpuid)]
use crate::bit_helper::BitHelper;

macro_rules! bit_range {
    ($msb_index:expr, $lsb_index:expr) => {
        crate::bit_helper::BitRange {
            msb_index: $msb_index,
            lsb_index: $lsb_index,
        }
    };
}

// Basic CPUID Information
#[allow(clippy::doc_markdown, dead_code)]
mod leaf_0x1 {
    pub const LEAF_NUM: u32 = 0x1;

    pub mod eax {
        use crate::bit_helper::BitRange;

        pub const EXTENDED_FAMILY_ID_BITRANGE: BitRange = bit_range!(27, 20);
        pub const EXTENDED_PROCESSOR_MODEL_BITRANGE: BitRange = bit_range!(19, 16);
        pub const PROCESSOR_TYPE_BITRANGE: BitRange = bit_range!(13, 12);
        pub const PROCESSOR_FAMILY_BITRANGE: BitRange = bit_range!(11, 8);
        pub const PROCESSOR_MODEL_BITRANGE: BitRange = bit_range!(7, 4);
        pub const STEPPING_BITRANGE: BitRange = bit_range!(3, 0);
    }

    pub mod ebx {
        use crate::bit_helper::BitRange;

        /// The bit-range containing the (fixed) default APIC ID.
        pub const APICID_BITRANGE: BitRange = bit_range!(31, 24);
        /// The bit-range containing the logical processor count.
        pub const CPU_COUNT_BITRANGE: BitRange = bit_range!(23, 16);
        /// The bit-range containing the number of bytes flushed when executing CLFLUSH.
        pub const CLFLUSH_SIZE_BITRANGE: BitRange = bit_range!(15, 8);
    }

    pub mod ecx {
        /// DTES64 = 64-bit debug store
        pub const DTES64_BITINDEX: u32 = 2;
        /// MONITOR = Monitor/MWAIT
        pub const MONITOR_BITINDEX: u32 = 3;
        /// CPL Qualified Debug Store
        pub const DS_CPL_SHIFT: u32 = 4;
        /// Virtual Machine Extensions
        pub const VMX_BITINDEX: u32 = 5;
        /// 6 = SMX (Safer Mode Extensions)
        pub const SMX_BITINDEX: u32 = 6;
        /// 7 = EIST (Enhanced Intel SpeedStep® technology)
        pub const EIST_BITINDEX: u32 = 7;
        /// TM2 = Thermal Monitor 2
        pub const TM2_BITINDEX: u32 = 8;
        /// CNXT_ID = L1 Context ID (L1 data cache can be set to adaptive/shared mode)
        pub const CNXT_ID_BITINDEX: u32 = 10;
        /// SDBG (cpu supports IA32_DEBUG_INTERFACE MSR for silicon debug)
        pub const SDBG_BITINDEX: u32 = 11;
        pub const FMA_BITINDEX: u32 = 12;
        /// XTPR_UPDATE = xTPR Update Control
        pub const XTPR_UPDATE_BITINDEX: u32 = 14;
        /// PDCM = Perfmon and Debug Capability
        pub const PDCM_BITINDEX: u32 = 15;
        /// 18 = DCA Direct Cache Access (prefetch data from a memory mapped device)
        pub const DCA_BITINDEX: u32 = 18;
        pub const MOVBE_BITINDEX: u32 = 22;
        pub const TSC_DEADLINE_TIMER_BITINDEX: u32 = 24;
        pub const OSXSAVE_BITINDEX: u32 = 27;
        /// Cpu is running on a hypervisor.
        pub const HYPERVISOR_BITINDEX: u32 = 31;
    }

    pub mod edx {
        /// Memory Check Exception
        pub const MCE_BITINDEX: u32 = 7;
        /// Memory Type Range Registers
        pub const MTRR_BITINDEX: u32 = 12;
        /// Processor Serial Number
        pub const PSN_BITINDEX: u32 = 18;
        /// SSE 4.2
        pub const SSE42_BITINDEX: u32 = 20;
        /// Debug Store.
        pub const DS_BITINDEX: u32 = 21;
        /// Thermal Monitor and Software Controlled Clock Facilities.
        pub const ACPI_BITINDEX: u32 = 22;
        /// Self Snoop
        pub const SS_BITINDEX: u32 = 27;
        /// Max APIC IDs reserved field is valid
        pub const HTT_BITINDEX: u32 = 28;
    }
}

// Structured Extended Feature Flags Enumeration Leaf
pub mod leaf_0x7 {
    pub const LEAF_NUM: u32 = 0x7;

    pub mod index0 {
        pub mod ebx {
            // 1 = TSC_ADJUST
            pub const SGX_BITINDEX: u32 = 2;
            pub const BMI1_BITINDEX: u32 = 3;
            pub const HLE_BITINDEX: u32 = 4;
            pub const AVX2_BITINDEX: u32 = 5;
            // FPU Data Pointer updated only on x87 exceptions if 1.
            pub const FPDP_BITINDEX: u32 = 6;
            // 7 = SMEP (Supervisor-Mode Execution Prevention if 1)
            pub const BMI2_BITINDEX: u32 = 8;
            // 9 = Enhanced REP MOVSB/STOSB if 1
            // 10 = INVPCID
            pub const INVPCID_BITINDEX: u32 = 10;
            pub const RTM_BITINDEX: u32 = 11;
            // Intel® Resource Director Technology (Intel® RDT) Monitoring
            pub const RDT_M_BITINDEX: u32 = 12;
            // 13 = Deprecates FPU CS and FPU DS values if 1
            pub const FPU_CS_DS_DEPRECATE_BITINDEX: u32 = 13;
            // Memory Protection Extensions
            pub const MPX_BITINDEX: u32 = 14;
            // RDT = Intel® Resource Director Technology
            pub const RDT_A_BITINDEX: u32 = 15;
            // AVX-512 Foundation instructions
            pub const AVX512F_BITINDEX: u32 = 16;
            // AVX-512 Doubleword and Quadword Instructions
            pub const AVX512DQ_BITINDEX: u32 = 17;
            pub const RDSEED_BITINDEX: u32 = 18;
            pub const ADX_BITINDEX: u32 = 19;
            // 20 = SMAP (Supervisor-Mode Access Prevention)
            // AVX512IFMA = AVX-512 Integer Fused Multiply-Add Instructions
            pub const AVX512IFMA_BITINDEX: u32 = 21;
            // 22 = PCOMMIT intruction
            pub const PCOMMIT_BITINDEX: u32 = 22;
            // CLFLUSHOPT (flushing multiple cache lines in parallel within a single logical
            // processor)
            pub const CLFLUSHOPT_BITINDEX: u32 = 23;
            // CLWB = Cache Line Write Back
            pub const CLWB_BITINDEX: u32 = 24;
            // PT = Intel Processor Trace
            pub const PT_BITINDEX: u32 = 25;
            // AVX512PF = AVX512 Prefetch Instructions
            pub const AVX512PF_BITINDEX: u32 = 26;
            // AVX512ER = AVX-512 Exponential and Reciprocal Instructions
            pub const AVX512ER_BITINDEX: u32 = 27;
            // AVX512CD = AVX-512 Conflict Detection Instructions
            pub const AVX512CD_BITINDEX: u32 = 28;
            // Intel Secure Hash Algorithm Extensions
            pub const SHA_BITINDEX: u32 = 29;
            // AVX-512 Byte and Word Instructions
            pub const AVX512BW_BITINDEX: u32 = 30;
            // AVX-512 Vector Length Extensions
            pub const AVX512VL_BITINDEX: u32 = 31;
        }

        pub mod ecx {
            // 0 = PREFETCHWT1 (move data closer to the processor in anticipation of future use)
            // AVX512_VBMI = AVX-512 Vector Byte Manipulation Instructions
            pub const AVX512_VBMI_BITINDEX: u32 = 1;
            // UMIP (User Mode Instruction Prevention)
            pub const UMIP_BITINDEX: u32 = 2;
            // PKU = Protection Keys for user-mode pages
            pub const PKU_BITINDEX: u32 = 3;
            // OSPKE = If 1, OS has set CR4.PKE to enable protection keys
            pub const OSPKE_BITINDEX: u32 = 4;
            // 5 = WAITPKG
            // 6 = AVX512_VBMI2
            // 7 reserved
            // 8 = GFNI
            // 9 = VAES
            // 10 = VPCLMULQDQ
            // AVX512_VNNI = Vector Neural Network Instructions
            pub const AVX512_VNNI_BITINDEX: u32 = 11;
            // 12 = AVX512_BITALG
            // 13 = TME
            // AVX512_VPOPCNTDQ = Vector population count instruction (Intel® Xeon Phi™ only.)
            pub const AVX512_VPOPCNTDQ_BITINDEX: u32 = 14;
            // LA57 = 5-level page tables.
            pub const LA57: u32 = 16;
            // 21 - 17 = The value of MAWAU used by the BNDLDX and BNDSTX instructions in 64-bit
            // mode. Read Processor ID
            pub const RDPID_BITINDEX: u32 = 22;
            // 23 - 29 reserved
            // SGX_LC = SGX Launch Configuration
            pub const SGX_LC_BITINDEX: u32 = 30;
            // 31 reserved
        }

        pub mod edx {
            // AVX-512 4-register Neural Network Instructions
            pub const AVX512_4VNNIW_BITINDEX: u32 = 2;
            // AVX-512 4-register Multiply Accumulation Single precision
            pub const AVX512_4FMAPS_BITINDEX: u32 = 3;
            pub const ARCH_CAPABILITIES_BITINDEX: u32 = 29;
        }
    }
}

/// Intel brand string.
pub const VENDOR_ID_INTEL: &[u8; 12] = b"GenuineIntel";
/// AMD brand string.
pub const VENDOR_ID_AMD: &[u8; 12] = b"AuthenticAMD";

/// cpuid related error.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    /// The function was called with invalid parameters.
    #[error("The function was called with invalid parameters.")]
    InvalidParameters(String),
    /// Function not supported on the current architecture.
    #[error("Function not supported on the current architecture.")]
    NotSupported,
}

/// Error type for [`get_cpuid`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum GetCpuidError {
    /// Invalid leaf.
    #[error("Un-supported leaf: {0}")]
    UnsupportedLeaf(u32),
    /// Invalid subleaf.
    #[error("Invalid subleaf: {0}")]
    InvalidSubleaf(u32),
}

/// Extract entry from the cpuid.
///
/// # Errors
///
/// - When the given `leaf` is more than `max_leaf` supported by CPUID.
/// - When the the CPUID leaf `sub-leaf` is invalid (all its register equal 0).
#[cfg(cpuid)]
pub fn get_cpuid(leaf: u32, subleaf: u32) -> Result<std::arch::x86_64::CpuidResult, GetCpuidError> {
    let max_leaf =
        // SAFETY: This is safe because the host supports the `cpuid` instruction
        unsafe { std::arch::x86_64::__get_cpuid_max(leaf & 0x8000_0000).0 };
    if leaf > max_leaf {
        return Err(GetCpuidError::UnsupportedLeaf(leaf));
    }

    // SAFETY: This is safe because the host supports the `cpuid` instruction
    let entry = unsafe { std::arch::x86_64::__cpuid_count(leaf, subleaf) };
    if entry.eax == 0 && entry.ebx == 0 && entry.ecx == 0 && entry.edx == 0 {
        return Err(GetCpuidError::InvalidSubleaf(subleaf));
    }

    Ok(entry)
}

/// Extracts the CPU vendor id from leaf 0x0.
///
/// # Errors
///
/// When CPUID leaf 0 is not supported.
#[cfg(cpuid)]
pub fn get_vendor_id_from_host() -> Result<[u8; 12], GetCpuidError> {
    // SAFETY: Always safe.
    get_cpuid(0, 0).map(|vendor_entry| unsafe {
        std::mem::transmute::<[u32; 3], [u8; 12]>([
            vendor_entry.ebx,
            vendor_entry.edx,
            vendor_entry.ecx,
        ])
    })
}

/// Error type for [`get_vendor_id_from_cpuid`].
#[cfg(cpuid)]
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Leaf 0 not found in given `CpuId`.")]
pub struct Leaf0NotFoundInCpuid;

/// Extracts the CPU vendor id from leaf 0x0.
///
/// # Errors
///
/// When CPUID leaf 0 is not supported.
#[cfg(cpuid)]
pub fn get_vendor_id_from_cpuid(
    cpuid: &kvm_bindings::CpuId,
) -> Result<[u8; 12], Leaf0NotFoundInCpuid> {
    // Search for vendor id entry.
    let entry_opt = cpuid
        .as_slice()
        .iter()
        .find(|entry| entry.function == 0 && entry.index == 0);
    match entry_opt {
        Some(entry) => {
            let cpu_vendor_id: [u8; 12] =
            // SAFETY: This is safe because the resulting type has a lower alignment requirement
                unsafe { std::mem::transmute([entry.ebx, entry.edx, entry.ecx]) };
            Ok(cpu_vendor_id)
        }
        None => Err(Leaf0NotFoundInCpuid),
    }
}

/// Validates that the provided CPUID belongs to a CPU of the same
/// model as the host's.
#[cfg(cpuid)]
#[must_use]
pub fn is_same_model(cpuid: &kvm_bindings::CpuId) -> bool {
    // Try to get the vendor IDs from the host and the CPUID struct.
    if let (Ok(host_vendor_id), Ok(cpuid_vendor_id)) =
        (get_vendor_id_from_host(), get_vendor_id_from_cpuid(cpuid))
    {
        // If the vendor IDs aren't the same, the CPUs are not identical.
        if host_vendor_id != cpuid_vendor_id {
            return false;
        }
    } else {
        // This only fails when CPUID is not supported, in which case
        // we can't tell if the CPUs are identical.
        return false;
    }

    // Try to get the feature information leaf from the host CPUID.
    let host_feature_info_leaf = get_cpuid(leaf_0x1::LEAF_NUM, 0);

    // The relevant information for this comparison is in the EAX register.
    let host_feature_info_leaf_eax = match host_feature_info_leaf {
        Ok(leaf) => leaf.eax,
        Err(_) => {
            // If this fails, we can't tell if the CPUs are identical.
            return false;
        }
    };

    // Search for the entry for leaf0x1.
    let feature_info_leaf = cpuid
        .as_slice()
        .iter()
        .find(|entry| entry.function == leaf_0x1::LEAF_NUM);

    // The relevant information is in EAX.
    let feature_info_leaf_eax = match feature_info_leaf {
        Some(leaf) => leaf.eax,
        None => {
            // Fail fast if we can't retrieve the relevant
            // information from CPUID.
            return false;
        }
    };

    // Validate that all of these properties are the same.
    for elem in &[
        leaf_0x1::eax::EXTENDED_FAMILY_ID_BITRANGE,
        leaf_0x1::eax::EXTENDED_PROCESSOR_MODEL_BITRANGE,
        leaf_0x1::eax::PROCESSOR_FAMILY_BITRANGE,
        leaf_0x1::eax::PROCESSOR_MODEL_BITRANGE,
        leaf_0x1::eax::STEPPING_BITRANGE,
    ] {
        if feature_info_leaf_eax.read_bits_in_range(elem)
            != host_feature_info_leaf_eax.read_bits_in_range(elem)
        {
            return false;
        }
    }

    true
}

/// Returns MSRs to be saved based on CPUID features that are enabled.
///
/// # Errors
///
/// When CPUID leaf 0 is not supported.
#[cfg(cpuid)]
pub fn msrs_to_save_by_cpuid(
    cpuid: &kvm_bindings::CpuId,
) -> Result<std::collections::HashSet<u32>, Leaf0NotFoundInCpuid> {
    let vendor_id = get_vendor_id_from_cpuid(cpuid)?;
    match &vendor_id {
        VENDOR_ID_INTEL => Ok(intel_msrs_to_save_by_cpuid(cpuid)),
        // We don't have MSR-CPUID dependencies set for other vendors yet.
        _ => Ok(std::collections::HashSet::new()),
    }
}

/// Returns MSRs to be saved based on the Intel CPUID features that are enabled.
#[cfg(cpuid)]
#[must_use]
pub fn intel_msrs_to_save_by_cpuid(cpuid: &kvm_bindings::CpuId) -> std::collections::HashSet<u32> {
    /// Scans through the CPUID and determines if a feature bit is set.
    // TODO: This currently involves a linear search which would be improved
    //       when we'll refactor the cpuid crate.
    macro_rules! cpuid_is_feature_set {
        ($cpuid:ident, $leaf:expr, $index:expr, $reg:tt, $feature_bit:expr) => {{
            let mut res = false;
            for entry in $cpuid.as_slice().iter() {
                if entry.function == $leaf && entry.index == $index {
                    if entry.$reg & (1 << $feature_bit) != 0 {
                        res = true;
                        break;
                    }
                }
            }
            res
        }};
    }

    let mut msrs = std::collections::HashSet::new();

    // Macro used for easy definition of CPUID-MSR dependencies.
    macro_rules! cpuid_msr_dep {
        ($leaf:expr, $index:expr, $reg:tt, $feature_bit:expr, $msr:expr) => {
            if cpuid_is_feature_set!(cpuid, $leaf, $index, $reg, $feature_bit) {
                msrs.extend($msr)
            }
        };
    }

    // TODO: Add more dependencies.
    cpuid_msr_dep!(
        0x7,
        0,
        ebx,
        leaf_0x7::index0::ebx::MPX_BITINDEX,
        [arch_gen::x86::msr_index::MSR_IA32_BNDCFGS]
    );

    // IA32_MTRR_PHYSBASEn, IA32_MTRR_PHYSMASKn
    cpuid_msr_dep!(0x1, 0, edx, leaf_0x1::edx::MTRR_BITINDEX, 0x200..0x210);

    // Other MTRR MSRs
    cpuid_msr_dep!(
        0x1,
        0,
        edx,
        leaf_0x1::edx::MTRR_BITINDEX,
        [
            0x250, // IA32_MTRR_FIX64K_00000
            0x258, // IA32_MTRR_FIX16K_80000
            0x259, // IA32_MTRR_FIX16K_A0000
            0x268, // IA32_MTRR_FIX4K_C0000
            0x269, // IA32_MTRR_FIX4K_C8000
            0x26a, // IA32_MTRR_FIX4K_D0000
            0x26b, // IA32_MTRR_FIX4K_D8000
            0x26c, // IA32_MTRR_FIX4K_E0000
            0x26d, // IA32_MTRR_FIX4K_E8000
            0x26e, // IA32_MTRR_FIX4K_F0000
            0x26f, // IA32_MTRR_FIX4K_F8000
            0x277, // IA32_PAT
            0x2ff  // IA32_MTRR_DEF_TYPE
        ]
    );

    // MCE MSRs
    // We are saving 32 MCE banks here as this is the maximum number supported by KVM
    // and configured by default.
    // The physical number of the MCE banks depends on the CPU.
    // The number of emulated MCE banks can be configured via KVM_X86_SETUP_MCE.
    cpuid_msr_dep!(0x1, 0, edx, leaf_0x1::edx::MCE_BITINDEX, 0x400..0x480);

    msrs
}
