// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
#[log_instrument::instrument]
fn main() {
    unsafe {
        // Harmless print to standard output.
        libc::syscall(libc::SYS_write, libc::STDOUT_FILENO, "Hello, world!\n", 14);
    }
}
