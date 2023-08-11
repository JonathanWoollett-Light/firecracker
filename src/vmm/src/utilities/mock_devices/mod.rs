// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
#![allow(missing_docs)]

use std::fs::File;
use std::io;
use std::os::unix::io::{AsRawFd, RawFd};

#[derive(Debug)]
pub struct MockSerialInput(pub File);

impl io::Read for MockSerialInput {
    #[tracing::instrument(level = "trace", skip(self,buf))]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl AsRawFd for MockSerialInput {
    #[tracing::instrument(level = "trace", skip(self))]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}
