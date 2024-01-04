// Copyright 2023 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

// automatically generated by tools/bindgen.sh

#![allow(
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    non_snake_case,
    clippy::as_conversions,
    clippy::undocumented_unsafe_blocks,
    missing_debug_implementations,
    clippy::tests_outside_test_module
)]

pub const MSR_EFER: u32 = 0xc0000080;
pub const MSR_STAR: u32 = 0xc0000081;
pub const MSR_LSTAR: u32 = 0xc0000082;
pub const MSR_CSTAR: u32 = 0xc0000083;
pub const MSR_SYSCALL_MASK: u32 = 0xc0000084;
pub const MSR_FS_BASE: u32 = 0xc0000100;
pub const MSR_GS_BASE: u32 = 0xc0000101;
pub const MSR_KERNEL_GS_BASE: u32 = 0xc0000102;
pub const MSR_TSC_AUX: u32 = 0xc0000103;
pub const MSR_TEST_CTRL: u32 = 0x33;
pub const MSR_TEST_CTRL_SPLIT_LOCK_DETECT_BIT: u32 = 0x1d;
pub const MSR_IA32_SPEC_CTRL: u32 = 0x48;
pub const MSR_IA32_PRED_CMD: u32 = 0x49;
pub const MSR_PPIN_CTL: u32 = 0x4e;
pub const MSR_PPIN: u32 = 0x4f;
pub const MSR_IA32_PERFCTR0: u32 = 0xc1;
pub const MSR_IA32_PERFCTR1: u32 = 0xc2;
pub const MSR_FSB_FREQ: u32 = 0xcd;
pub const MSR_PLATFORM_INFO: u32 = 0xce;
pub const MSR_PLATFORM_INFO_CPUID_FAULT_BIT: u32 = 0x1f;
pub const MSR_IA32_UMWAIT_CONTROL: u32 = 0xe1;
pub const MSR_IA32_UMWAIT_CONTROL_TIME_MASK: i32 = -4;
pub const MSR_IA32_CORE_CAPS: u32 = 0xcf;
pub const MSR_IA32_CORE_CAPS_SPLIT_LOCK_DETECT_BIT: u32 = 0x5;
pub const MSR_PKG_CST_CONFIG_CONTROL: u32 = 0xe2;
pub const MSR_MTRRcap: u32 = 0xfe;
pub const MSR_IA32_ARCH_CAPABILITIES: u32 = 0x10a;
pub const MSR_IA32_FLUSH_CMD: u32 = 0x10b;
pub const MSR_IA32_BBL_CR_CTL: u32 = 0x119;
pub const MSR_IA32_BBL_CR_CTL3: u32 = 0x11e;
pub const MSR_IA32_TSX_CTRL: u32 = 0x122;
pub const MSR_IA32_MCU_OPT_CTRL: u32 = 0x123;
pub const MSR_IA32_SYSENTER_CS: u32 = 0x174;
pub const MSR_IA32_SYSENTER_ESP: u32 = 0x175;
pub const MSR_IA32_SYSENTER_EIP: u32 = 0x176;
pub const MSR_IA32_MCG_CAP: u32 = 0x179;
pub const MSR_IA32_MCG_STATUS: u32 = 0x17a;
pub const MSR_IA32_MCG_CTL: u32 = 0x17b;
pub const MSR_IA32_MCG_EXT_CTL: u32 = 0x4d0;
pub const MSR_OFFCORE_RSP_0: u32 = 0x1a6;
pub const MSR_OFFCORE_RSP_1: u32 = 0x1a7;
pub const MSR_TURBO_RATIO_LIMIT: u32 = 0x1ad;
pub const MSR_TURBO_RATIO_LIMIT1: u32 = 0x1ae;
pub const MSR_TURBO_RATIO_LIMIT2: u32 = 0x1af;
pub const MSR_LBR_SELECT: u32 = 0x1c8;
pub const MSR_LBR_TOS: u32 = 0x1c9;
pub const MSR_IA32_POWER_CTL: u32 = 0x1fc;
pub const MSR_IA32_POWER_CTL_BIT_EE: u32 = 0x13;
pub const MSR_LBR_NHM_FROM: u32 = 0x680;
pub const MSR_LBR_NHM_TO: u32 = 0x6c0;
pub const MSR_LBR_CORE_FROM: u32 = 0x40;
pub const MSR_LBR_CORE_TO: u32 = 0x60;
pub const MSR_LBR_INFO_0: u32 = 0xdc0;
pub const MSR_ARCH_LBR_CTL: u32 = 0x14ce;
pub const MSR_ARCH_LBR_DEPTH: u32 = 0x14cf;
pub const MSR_ARCH_LBR_FROM_0: u32 = 0x1500;
pub const MSR_ARCH_LBR_TO_0: u32 = 0x1600;
pub const MSR_ARCH_LBR_INFO_0: u32 = 0x1200;
pub const MSR_IA32_PEBS_ENABLE: u32 = 0x3f1;
pub const MSR_PEBS_DATA_CFG: u32 = 0x3f2;
pub const MSR_IA32_DS_AREA: u32 = 0x600;
pub const MSR_IA32_PERF_CAPABILITIES: u32 = 0x345;
pub const MSR_PEBS_LD_LAT_THRESHOLD: u32 = 0x3f6;
pub const MSR_IA32_RTIT_CTL: u32 = 0x570;
pub const MSR_IA32_RTIT_STATUS: u32 = 0x571;
pub const MSR_IA32_RTIT_ADDR0_A: u32 = 0x580;
pub const MSR_IA32_RTIT_ADDR0_B: u32 = 0x581;
pub const MSR_IA32_RTIT_ADDR1_A: u32 = 0x582;
pub const MSR_IA32_RTIT_ADDR1_B: u32 = 0x583;
pub const MSR_IA32_RTIT_ADDR2_A: u32 = 0x584;
pub const MSR_IA32_RTIT_ADDR2_B: u32 = 0x585;
pub const MSR_IA32_RTIT_ADDR3_A: u32 = 0x586;
pub const MSR_IA32_RTIT_ADDR3_B: u32 = 0x587;
pub const MSR_IA32_RTIT_CR3_MATCH: u32 = 0x572;
pub const MSR_IA32_RTIT_OUTPUT_BASE: u32 = 0x560;
pub const MSR_IA32_RTIT_OUTPUT_MASK: u32 = 0x561;
pub const MSR_MTRRfix64K_00000: u32 = 0x250;
pub const MSR_MTRRfix16K_80000: u32 = 0x258;
pub const MSR_MTRRfix16K_A0000: u32 = 0x259;
pub const MSR_MTRRfix4K_C0000: u32 = 0x268;
pub const MSR_MTRRfix4K_C8000: u32 = 0x269;
pub const MSR_MTRRfix4K_D0000: u32 = 0x26a;
pub const MSR_MTRRfix4K_D8000: u32 = 0x26b;
pub const MSR_MTRRfix4K_E0000: u32 = 0x26c;
pub const MSR_MTRRfix4K_E8000: u32 = 0x26d;
pub const MSR_MTRRfix4K_F0000: u32 = 0x26e;
pub const MSR_MTRRfix4K_F8000: u32 = 0x26f;
pub const MSR_MTRRdefType: u32 = 0x2ff;
pub const MSR_IA32_CR_PAT: u32 = 0x277;
pub const MSR_IA32_DEBUGCTLMSR: u32 = 0x1d9;
pub const MSR_IA32_LASTBRANCHFROMIP: u32 = 0x1db;
pub const MSR_IA32_LASTBRANCHTOIP: u32 = 0x1dc;
pub const MSR_IA32_LASTINTFROMIP: u32 = 0x1dd;
pub const MSR_IA32_LASTINTTOIP: u32 = 0x1de;
pub const MSR_IA32_PASID: u32 = 0xd93;
pub const MSR_PEBS_FRONTEND: u32 = 0x3f7;
pub const MSR_IA32_MC0_CTL: u32 = 0x400;
pub const MSR_IA32_MC0_STATUS: u32 = 0x401;
pub const MSR_IA32_MC0_ADDR: u32 = 0x402;
pub const MSR_IA32_MC0_MISC: u32 = 0x403;
pub const MSR_PKG_C3_RESIDENCY: u32 = 0x3f8;
pub const MSR_PKG_C6_RESIDENCY: u32 = 0x3f9;
pub const MSR_ATOM_PKG_C6_RESIDENCY: u32 = 0x3fa;
pub const MSR_PKG_C7_RESIDENCY: u32 = 0x3fa;
pub const MSR_CORE_C3_RESIDENCY: u32 = 0x3fc;
pub const MSR_CORE_C6_RESIDENCY: u32 = 0x3fd;
pub const MSR_CORE_C7_RESIDENCY: u32 = 0x3fe;
pub const MSR_KNL_CORE_C6_RESIDENCY: u32 = 0x3ff;
pub const MSR_PKG_C2_RESIDENCY: u32 = 0x60d;
pub const MSR_PKG_C8_RESIDENCY: u32 = 0x630;
pub const MSR_PKG_C9_RESIDENCY: u32 = 0x631;
pub const MSR_PKG_C10_RESIDENCY: u32 = 0x632;
pub const MSR_PKGC3_IRTL: u32 = 0x60a;
pub const MSR_PKGC6_IRTL: u32 = 0x60b;
pub const MSR_PKGC7_IRTL: u32 = 0x60c;
pub const MSR_PKGC8_IRTL: u32 = 0x633;
pub const MSR_PKGC9_IRTL: u32 = 0x634;
pub const MSR_PKGC10_IRTL: u32 = 0x635;
pub const MSR_RAPL_POWER_UNIT: u32 = 0x606;
pub const MSR_PKG_POWER_LIMIT: u32 = 0x610;
pub const MSR_PKG_ENERGY_STATUS: u32 = 0x611;
pub const MSR_PKG_PERF_STATUS: u32 = 0x613;
pub const MSR_PKG_POWER_INFO: u32 = 0x614;
pub const MSR_DRAM_POWER_LIMIT: u32 = 0x618;
pub const MSR_DRAM_ENERGY_STATUS: u32 = 0x619;
pub const MSR_DRAM_PERF_STATUS: u32 = 0x61b;
pub const MSR_DRAM_POWER_INFO: u32 = 0x61c;
pub const MSR_PP0_POWER_LIMIT: u32 = 0x638;
pub const MSR_PP0_ENERGY_STATUS: u32 = 0x639;
pub const MSR_PP0_POLICY: u32 = 0x63a;
pub const MSR_PP0_PERF_STATUS: u32 = 0x63b;
pub const MSR_PP1_POWER_LIMIT: u32 = 0x640;
pub const MSR_PP1_ENERGY_STATUS: u32 = 0x641;
pub const MSR_PP1_POLICY: u32 = 0x642;
pub const MSR_AMD_PKG_ENERGY_STATUS: u32 = 0xc001029b;
pub const MSR_AMD_RAPL_POWER_UNIT: u32 = 0xc0010299;
pub const MSR_CONFIG_TDP_NOMINAL: u32 = 0x648;
pub const MSR_CONFIG_TDP_LEVEL_1: u32 = 0x649;
pub const MSR_CONFIG_TDP_LEVEL_2: u32 = 0x64a;
pub const MSR_CONFIG_TDP_CONTROL: u32 = 0x64b;
pub const MSR_TURBO_ACTIVATION_RATIO: u32 = 0x64c;
pub const MSR_PLATFORM_ENERGY_STATUS: u32 = 0x64d;
pub const MSR_PKG_WEIGHTED_CORE_C0_RES: u32 = 0x658;
pub const MSR_PKG_ANY_CORE_C0_RES: u32 = 0x659;
pub const MSR_PKG_ANY_GFXE_C0_RES: u32 = 0x65a;
pub const MSR_PKG_BOTH_CORE_GFXE_C0_RES: u32 = 0x65b;
pub const MSR_CORE_C1_RES: u32 = 0x660;
pub const MSR_MODULE_C6_RES_MS: u32 = 0x664;
pub const MSR_CC6_DEMOTION_POLICY_CONFIG: u32 = 0x668;
pub const MSR_MC6_DEMOTION_POLICY_CONFIG: u32 = 0x669;
pub const MSR_ATOM_CORE_RATIOS: u32 = 0x66a;
pub const MSR_ATOM_CORE_VIDS: u32 = 0x66b;
pub const MSR_ATOM_CORE_TURBO_RATIOS: u32 = 0x66c;
pub const MSR_ATOM_CORE_TURBO_VIDS: u32 = 0x66d;
pub const MSR_CORE_PERF_LIMIT_REASONS: u32 = 0x690;
pub const MSR_GFX_PERF_LIMIT_REASONS: u32 = 0x6b0;
pub const MSR_RING_PERF_LIMIT_REASONS: u32 = 0x6b1;
pub const MSR_PPERF: u32 = 0x64e;
pub const MSR_PERF_LIMIT_REASONS: u32 = 0x64f;
pub const MSR_PM_ENABLE: u32 = 0x770;
pub const MSR_HWP_CAPABILITIES: u32 = 0x771;
pub const MSR_HWP_REQUEST_PKG: u32 = 0x772;
pub const MSR_HWP_INTERRUPT: u32 = 0x773;
pub const MSR_HWP_REQUEST: u32 = 0x774;
pub const MSR_HWP_STATUS: u32 = 0x777;
pub const MSR_AMD64_MC0_MASK: u32 = 0xc0010044;
pub const MSR_IA32_MC0_CTL2: u32 = 0x280;
pub const MSR_P6_PERFCTR0: u32 = 0xc1;
pub const MSR_P6_PERFCTR1: u32 = 0xc2;
pub const MSR_P6_EVNTSEL0: u32 = 0x186;
pub const MSR_P6_EVNTSEL1: u32 = 0x187;
pub const MSR_KNC_PERFCTR0: u32 = 0x20;
pub const MSR_KNC_PERFCTR1: u32 = 0x21;
pub const MSR_KNC_EVNTSEL0: u32 = 0x28;
pub const MSR_KNC_EVNTSEL1: u32 = 0x29;
pub const MSR_IA32_PMC0: u32 = 0x4c1;
pub const MSR_RELOAD_PMC0: u32 = 0x14c1;
pub const MSR_RELOAD_FIXED_CTR0: u32 = 0x1309;
pub const MSR_AMD64_PATCH_LEVEL: u32 = 0x8b;
pub const MSR_AMD64_TSC_RATIO: u32 = 0xc0000104;
pub const MSR_AMD64_NB_CFG: u32 = 0xc001001f;
pub const MSR_AMD64_PATCH_LOADER: u32 = 0xc0010020;
pub const MSR_AMD_PERF_CTL: u32 = 0xc0010062;
pub const MSR_AMD_PERF_STATUS: u32 = 0xc0010063;
pub const MSR_AMD_PSTATE_DEF_BASE: u32 = 0xc0010064;
pub const MSR_AMD64_OSVW_ID_LENGTH: u32 = 0xc0010140;
pub const MSR_AMD64_OSVW_STATUS: u32 = 0xc0010141;
pub const MSR_AMD_PPIN_CTL: u32 = 0xc00102f0;
pub const MSR_AMD_PPIN: u32 = 0xc00102f1;
pub const MSR_AMD64_CPUID_FN_1: u32 = 0xc0011004;
pub const MSR_AMD64_LS_CFG: u32 = 0xc0011020;
pub const MSR_AMD64_DC_CFG: u32 = 0xc0011022;
pub const MSR_AMD64_DE_CFG: u32 = 0xc0011029;
pub const MSR_AMD64_DE_CFG_LFENCE_SERIALIZE_BIT: u32 = 0x1;
pub const MSR_AMD64_BU_CFG2: u32 = 0xc001102a;
pub const MSR_AMD64_IBSFETCHCTL: u32 = 0xc0011030;
pub const MSR_AMD64_IBSFETCHLINAD: u32 = 0xc0011031;
pub const MSR_AMD64_IBSFETCHPHYSAD: u32 = 0xc0011032;
pub const MSR_AMD64_IBSFETCH_REG_COUNT: u32 = 0x3;
pub const MSR_AMD64_IBSFETCH_REG_MASK: u32 = 0x7;
pub const MSR_AMD64_IBSOPCTL: u32 = 0xc0011033;
pub const MSR_AMD64_IBSOPRIP: u32 = 0xc0011034;
pub const MSR_AMD64_IBSOPDATA: u32 = 0xc0011035;
pub const MSR_AMD64_IBSOPDATA2: u32 = 0xc0011036;
pub const MSR_AMD64_IBSOPDATA3: u32 = 0xc0011037;
pub const MSR_AMD64_IBSDCLINAD: u32 = 0xc0011038;
pub const MSR_AMD64_IBSDCPHYSAD: u32 = 0xc0011039;
pub const MSR_AMD64_IBSOP_REG_COUNT: u32 = 0x7;
pub const MSR_AMD64_IBSOP_REG_MASK: u32 = 0x7f;
pub const MSR_AMD64_IBSCTL: u32 = 0xc001103a;
pub const MSR_AMD64_IBSBRTARGET: u32 = 0xc001103b;
pub const MSR_AMD64_ICIBSEXTDCTL: u32 = 0xc001103c;
pub const MSR_AMD64_IBSOPDATA4: u32 = 0xc001103d;
pub const MSR_AMD64_IBS_REG_COUNT_MAX: u32 = 0x8;
pub const MSR_AMD64_SEV_ES_GHCB: u32 = 0xc0010130;
pub const MSR_AMD64_SEV: u32 = 0xc0010131;
pub const MSR_AMD64_SEV_ENABLED_BIT: u32 = 0x0;
pub const MSR_AMD64_SEV_ES_ENABLED_BIT: u32 = 0x1;
pub const MSR_AMD64_VIRT_SPEC_CTRL: u32 = 0xc001011f;
pub const MSR_F17H_IRPERF: u32 = 0xc00000e9;
pub const MSR_ZEN2_SPECTRAL_CHICKEN: u32 = 0xc00110e3;
pub const MSR_F16H_L2I_PERF_CTL: u32 = 0xc0010230;
pub const MSR_F16H_L2I_PERF_CTR: u32 = 0xc0010231;
pub const MSR_F16H_DR1_ADDR_MASK: u32 = 0xc0011019;
pub const MSR_F16H_DR2_ADDR_MASK: u32 = 0xc001101a;
pub const MSR_F16H_DR3_ADDR_MASK: u32 = 0xc001101b;
pub const MSR_F16H_DR0_ADDR_MASK: u32 = 0xc0011027;
pub const MSR_F15H_CU_PWR_ACCUMULATOR: u32 = 0xc001007a;
pub const MSR_F15H_CU_MAX_PWR_ACCUMULATOR: u32 = 0xc001007b;
pub const MSR_F15H_PERF_CTL: u32 = 0xc0010200;
pub const MSR_F15H_PERF_CTL0: u32 = 0xc0010200;
pub const MSR_F15H_PERF_CTL1: u32 = 0xc0010202;
pub const MSR_F15H_PERF_CTL2: u32 = 0xc0010204;
pub const MSR_F15H_PERF_CTL3: u32 = 0xc0010206;
pub const MSR_F15H_PERF_CTL4: u32 = 0xc0010208;
pub const MSR_F15H_PERF_CTL5: u32 = 0xc001020a;
pub const MSR_F15H_PERF_CTR: u32 = 0xc0010201;
pub const MSR_F15H_PERF_CTR0: u32 = 0xc0010201;
pub const MSR_F15H_PERF_CTR1: u32 = 0xc0010203;
pub const MSR_F15H_PERF_CTR2: u32 = 0xc0010205;
pub const MSR_F15H_PERF_CTR3: u32 = 0xc0010207;
pub const MSR_F15H_PERF_CTR4: u32 = 0xc0010209;
pub const MSR_F15H_PERF_CTR5: u32 = 0xc001020b;
pub const MSR_F15H_NB_PERF_CTL: u32 = 0xc0010240;
pub const MSR_F15H_NB_PERF_CTR: u32 = 0xc0010241;
pub const MSR_F15H_PTSC: u32 = 0xc0010280;
pub const MSR_F15H_IC_CFG: u32 = 0xc0011021;
pub const MSR_F15H_EX_CFG: u32 = 0xc001102c;
pub const MSR_FAM10H_MMIO_CONF_BASE: u32 = 0xc0010058;
pub const MSR_FAM10H_NODE_ID: u32 = 0xc001100c;
pub const MSR_K8_TOP_MEM1: u32 = 0xc001001a;
pub const MSR_K8_TOP_MEM2: u32 = 0xc001001d;
pub const MSR_K8_SYSCFG: u32 = 0xc0010010;
pub const MSR_K8_SYSCFG_MEM_ENCRYPT_BIT: u32 = 0x17;
pub const MSR_K8_INT_PENDING_MSG: u32 = 0xc0010055;
pub const MSR_K8_TSEG_ADDR: u32 = 0xc0010112;
pub const MSR_K8_TSEG_MASK: u32 = 0xc0010113;
pub const MSR_K7_EVNTSEL0: u32 = 0xc0010000;
pub const MSR_K7_PERFCTR0: u32 = 0xc0010004;
pub const MSR_K7_EVNTSEL1: u32 = 0xc0010001;
pub const MSR_K7_PERFCTR1: u32 = 0xc0010005;
pub const MSR_K7_EVNTSEL2: u32 = 0xc0010002;
pub const MSR_K7_PERFCTR2: u32 = 0xc0010006;
pub const MSR_K7_EVNTSEL3: u32 = 0xc0010003;
pub const MSR_K7_PERFCTR3: u32 = 0xc0010007;
pub const MSR_K7_CLK_CTL: u32 = 0xc001001b;
pub const MSR_K7_HWCR: u32 = 0xc0010015;
pub const MSR_K7_HWCR_SMMLOCK_BIT: u32 = 0x0;
pub const MSR_K7_HWCR_IRPERF_EN_BIT: u32 = 0x1e;
pub const MSR_K7_FID_VID_CTL: u32 = 0xc0010041;
pub const MSR_K7_FID_VID_STATUS: u32 = 0xc0010042;
pub const MSR_K6_WHCR: u32 = 0xc0000082;
pub const MSR_K6_UWCCR: u32 = 0xc0000085;
pub const MSR_K6_EPMR: u32 = 0xc0000086;
pub const MSR_K6_PSOR: u32 = 0xc0000087;
pub const MSR_K6_PFIR: u32 = 0xc0000088;
pub const MSR_IDT_FCR1: u32 = 0x107;
pub const MSR_IDT_FCR2: u32 = 0x108;
pub const MSR_IDT_FCR3: u32 = 0x109;
pub const MSR_IDT_FCR4: u32 = 0x10a;
pub const MSR_IDT_MCR0: u32 = 0x110;
pub const MSR_IDT_MCR1: u32 = 0x111;
pub const MSR_IDT_MCR2: u32 = 0x112;
pub const MSR_IDT_MCR3: u32 = 0x113;
pub const MSR_IDT_MCR4: u32 = 0x114;
pub const MSR_IDT_MCR5: u32 = 0x115;
pub const MSR_IDT_MCR6: u32 = 0x116;
pub const MSR_IDT_MCR7: u32 = 0x117;
pub const MSR_IDT_MCR_CTRL: u32 = 0x120;
pub const MSR_VIA_FCR: u32 = 0x1107;
pub const MSR_VIA_LONGHAUL: u32 = 0x110a;
pub const MSR_VIA_RNG: u32 = 0x110b;
pub const MSR_VIA_BCR2: u32 = 0x1147;
pub const MSR_TMTA_LONGRUN_CTRL: u32 = 0x80868010;
pub const MSR_TMTA_LONGRUN_FLAGS: u32 = 0x80868011;
pub const MSR_TMTA_LRTI_READOUT: u32 = 0x80868018;
pub const MSR_TMTA_LRTI_VOLT_MHZ: u32 = 0x8086801a;
pub const MSR_IA32_P5_MC_ADDR: u32 = 0x0;
pub const MSR_IA32_P5_MC_TYPE: u32 = 0x1;
pub const MSR_IA32_TSC: u32 = 0x10;
pub const MSR_IA32_PLATFORM_ID: u32 = 0x17;
pub const MSR_IA32_EBL_CR_POWERON: u32 = 0x2a;
pub const MSR_EBC_FREQUENCY_ID: u32 = 0x2c;
pub const MSR_SMI_COUNT: u32 = 0x34;
pub const MSR_IA32_FEAT_CTL: u32 = 0x3a;
pub const MSR_IA32_TSC_ADJUST: u32 = 0x3b;
pub const MSR_IA32_BNDCFGS: u32 = 0xd90;
pub const MSR_IA32_BNDCFGS_RSVD: u32 = 0xffc;
pub const MSR_IA32_XSS: u32 = 0xda0;
pub const MSR_IA32_APICBASE: u32 = 0x1b;
pub const MSR_IA32_APICBASE_BSP: u32 = 0x100;
pub const MSR_IA32_APICBASE_ENABLE: u32 = 0x800;
pub const MSR_IA32_APICBASE_BASE: u32 = 0xfffff000;
pub const MSR_IA32_TSCDEADLINE: u32 = 0x6e0;
pub const MSR_IA32_UCODE_WRITE: u32 = 0x79;
pub const MSR_IA32_UCODE_REV: u32 = 0x8b;
pub const MSR_IA32_SMM_MONITOR_CTL: u32 = 0x9b;
pub const MSR_IA32_SMBASE: u32 = 0x9e;
pub const MSR_IA32_PERF_STATUS: u32 = 0x198;
pub const MSR_IA32_PERF_CTL: u32 = 0x199;
pub const MSR_IA32_MPERF: u32 = 0xe7;
pub const MSR_IA32_APERF: u32 = 0xe8;
pub const MSR_IA32_THERM_CONTROL: u32 = 0x19a;
pub const MSR_IA32_THERM_INTERRUPT: u32 = 0x19b;
pub const MSR_IA32_THERM_STATUS: u32 = 0x19c;
pub const MSR_THERM2_CTL: u32 = 0x19d;
pub const MSR_THERM2_CTL_TM_SELECT: u32 = 0x10000;
pub const MSR_IA32_MISC_ENABLE: u32 = 0x1a0;
pub const MSR_IA32_TEMPERATURE_TARGET: u32 = 0x1a2;
pub const MSR_MISC_FEATURE_CONTROL: u32 = 0x1a4;
pub const MSR_MISC_PWR_MGMT: u32 = 0x1aa;
pub const MSR_IA32_ENERGY_PERF_BIAS: u32 = 0x1b0;
pub const MSR_IA32_PACKAGE_THERM_STATUS: u32 = 0x1b1;
pub const MSR_IA32_PACKAGE_THERM_INTERRUPT: u32 = 0x1b2;
pub const MSR_IA32_MISC_ENABLE_FAST_STRING_BIT: u32 = 0x0;
pub const MSR_IA32_MISC_ENABLE_FAST_STRING: u32 = 0x1;
pub const MSR_IA32_MISC_ENABLE_TCC_BIT: u32 = 0x1;
pub const MSR_IA32_MISC_ENABLE_TCC: u32 = 0x2;
pub const MSR_IA32_MISC_ENABLE_EMON_BIT: u32 = 0x7;
pub const MSR_IA32_MISC_ENABLE_EMON: u32 = 0x80;
pub const MSR_IA32_MISC_ENABLE_BTS_UNAVAIL_BIT: u32 = 0xb;
pub const MSR_IA32_MISC_ENABLE_BTS_UNAVAIL: u32 = 0x800;
pub const MSR_IA32_MISC_ENABLE_PEBS_UNAVAIL_BIT: u32 = 0xc;
pub const MSR_IA32_MISC_ENABLE_PEBS_UNAVAIL: u32 = 0x1000;
pub const MSR_IA32_MISC_ENABLE_ENHANCED_SPEEDSTEP_BIT: u32 = 0x10;
pub const MSR_IA32_MISC_ENABLE_ENHANCED_SPEEDSTEP: u32 = 0x10000;
pub const MSR_IA32_MISC_ENABLE_MWAIT_BIT: u32 = 0x12;
pub const MSR_IA32_MISC_ENABLE_MWAIT: u32 = 0x40000;
pub const MSR_IA32_MISC_ENABLE_LIMIT_CPUID_BIT: u32 = 0x16;
pub const MSR_IA32_MISC_ENABLE_LIMIT_CPUID: u32 = 0x400000;
pub const MSR_IA32_MISC_ENABLE_XTPR_DISABLE_BIT: u32 = 0x17;
pub const MSR_IA32_MISC_ENABLE_XTPR_DISABLE: u32 = 0x800000;
pub const MSR_IA32_MISC_ENABLE_XD_DISABLE_BIT: u32 = 0x22;
pub const MSR_IA32_MISC_ENABLE_XD_DISABLE: u64 = 0x400000000;
pub const MSR_IA32_MISC_ENABLE_X87_COMPAT_BIT: u32 = 0x2;
pub const MSR_IA32_MISC_ENABLE_X87_COMPAT: u32 = 0x4;
pub const MSR_IA32_MISC_ENABLE_TM1_BIT: u32 = 0x3;
pub const MSR_IA32_MISC_ENABLE_TM1: u32 = 0x8;
pub const MSR_IA32_MISC_ENABLE_SPLIT_LOCK_DISABLE_BIT: u32 = 0x4;
pub const MSR_IA32_MISC_ENABLE_SPLIT_LOCK_DISABLE: u32 = 0x10;
pub const MSR_IA32_MISC_ENABLE_L3CACHE_DISABLE_BIT: u32 = 0x6;
pub const MSR_IA32_MISC_ENABLE_L3CACHE_DISABLE: u32 = 0x40;
pub const MSR_IA32_MISC_ENABLE_SUPPRESS_LOCK_BIT: u32 = 0x8;
pub const MSR_IA32_MISC_ENABLE_SUPPRESS_LOCK: u32 = 0x100;
pub const MSR_IA32_MISC_ENABLE_PREFETCH_DISABLE_BIT: u32 = 0x9;
pub const MSR_IA32_MISC_ENABLE_PREFETCH_DISABLE: u32 = 0x200;
pub const MSR_IA32_MISC_ENABLE_FERR_BIT: u32 = 0xa;
pub const MSR_IA32_MISC_ENABLE_FERR: u32 = 0x400;
pub const MSR_IA32_MISC_ENABLE_FERR_MULTIPLEX_BIT: u32 = 0xa;
pub const MSR_IA32_MISC_ENABLE_FERR_MULTIPLEX: u32 = 0x400;
pub const MSR_IA32_MISC_ENABLE_TM2_BIT: u32 = 0xd;
pub const MSR_IA32_MISC_ENABLE_TM2: u32 = 0x2000;
pub const MSR_IA32_MISC_ENABLE_ADJ_PREF_DISABLE_BIT: u32 = 0x13;
pub const MSR_IA32_MISC_ENABLE_ADJ_PREF_DISABLE: u32 = 0x80000;
pub const MSR_IA32_MISC_ENABLE_SPEEDSTEP_LOCK_BIT: u32 = 0x14;
pub const MSR_IA32_MISC_ENABLE_SPEEDSTEP_LOCK: u32 = 0x100000;
pub const MSR_IA32_MISC_ENABLE_L1D_CONTEXT_BIT: u32 = 0x18;
pub const MSR_IA32_MISC_ENABLE_L1D_CONTEXT: u32 = 0x1000000;
pub const MSR_IA32_MISC_ENABLE_DCU_PREF_DISABLE_BIT: u32 = 0x25;
pub const MSR_IA32_MISC_ENABLE_DCU_PREF_DISABLE: u64 = 0x2000000000;
pub const MSR_IA32_MISC_ENABLE_TURBO_DISABLE_BIT: u32 = 0x26;
pub const MSR_IA32_MISC_ENABLE_TURBO_DISABLE: u64 = 0x4000000000;
pub const MSR_IA32_MISC_ENABLE_IP_PREF_DISABLE_BIT: u32 = 0x27;
pub const MSR_IA32_MISC_ENABLE_IP_PREF_DISABLE: u64 = 0x8000000000;
pub const MSR_MISC_FEATURES_ENABLES: u32 = 0x140;
pub const MSR_MISC_FEATURES_ENABLES_CPUID_FAULT_BIT: u32 = 0x0;
pub const MSR_MISC_FEATURES_ENABLES_RING3MWAIT_BIT: u32 = 0x1;
pub const MSR_IA32_TSC_DEADLINE: u32 = 0x6e0;
pub const MSR_TSX_FORCE_ABORT: u32 = 0x10f;
pub const MSR_TFA_RTM_FORCE_ABORT_BIT: u32 = 0x0;
pub const MSR_IA32_MCG_EAX: u32 = 0x180;
pub const MSR_IA32_MCG_EBX: u32 = 0x181;
pub const MSR_IA32_MCG_ECX: u32 = 0x182;
pub const MSR_IA32_MCG_EDX: u32 = 0x183;
pub const MSR_IA32_MCG_ESI: u32 = 0x184;
pub const MSR_IA32_MCG_EDI: u32 = 0x185;
pub const MSR_IA32_MCG_EBP: u32 = 0x186;
pub const MSR_IA32_MCG_ESP: u32 = 0x187;
pub const MSR_IA32_MCG_EFLAGS: u32 = 0x188;
pub const MSR_IA32_MCG_EIP: u32 = 0x189;
pub const MSR_IA32_MCG_RESERVED: u32 = 0x18a;
pub const MSR_P4_BPU_PERFCTR0: u32 = 0x300;
pub const MSR_P4_BPU_PERFCTR1: u32 = 0x301;
pub const MSR_P4_BPU_PERFCTR2: u32 = 0x302;
pub const MSR_P4_BPU_PERFCTR3: u32 = 0x303;
pub const MSR_P4_MS_PERFCTR0: u32 = 0x304;
pub const MSR_P4_MS_PERFCTR1: u32 = 0x305;
pub const MSR_P4_MS_PERFCTR2: u32 = 0x306;
pub const MSR_P4_MS_PERFCTR3: u32 = 0x307;
pub const MSR_P4_FLAME_PERFCTR0: u32 = 0x308;
pub const MSR_P4_FLAME_PERFCTR1: u32 = 0x309;
pub const MSR_P4_FLAME_PERFCTR2: u32 = 0x30a;
pub const MSR_P4_FLAME_PERFCTR3: u32 = 0x30b;
pub const MSR_P4_IQ_PERFCTR0: u32 = 0x30c;
pub const MSR_P4_IQ_PERFCTR1: u32 = 0x30d;
pub const MSR_P4_IQ_PERFCTR2: u32 = 0x30e;
pub const MSR_P4_IQ_PERFCTR3: u32 = 0x30f;
pub const MSR_P4_IQ_PERFCTR4: u32 = 0x310;
pub const MSR_P4_IQ_PERFCTR5: u32 = 0x311;
pub const MSR_P4_BPU_CCCR0: u32 = 0x360;
pub const MSR_P4_BPU_CCCR1: u32 = 0x361;
pub const MSR_P4_BPU_CCCR2: u32 = 0x362;
pub const MSR_P4_BPU_CCCR3: u32 = 0x363;
pub const MSR_P4_MS_CCCR0: u32 = 0x364;
pub const MSR_P4_MS_CCCR1: u32 = 0x365;
pub const MSR_P4_MS_CCCR2: u32 = 0x366;
pub const MSR_P4_MS_CCCR3: u32 = 0x367;
pub const MSR_P4_FLAME_CCCR0: u32 = 0x368;
pub const MSR_P4_FLAME_CCCR1: u32 = 0x369;
pub const MSR_P4_FLAME_CCCR2: u32 = 0x36a;
pub const MSR_P4_FLAME_CCCR3: u32 = 0x36b;
pub const MSR_P4_IQ_CCCR0: u32 = 0x36c;
pub const MSR_P4_IQ_CCCR1: u32 = 0x36d;
pub const MSR_P4_IQ_CCCR2: u32 = 0x36e;
pub const MSR_P4_IQ_CCCR3: u32 = 0x36f;
pub const MSR_P4_IQ_CCCR4: u32 = 0x370;
pub const MSR_P4_IQ_CCCR5: u32 = 0x371;
pub const MSR_P4_ALF_ESCR0: u32 = 0x3ca;
pub const MSR_P4_ALF_ESCR1: u32 = 0x3cb;
pub const MSR_P4_BPU_ESCR0: u32 = 0x3b2;
pub const MSR_P4_BPU_ESCR1: u32 = 0x3b3;
pub const MSR_P4_BSU_ESCR0: u32 = 0x3a0;
pub const MSR_P4_BSU_ESCR1: u32 = 0x3a1;
pub const MSR_P4_CRU_ESCR0: u32 = 0x3b8;
pub const MSR_P4_CRU_ESCR1: u32 = 0x3b9;
pub const MSR_P4_CRU_ESCR2: u32 = 0x3cc;
pub const MSR_P4_CRU_ESCR3: u32 = 0x3cd;
pub const MSR_P4_CRU_ESCR4: u32 = 0x3e0;
pub const MSR_P4_CRU_ESCR5: u32 = 0x3e1;
pub const MSR_P4_DAC_ESCR0: u32 = 0x3a8;
pub const MSR_P4_DAC_ESCR1: u32 = 0x3a9;
pub const MSR_P4_FIRM_ESCR0: u32 = 0x3a4;
pub const MSR_P4_FIRM_ESCR1: u32 = 0x3a5;
pub const MSR_P4_FLAME_ESCR0: u32 = 0x3a6;
pub const MSR_P4_FLAME_ESCR1: u32 = 0x3a7;
pub const MSR_P4_FSB_ESCR0: u32 = 0x3a2;
pub const MSR_P4_FSB_ESCR1: u32 = 0x3a3;
pub const MSR_P4_IQ_ESCR0: u32 = 0x3ba;
pub const MSR_P4_IQ_ESCR1: u32 = 0x3bb;
pub const MSR_P4_IS_ESCR0: u32 = 0x3b4;
pub const MSR_P4_IS_ESCR1: u32 = 0x3b5;
pub const MSR_P4_ITLB_ESCR0: u32 = 0x3b6;
pub const MSR_P4_ITLB_ESCR1: u32 = 0x3b7;
pub const MSR_P4_IX_ESCR0: u32 = 0x3c8;
pub const MSR_P4_IX_ESCR1: u32 = 0x3c9;
pub const MSR_P4_MOB_ESCR0: u32 = 0x3aa;
pub const MSR_P4_MOB_ESCR1: u32 = 0x3ab;
pub const MSR_P4_MS_ESCR0: u32 = 0x3c0;
pub const MSR_P4_MS_ESCR1: u32 = 0x3c1;
pub const MSR_P4_PMH_ESCR0: u32 = 0x3ac;
pub const MSR_P4_PMH_ESCR1: u32 = 0x3ad;
pub const MSR_P4_RAT_ESCR0: u32 = 0x3bc;
pub const MSR_P4_RAT_ESCR1: u32 = 0x3bd;
pub const MSR_P4_SAAT_ESCR0: u32 = 0x3ae;
pub const MSR_P4_SAAT_ESCR1: u32 = 0x3af;
pub const MSR_P4_SSU_ESCR0: u32 = 0x3be;
pub const MSR_P4_SSU_ESCR1: u32 = 0x3bf;
pub const MSR_P4_TBPU_ESCR0: u32 = 0x3c2;
pub const MSR_P4_TBPU_ESCR1: u32 = 0x3c3;
pub const MSR_P4_TC_ESCR0: u32 = 0x3c4;
pub const MSR_P4_TC_ESCR1: u32 = 0x3c5;
pub const MSR_P4_U2L_ESCR0: u32 = 0x3b0;
pub const MSR_P4_U2L_ESCR1: u32 = 0x3b1;
pub const MSR_P4_PEBS_MATRIX_VERT: u32 = 0x3f2;
pub const MSR_CORE_PERF_FIXED_CTR0: u32 = 0x309;
pub const MSR_CORE_PERF_FIXED_CTR1: u32 = 0x30a;
pub const MSR_CORE_PERF_FIXED_CTR2: u32 = 0x30b;
pub const MSR_CORE_PERF_FIXED_CTR3: u32 = 0x30c;
pub const MSR_CORE_PERF_FIXED_CTR_CTRL: u32 = 0x38d;
pub const MSR_CORE_PERF_GLOBAL_STATUS: u32 = 0x38e;
pub const MSR_CORE_PERF_GLOBAL_CTRL: u32 = 0x38f;
pub const MSR_CORE_PERF_GLOBAL_OVF_CTRL: u32 = 0x390;
pub const MSR_PERF_METRICS: u32 = 0x329;
pub const MSR_CORE_PERF_GLOBAL_OVF_CTRL_TRACE_TOPA_PMI_BIT: u32 = 0x37;
pub const MSR_CORE_PERF_GLOBAL_OVF_CTRL_TRACE_TOPA_PMI: u64 = 0x80000000000000;
pub const MSR_CORE_PERF_GLOBAL_OVF_CTRL_OVF_BUF_BIT: u32 = 0x3e;
pub const MSR_CORE_PERF_GLOBAL_OVF_CTRL_OVF_BUF: u64 = 0x4000000000000000;
pub const MSR_CORE_PERF_GLOBAL_OVF_CTRL_COND_CHGD_BIT: u32 = 0x3f;
pub const MSR_CORE_PERF_GLOBAL_OVF_CTRL_COND_CHGD: i64 = -9223372036854775808;
pub const MSR_GEODE_BUSCONT_CONF0: u32 = 0x1900;
pub const MSR_IA32_VMX_BASIC: u32 = 0x480;
pub const MSR_IA32_VMX_PINBASED_CTLS: u32 = 0x481;
pub const MSR_IA32_VMX_PROCBASED_CTLS: u32 = 0x482;
pub const MSR_IA32_VMX_EXIT_CTLS: u32 = 0x483;
pub const MSR_IA32_VMX_ENTRY_CTLS: u32 = 0x484;
pub const MSR_IA32_VMX_MISC: u32 = 0x485;
pub const MSR_IA32_VMX_CR0_FIXED0: u32 = 0x486;
pub const MSR_IA32_VMX_CR0_FIXED1: u32 = 0x487;
pub const MSR_IA32_VMX_CR4_FIXED0: u32 = 0x488;
pub const MSR_IA32_VMX_CR4_FIXED1: u32 = 0x489;
pub const MSR_IA32_VMX_VMCS_ENUM: u32 = 0x48a;
pub const MSR_IA32_VMX_PROCBASED_CTLS2: u32 = 0x48b;
pub const MSR_IA32_VMX_EPT_VPID_CAP: u32 = 0x48c;
pub const MSR_IA32_VMX_TRUE_PINBASED_CTLS: u32 = 0x48d;
pub const MSR_IA32_VMX_TRUE_PROCBASED_CTLS: u32 = 0x48e;
pub const MSR_IA32_VMX_TRUE_EXIT_CTLS: u32 = 0x48f;
pub const MSR_IA32_VMX_TRUE_ENTRY_CTLS: u32 = 0x490;
pub const MSR_IA32_VMX_VMFUNC: u32 = 0x491;
pub const MSR_IA32_VMX_MISC_INTEL_PT: u32 = 0x4000;
pub const MSR_IA32_VMX_MISC_VMWRITE_SHADOW_RO_FIELDS: u32 = 0x20000000;
pub const MSR_IA32_VMX_MISC_PREEMPTION_TIMER_SCALE: u32 = 0x1f;
pub const MSR_VM_CR: u32 = 0xc0010114;
pub const MSR_VM_IGNNE: u32 = 0xc0010115;
pub const MSR_VM_HSAVE_PA: u32 = 0xc0010117;
