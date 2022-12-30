// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

fn main() {
    // Sets a `--cfg` flag for conditional compilation.
    //
    // TODO: Replace checking of CPUID availability with `x86` and `x86_64` check and
    // [`std::arch_x86_64::has_cpuid()`] when this is stabilized. CPUID is supported when:
    // - We are on an x86 architecture with `sse` enabled and `sgx disabled`.
    // - We are on an x86_64 architecture with `sgx` disabled
    #[cfg(any(
        all(target_arch = "x86", target_feature = "sse", not(target_env = "sgx")),
        all(target_arch = "x86_64", not(target_env = "sgx"))
    ))]
    println!("cargo:rustc-cfg=cpuid");
}
