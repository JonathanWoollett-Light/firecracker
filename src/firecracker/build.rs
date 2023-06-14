// Copyright 2023 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
fn main() {
    println!(
        "cargo:rustc-env=FIRECRACKER_VERSION={}",
        env!("CARGO_PKG_VERSION")
    );
}
