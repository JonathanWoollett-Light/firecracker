// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

// automatically generated by tools/bindgen.sh

#![allow(
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    non_snake_case
)]

pub const MPC_SIGNATURE: &[u8; 5usize] = b"PCMP\0";
pub const MP_PROCESSOR: u32 = 0;
pub const MP_BUS: u32 = 1;
pub const MP_IOAPIC: u32 = 2;
pub const MP_INTSRC: u32 = 3;
pub const MP_LINTSRC: u32 = 4;
pub const MP_TRANSLATION: u32 = 192;
pub const CPU_ENABLED: u32 = 1;
pub const CPU_BOOTPROCESSOR: u32 = 2;
pub const CPU_STEPPING_MASK: u32 = 15;
pub const CPU_MODEL_MASK: u32 = 240;
pub const CPU_FAMILY_MASK: u32 = 3840;
pub const BUSTYPE_EISA: &[u8; 5usize] = b"EISA\0";
pub const BUSTYPE_ISA: &[u8; 4usize] = b"ISA\0";
pub const BUSTYPE_INTERN: &[u8; 7usize] = b"INTERN\0";
pub const BUSTYPE_MCA: &[u8; 4usize] = b"MCA\0";
pub const BUSTYPE_VL: &[u8; 3usize] = b"VL\0";
pub const BUSTYPE_PCI: &[u8; 4usize] = b"PCI\0";
pub const BUSTYPE_PCMCIA: &[u8; 7usize] = b"PCMCIA\0";
pub const BUSTYPE_CBUS: &[u8; 5usize] = b"CBUS\0";
pub const BUSTYPE_CBUSII: &[u8; 7usize] = b"CBUSII\0";
pub const BUSTYPE_FUTURE: &[u8; 7usize] = b"FUTURE\0";
pub const BUSTYPE_MBI: &[u8; 4usize] = b"MBI\0";
pub const BUSTYPE_MBII: &[u8; 5usize] = b"MBII\0";
pub const BUSTYPE_MPI: &[u8; 4usize] = b"MPI\0";
pub const BUSTYPE_MPSA: &[u8; 5usize] = b"MPSA\0";
pub const BUSTYPE_NUBUS: &[u8; 6usize] = b"NUBUS\0";
pub const BUSTYPE_TC: &[u8; 3usize] = b"TC\0";
pub const BUSTYPE_VME: &[u8; 4usize] = b"VME\0";
pub const BUSTYPE_XPRESS: &[u8; 7usize] = b"XPRESS\0";
pub const MPC_APIC_USABLE: u32 = 1;
pub const MP_IRQPOL_DEFAULT: u32 = 0;
pub const MP_IRQPOL_ACTIVE_HIGH: u32 = 1;
pub const MP_IRQPOL_RESERVED: u32 = 2;
pub const MP_IRQPOL_ACTIVE_LOW: u32 = 3;
pub const MP_IRQPOL_MASK: u32 = 3;
pub const MP_IRQTRIG_DEFAULT: u32 = 0;
pub const MP_IRQTRIG_EDGE: u32 = 4;
pub const MP_IRQTRIG_RESERVED: u32 = 8;
pub const MP_IRQTRIG_LEVEL: u32 = 12;
pub const MP_IRQTRIG_MASK: u32 = 12;
pub const MP_APIC_ALL: u32 = 255;
pub const MPC_OEM_SIGNATURE: &[u8; 5usize] = b"_OEM\0";
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct mpf_intel {
    pub signature: [::std::os::raw::c_char; 4usize],
    pub physptr: ::std::os::raw::c_uint,
    pub length: ::std::os::raw::c_uchar,
    pub specification: ::std::os::raw::c_uchar,
    pub checksum: ::std::os::raw::c_uchar,
    pub feature1: ::std::os::raw::c_uchar,
    pub feature2: ::std::os::raw::c_uchar,
    pub feature3: ::std::os::raw::c_uchar,
    pub feature4: ::std::os::raw::c_uchar,
    pub feature5: ::std::os::raw::c_uchar,
}
#[test]
fn bindgen_test_layout_mpf_intel() {
    assert_eq!(
        ::std::mem::size_of::<mpf_intel>(),
        16usize,
        concat!("Size of: ", stringify!(mpf_intel))
    );
    assert_eq!(
        ::std::mem::align_of::<mpf_intel>(),
        4usize,
        concat!("Alignment of ", stringify!(mpf_intel))
    );
    #[tracing::instrument(level = "info", skip())]
    fn test_field_signature() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpf_intel>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).signature) as usize - ptr as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(mpf_intel),
                "::",
                stringify!(signature)
            )
        );
    }
    test_field_signature();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_physptr() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpf_intel>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).physptr) as usize - ptr as usize
            },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(mpf_intel),
                "::",
                stringify!(physptr)
            )
        );
    }
    test_field_physptr();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_length() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpf_intel>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).length) as usize - ptr as usize
            },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(mpf_intel),
                "::",
                stringify!(length)
            )
        );
    }
    test_field_length();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_specification() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpf_intel>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).specification) as usize - ptr as usize
            },
            9usize,
            concat!(
                "Offset of field: ",
                stringify!(mpf_intel),
                "::",
                stringify!(specification)
            )
        );
    }
    test_field_specification();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_checksum() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpf_intel>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).checksum) as usize - ptr as usize
            },
            10usize,
            concat!(
                "Offset of field: ",
                stringify!(mpf_intel),
                "::",
                stringify!(checksum)
            )
        );
    }
    test_field_checksum();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_feature1() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpf_intel>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).feature1) as usize - ptr as usize
            },
            11usize,
            concat!(
                "Offset of field: ",
                stringify!(mpf_intel),
                "::",
                stringify!(feature1)
            )
        );
    }
    test_field_feature1();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_feature2() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpf_intel>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).feature2) as usize - ptr as usize
            },
            12usize,
            concat!(
                "Offset of field: ",
                stringify!(mpf_intel),
                "::",
                stringify!(feature2)
            )
        );
    }
    test_field_feature2();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_feature3() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpf_intel>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).feature3) as usize - ptr as usize
            },
            13usize,
            concat!(
                "Offset of field: ",
                stringify!(mpf_intel),
                "::",
                stringify!(feature3)
            )
        );
    }
    test_field_feature3();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_feature4() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpf_intel>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).feature4) as usize - ptr as usize
            },
            14usize,
            concat!(
                "Offset of field: ",
                stringify!(mpf_intel),
                "::",
                stringify!(feature4)
            )
        );
    }
    test_field_feature4();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_feature5() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpf_intel>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).feature5) as usize - ptr as usize
            },
            15usize,
            concat!(
                "Offset of field: ",
                stringify!(mpf_intel),
                "::",
                stringify!(feature5)
            )
        );
    }
    test_field_feature5();
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct mpc_table {
    pub signature: [::std::os::raw::c_char; 4usize],
    pub length: ::std::os::raw::c_ushort,
    pub spec: ::std::os::raw::c_char,
    pub checksum: ::std::os::raw::c_char,
    pub oem: [::std::os::raw::c_char; 8usize],
    pub productid: [::std::os::raw::c_char; 12usize],
    pub oemptr: ::std::os::raw::c_uint,
    pub oemsize: ::std::os::raw::c_ushort,
    pub oemcount: ::std::os::raw::c_ushort,
    pub lapic: ::std::os::raw::c_uint,
    pub reserved: ::std::os::raw::c_uint,
}
#[test]
fn bindgen_test_layout_mpc_table() {
    assert_eq!(
        ::std::mem::size_of::<mpc_table>(),
        44usize,
        concat!("Size of: ", stringify!(mpc_table))
    );
    assert_eq!(
        ::std::mem::align_of::<mpc_table>(),
        4usize,
        concat!("Alignment of ", stringify!(mpc_table))
    );
    #[tracing::instrument(level = "info", skip())]
    fn test_field_signature() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_table>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).signature) as usize - ptr as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_table),
                "::",
                stringify!(signature)
            )
        );
    }
    test_field_signature();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_length() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_table>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).length) as usize - ptr as usize
            },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_table),
                "::",
                stringify!(length)
            )
        );
    }
    test_field_length();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_spec() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_table>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).spec) as usize - ptr as usize
            },
            6usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_table),
                "::",
                stringify!(spec)
            )
        );
    }
    test_field_spec();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_checksum() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_table>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).checksum) as usize - ptr as usize
            },
            7usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_table),
                "::",
                stringify!(checksum)
            )
        );
    }
    test_field_checksum();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_oem() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_table>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).oem) as usize - ptr as usize
            },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_table),
                "::",
                stringify!(oem)
            )
        );
    }
    test_field_oem();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_productid() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_table>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).productid) as usize - ptr as usize
            },
            16usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_table),
                "::",
                stringify!(productid)
            )
        );
    }
    test_field_productid();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_oemptr() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_table>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).oemptr) as usize - ptr as usize
            },
            28usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_table),
                "::",
                stringify!(oemptr)
            )
        );
    }
    test_field_oemptr();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_oemsize() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_table>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).oemsize) as usize - ptr as usize
            },
            32usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_table),
                "::",
                stringify!(oemsize)
            )
        );
    }
    test_field_oemsize();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_oemcount() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_table>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).oemcount) as usize - ptr as usize
            },
            34usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_table),
                "::",
                stringify!(oemcount)
            )
        );
    }
    test_field_oemcount();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_lapic() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_table>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).lapic) as usize - ptr as usize
            },
            36usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_table),
                "::",
                stringify!(lapic)
            )
        );
    }
    test_field_lapic();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_reserved() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_table>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).reserved) as usize - ptr as usize
            },
            40usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_table),
                "::",
                stringify!(reserved)
            )
        );
    }
    test_field_reserved();
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct mpc_cpu {
    pub type_: ::std::os::raw::c_uchar,
    pub apicid: ::std::os::raw::c_uchar,
    pub apicver: ::std::os::raw::c_uchar,
    pub cpuflag: ::std::os::raw::c_uchar,
    pub cpufeature: ::std::os::raw::c_uint,
    pub featureflag: ::std::os::raw::c_uint,
    pub reserved: [::std::os::raw::c_uint; 2usize],
}
#[test]
fn bindgen_test_layout_mpc_cpu() {
    assert_eq!(
        ::std::mem::size_of::<mpc_cpu>(),
        20usize,
        concat!("Size of: ", stringify!(mpc_cpu))
    );
    assert_eq!(
        ::std::mem::align_of::<mpc_cpu>(),
        4usize,
        concat!("Alignment of ", stringify!(mpc_cpu))
    );
    #[tracing::instrument(level = "info", skip())]
    fn test_field_type() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_cpu>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).type_) as usize - ptr as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_cpu),
                "::",
                stringify!(type_)
            )
        );
    }
    test_field_type();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_apicid() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_cpu>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).apicid) as usize - ptr as usize
            },
            1usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_cpu),
                "::",
                stringify!(apicid)
            )
        );
    }
    test_field_apicid();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_apicver() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_cpu>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).apicver) as usize - ptr as usize
            },
            2usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_cpu),
                "::",
                stringify!(apicver)
            )
        );
    }
    test_field_apicver();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_cpuflag() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_cpu>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).cpuflag) as usize - ptr as usize
            },
            3usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_cpu),
                "::",
                stringify!(cpuflag)
            )
        );
    }
    test_field_cpuflag();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_cpufeature() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_cpu>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).cpufeature) as usize - ptr as usize
            },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_cpu),
                "::",
                stringify!(cpufeature)
            )
        );
    }
    test_field_cpufeature();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_featureflag() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_cpu>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).featureflag) as usize - ptr as usize
            },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_cpu),
                "::",
                stringify!(featureflag)
            )
        );
    }
    test_field_featureflag();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_reserved() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_cpu>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).reserved) as usize - ptr as usize
            },
            12usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_cpu),
                "::",
                stringify!(reserved)
            )
        );
    }
    test_field_reserved();
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct mpc_bus {
    pub type_: ::std::os::raw::c_uchar,
    pub busid: ::std::os::raw::c_uchar,
    pub bustype: [::std::os::raw::c_uchar; 6usize],
}
#[test]
fn bindgen_test_layout_mpc_bus() {
    assert_eq!(
        ::std::mem::size_of::<mpc_bus>(),
        8usize,
        concat!("Size of: ", stringify!(mpc_bus))
    );
    assert_eq!(
        ::std::mem::align_of::<mpc_bus>(),
        1usize,
        concat!("Alignment of ", stringify!(mpc_bus))
    );
    #[tracing::instrument(level = "info", skip())]
    fn test_field_type() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_bus>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).type_) as usize - ptr as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_bus),
                "::",
                stringify!(type_)
            )
        );
    }
    test_field_type();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_busid() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_bus>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).busid) as usize - ptr as usize
            },
            1usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_bus),
                "::",
                stringify!(busid)
            )
        );
    }
    test_field_busid();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_bustype() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_bus>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).bustype) as usize - ptr as usize
            },
            2usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_bus),
                "::",
                stringify!(bustype)
            )
        );
    }
    test_field_bustype();
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct mpc_ioapic {
    pub type_: ::std::os::raw::c_uchar,
    pub apicid: ::std::os::raw::c_uchar,
    pub apicver: ::std::os::raw::c_uchar,
    pub flags: ::std::os::raw::c_uchar,
    pub apicaddr: ::std::os::raw::c_uint,
}
#[test]
fn bindgen_test_layout_mpc_ioapic() {
    assert_eq!(
        ::std::mem::size_of::<mpc_ioapic>(),
        8usize,
        concat!("Size of: ", stringify!(mpc_ioapic))
    );
    assert_eq!(
        ::std::mem::align_of::<mpc_ioapic>(),
        4usize,
        concat!("Alignment of ", stringify!(mpc_ioapic))
    );
    #[tracing::instrument(level = "info", skip())]
    fn test_field_type() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_ioapic>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).type_) as usize - ptr as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_ioapic),
                "::",
                stringify!(type_)
            )
        );
    }
    test_field_type();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_apicid() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_ioapic>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).apicid) as usize - ptr as usize
            },
            1usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_ioapic),
                "::",
                stringify!(apicid)
            )
        );
    }
    test_field_apicid();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_apicver() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_ioapic>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).apicver) as usize - ptr as usize
            },
            2usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_ioapic),
                "::",
                stringify!(apicver)
            )
        );
    }
    test_field_apicver();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_flags() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_ioapic>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).flags) as usize - ptr as usize
            },
            3usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_ioapic),
                "::",
                stringify!(flags)
            )
        );
    }
    test_field_flags();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_apicaddr() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_ioapic>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).apicaddr) as usize - ptr as usize
            },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_ioapic),
                "::",
                stringify!(apicaddr)
            )
        );
    }
    test_field_apicaddr();
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct mpc_intsrc {
    pub type_: ::std::os::raw::c_uchar,
    pub irqtype: ::std::os::raw::c_uchar,
    pub irqflag: ::std::os::raw::c_ushort,
    pub srcbus: ::std::os::raw::c_uchar,
    pub srcbusirq: ::std::os::raw::c_uchar,
    pub dstapic: ::std::os::raw::c_uchar,
    pub dstirq: ::std::os::raw::c_uchar,
}
#[test]
fn bindgen_test_layout_mpc_intsrc() {
    assert_eq!(
        ::std::mem::size_of::<mpc_intsrc>(),
        8usize,
        concat!("Size of: ", stringify!(mpc_intsrc))
    );
    assert_eq!(
        ::std::mem::align_of::<mpc_intsrc>(),
        2usize,
        concat!("Alignment of ", stringify!(mpc_intsrc))
    );
    #[tracing::instrument(level = "info", skip())]
    fn test_field_type() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_intsrc>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).type_) as usize - ptr as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_intsrc),
                "::",
                stringify!(type_)
            )
        );
    }
    test_field_type();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_irqtype() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_intsrc>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).irqtype) as usize - ptr as usize
            },
            1usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_intsrc),
                "::",
                stringify!(irqtype)
            )
        );
    }
    test_field_irqtype();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_irqflag() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_intsrc>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).irqflag) as usize - ptr as usize
            },
            2usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_intsrc),
                "::",
                stringify!(irqflag)
            )
        );
    }
    test_field_irqflag();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_srcbus() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_intsrc>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).srcbus) as usize - ptr as usize
            },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_intsrc),
                "::",
                stringify!(srcbus)
            )
        );
    }
    test_field_srcbus();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_srcbusirq() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_intsrc>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).srcbusirq) as usize - ptr as usize
            },
            5usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_intsrc),
                "::",
                stringify!(srcbusirq)
            )
        );
    }
    test_field_srcbusirq();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_dstapic() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_intsrc>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).dstapic) as usize - ptr as usize
            },
            6usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_intsrc),
                "::",
                stringify!(dstapic)
            )
        );
    }
    test_field_dstapic();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_dstirq() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_intsrc>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).dstirq) as usize - ptr as usize
            },
            7usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_intsrc),
                "::",
                stringify!(dstirq)
            )
        );
    }
    test_field_dstirq();
}
pub const mp_irq_source_types_mp_INT: mp_irq_source_types = 0;
pub const mp_irq_source_types_mp_NMI: mp_irq_source_types = 1;
pub const mp_irq_source_types_mp_SMI: mp_irq_source_types = 2;
pub const mp_irq_source_types_mp_ExtINT: mp_irq_source_types = 3;
pub type mp_irq_source_types = ::std::os::raw::c_uint;
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct mpc_lintsrc {
    pub type_: ::std::os::raw::c_uchar,
    pub irqtype: ::std::os::raw::c_uchar,
    pub irqflag: ::std::os::raw::c_ushort,
    pub srcbusid: ::std::os::raw::c_uchar,
    pub srcbusirq: ::std::os::raw::c_uchar,
    pub destapic: ::std::os::raw::c_uchar,
    pub destapiclint: ::std::os::raw::c_uchar,
}
#[test]
fn bindgen_test_layout_mpc_lintsrc() {
    assert_eq!(
        ::std::mem::size_of::<mpc_lintsrc>(),
        8usize,
        concat!("Size of: ", stringify!(mpc_lintsrc))
    );
    assert_eq!(
        ::std::mem::align_of::<mpc_lintsrc>(),
        2usize,
        concat!("Alignment of ", stringify!(mpc_lintsrc))
    );
    #[tracing::instrument(level = "info", skip())]
    fn test_field_type() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_lintsrc>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).type_) as usize - ptr as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_lintsrc),
                "::",
                stringify!(type_)
            )
        );
    }
    test_field_type();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_irqtype() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_lintsrc>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).irqtype) as usize - ptr as usize
            },
            1usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_lintsrc),
                "::",
                stringify!(irqtype)
            )
        );
    }
    test_field_irqtype();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_irqflag() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_lintsrc>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).irqflag) as usize - ptr as usize
            },
            2usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_lintsrc),
                "::",
                stringify!(irqflag)
            )
        );
    }
    test_field_irqflag();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_srcbusid() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_lintsrc>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).srcbusid) as usize - ptr as usize
            },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_lintsrc),
                "::",
                stringify!(srcbusid)
            )
        );
    }
    test_field_srcbusid();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_srcbusirq() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_lintsrc>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).srcbusirq) as usize - ptr as usize
            },
            5usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_lintsrc),
                "::",
                stringify!(srcbusirq)
            )
        );
    }
    test_field_srcbusirq();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_destapic() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_lintsrc>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).destapic) as usize - ptr as usize
            },
            6usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_lintsrc),
                "::",
                stringify!(destapic)
            )
        );
    }
    test_field_destapic();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_destapiclint() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_lintsrc>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).destapiclint) as usize - ptr as usize
            },
            7usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_lintsrc),
                "::",
                stringify!(destapiclint)
            )
        );
    }
    test_field_destapiclint();
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct mpc_oemtable {
    pub signature: [::std::os::raw::c_char; 4usize],
    pub length: ::std::os::raw::c_ushort,
    pub rev: ::std::os::raw::c_char,
    pub checksum: ::std::os::raw::c_char,
    pub mpc: [::std::os::raw::c_char; 8usize],
}
#[test]
fn bindgen_test_layout_mpc_oemtable() {
    assert_eq!(
        ::std::mem::size_of::<mpc_oemtable>(),
        16usize,
        concat!("Size of: ", stringify!(mpc_oemtable))
    );
    assert_eq!(
        ::std::mem::align_of::<mpc_oemtable>(),
        2usize,
        concat!("Alignment of ", stringify!(mpc_oemtable))
    );
    #[tracing::instrument(level = "info", skip())]
    fn test_field_signature() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_oemtable>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).signature) as usize - ptr as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_oemtable),
                "::",
                stringify!(signature)
            )
        );
    }
    test_field_signature();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_length() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_oemtable>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).length) as usize - ptr as usize
            },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_oemtable),
                "::",
                stringify!(length)
            )
        );
    }
    test_field_length();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_rev() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_oemtable>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).rev) as usize - ptr as usize
            },
            6usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_oemtable),
                "::",
                stringify!(rev)
            )
        );
    }
    test_field_rev();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_checksum() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_oemtable>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).checksum) as usize - ptr as usize
            },
            7usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_oemtable),
                "::",
                stringify!(checksum)
            )
        );
    }
    test_field_checksum();
    #[tracing::instrument(level = "info", skip())]
    fn test_field_mpc() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<mpc_oemtable>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).mpc) as usize - ptr as usize
            },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(mpc_oemtable),
                "::",
                stringify!(mpc)
            )
        );
    }
    test_field_mpc();
}
pub const mp_bustype_MP_BUS_ISA: mp_bustype = 1;
pub const mp_bustype_MP_BUS_EISA: mp_bustype = 2;
pub const mp_bustype_MP_BUS_PCI: mp_bustype = 3;
pub type mp_bustype = ::std::os::raw::c_uint;
