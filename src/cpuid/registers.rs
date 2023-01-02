// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
#![allow(missing_docs)] // Adding documentation to registers requires notable effort.
#![allow(
    clippy::similar_names,
    clippy::module_name_repetitions,
    clippy::non_ascii_literal
)]

use bit_fields::bitfield;

// -------------------------------------------------------------------------------------------------
// Leaf 80000002
// -------------------------------------------------------------------------------------------------
#[rustfmt::skip]
bitfield!(Leaf80000002Eax, u32, {
    /// Processor Brand String.
    processor_brand_string: 0..32,
});
#[rustfmt::skip]
bitfield!(Leaf80000002Ebx, u32, {
    /// Processor Brand String Continued.
    processor_brand_string: 0..32,
});
#[rustfmt::skip]
bitfield!(Leaf80000002Ecx, u32, {
    /// Processor Brand String Continued.
    processor_brand_string: 0..32,
});
#[rustfmt::skip]
bitfield!(Leaf80000002Edx, u32, {
    /// Processor Brand String Continued.
    processor_brand_string: 0..32,
});
