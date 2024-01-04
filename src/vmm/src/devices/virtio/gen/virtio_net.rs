// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
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

pub const VIRTIO_F_NOTIFY_ON_EMPTY: u32 = 24;
pub const VIRTIO_F_ANY_LAYOUT: u32 = 27;
pub const VIRTIO_F_VERSION_1: u32 = 32;
pub const VIRTIO_F_IOMMU_PLATFORM: u32 = 33;
pub const VIRTIO_NET_F_CSUM: u32 = 0;
pub const VIRTIO_NET_F_GUEST_CSUM: u32 = 1;
pub const VIRTIO_NET_F_CTRL_GUEST_OFFLOADS: u32 = 2;
pub const VIRTIO_NET_F_MTU: u32 = 3;
pub const VIRTIO_NET_F_MAC: u32 = 5;
pub const VIRTIO_NET_F_GUEST_TSO4: u32 = 7;
pub const VIRTIO_NET_F_GUEST_TSO6: u32 = 8;
pub const VIRTIO_NET_F_GUEST_ECN: u32 = 9;
pub const VIRTIO_NET_F_GUEST_UFO: u32 = 10;
pub const VIRTIO_NET_F_HOST_TSO4: u32 = 11;
pub const VIRTIO_NET_F_HOST_TSO6: u32 = 12;
pub const VIRTIO_NET_F_HOST_ECN: u32 = 13;
pub const VIRTIO_NET_F_HOST_UFO: u32 = 14;
pub const VIRTIO_NET_F_MRG_RXBUF: u32 = 15;
pub const VIRTIO_NET_F_STATUS: u32 = 16;
pub const VIRTIO_NET_F_CTRL_VQ: u32 = 17;
pub const VIRTIO_NET_F_CTRL_RX: u32 = 18;
pub const VIRTIO_NET_F_CTRL_VLAN: u32 = 19;
pub const VIRTIO_NET_F_CTRL_RX_EXTRA: u32 = 20;
pub const VIRTIO_NET_F_GUEST_ANNOUNCE: u32 = 21;
pub const VIRTIO_NET_F_MQ: u32 = 22;
pub const VIRTIO_NET_F_CTRL_MAC_ADDR: u32 = 23;
pub const VIRTIO_NET_F_GSO: u32 = 6;
pub type __u8 = ::std::os::raw::c_uchar;
pub type __u16 = ::std::os::raw::c_ushort;
pub type __virtio16 = __u16;
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct virtio_net_hdr_v1 {
    pub flags: __u8,
    pub gso_type: __u8,
    pub hdr_len: __virtio16,
    pub gso_size: __virtio16,
    pub csum_start: __virtio16,
    pub csum_offset: __virtio16,
    pub num_buffers: __virtio16,
}
#[test]
fn bindgen_test_layout_virtio_net_hdr_v1() {
    assert_eq!(
        ::std::mem::size_of::<virtio_net_hdr_v1>(),
        12usize,
        concat!("Size of: ", stringify!(virtio_net_hdr_v1))
    );
    assert_eq!(
        ::std::mem::align_of::<virtio_net_hdr_v1>(),
        2usize,
        concat!("Alignment of ", stringify!(virtio_net_hdr_v1))
    );
    fn test_field_flags() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<virtio_net_hdr_v1>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).flags) as usize - ptr as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(virtio_net_hdr_v1),
                "::",
                stringify!(flags)
            )
        );
    }
    test_field_flags();
    fn test_field_gso_type() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<virtio_net_hdr_v1>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).gso_type) as usize - ptr as usize
            },
            1usize,
            concat!(
                "Offset of field: ",
                stringify!(virtio_net_hdr_v1),
                "::",
                stringify!(gso_type)
            )
        );
    }
    test_field_gso_type();
    fn test_field_hdr_len() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<virtio_net_hdr_v1>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).hdr_len) as usize - ptr as usize
            },
            2usize,
            concat!(
                "Offset of field: ",
                stringify!(virtio_net_hdr_v1),
                "::",
                stringify!(hdr_len)
            )
        );
    }
    test_field_hdr_len();
    fn test_field_gso_size() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<virtio_net_hdr_v1>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).gso_size) as usize - ptr as usize
            },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(virtio_net_hdr_v1),
                "::",
                stringify!(gso_size)
            )
        );
    }
    test_field_gso_size();
    fn test_field_csum_start() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<virtio_net_hdr_v1>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).csum_start) as usize - ptr as usize
            },
            6usize,
            concat!(
                "Offset of field: ",
                stringify!(virtio_net_hdr_v1),
                "::",
                stringify!(csum_start)
            )
        );
    }
    test_field_csum_start();
    fn test_field_csum_offset() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<virtio_net_hdr_v1>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).csum_offset) as usize - ptr as usize
            },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(virtio_net_hdr_v1),
                "::",
                stringify!(csum_offset)
            )
        );
    }
    test_field_csum_offset();
    fn test_field_num_buffers() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<virtio_net_hdr_v1>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).num_buffers) as usize - ptr as usize
            },
            10usize,
            concat!(
                "Offset of field: ",
                stringify!(virtio_net_hdr_v1),
                "::",
                stringify!(num_buffers)
            )
        );
    }
    test_field_num_buffers();
}
