// Copyright 2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//
// Portions Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the THIRD-PARTY file.

//! Implements a wrapper over an UART serial device.
use std::fmt::Debug;
use std::io;
use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, RawFd};

use event_manager::{EventOps, Events, MutEventSubscriber};
use log::{error, warn};
use logger::{IncMetric, METRICS};
use utils::epoll::EventSet;
use vm_superio::serial::{Error as SerialError, SerialEvents};
use vm_superio::{Serial, Trigger};

use crate::devices::legacy::EventFdTrigger;

/// Received Data Available interrupt - for letting the driver know that
/// there is some pending data to be processed.
pub const IER_RDA_BIT: u8 = 0b0000_0001;
/// Received Data Available interrupt offset
pub const IER_RDA_OFFSET: u8 = 1;

#[derive(Debug)]
pub enum RawIOError {
    Serial(SerialError<io::Error>),
}

pub trait RawIOHandler {
    /// Send raw input to this emulated device.
    fn raw_input(&mut self, _data: &[u8]) -> Result<(), RawIOError>;
}

impl<EV: SerialEvents + Debug, W: Write + Debug> RawIOHandler for Serial<EventFdTrigger, EV, W> {
    // This is not used for anything and is basically just a dummy implementation for `raw_input`.
    #[tracing::instrument(level = "trace", skip(self,data))]
    fn raw_input(&mut self, data: &[u8]) -> Result<(), RawIOError> {
        // Fail fast if the serial is serviced with more data than it can buffer.
        if data.len() > self.fifo_capacity() {
            return Err(RawIOError::Serial(SerialError::FullFifo));
        }

        // Before enqueuing bytes we first check if there is enough free space
        // in the FIFO.
        if self.fifo_capacity() >= data.len() {
            self.enqueue_raw_bytes(data).map_err(RawIOError::Serial)?;
        }
        Ok(())
    }
}

/// Wrapper over available events (i.e metrics, buffer ready etc).
#[derive(Debug)]
pub struct SerialEventsWrapper {
    /// Buffer ready event.
    pub buffer_ready_event_fd: Option<EventFdTrigger>,
}

impl SerialEvents for SerialEventsWrapper {
    #[tracing::instrument(level = "trace", skip(self))]
    fn buffer_read(&self) {
        METRICS.uart.read_count.inc();
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn out_byte(&self) {
        METRICS.uart.write_count.inc();
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn tx_lost_byte(&self) {
        METRICS.uart.missed_write_count.inc();
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn in_buffer_empty(&self) {
        match self
            .buffer_ready_event_fd
            .as_ref()
            .map_or(Ok(()), |buf_ready| buf_ready.write(1))
        {
            Ok(_) => (),
            Err(err) => error!(
                "Could not signal that serial device buffer is ready: {:?}",
                err
            ),
        }
    }
}

#[derive(Debug)]
pub enum SerialOut {
    Sink(std::io::Sink),
    Stdout(std::io::Stdout),
}
impl std::io::Write for SerialOut {
    #[tracing::instrument(level = "trace", skip(self,buf))]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Sink(sink) => sink.write(buf),
            Self::Stdout(stdout) => stdout.write(buf),
        }
    }
    #[tracing::instrument(level = "trace", skip(self))]
    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Sink(sink) => sink.flush(),
            Self::Stdout(stdout) => stdout.flush(),
        }
    }
}

/// Wrapper over the imported serial device.
#[derive(Debug)]
pub struct SerialWrapper<T: Trigger, EV: SerialEvents, I: Read + AsRawFd + Send> {
    /// Serial device object.
    pub serial: Serial<T, EV, SerialOut>,
    /// Input to the serial device (needs to be readable).
    pub input: Option<I>,
}

impl<I: Read + AsRawFd + Send + Debug> SerialWrapper<EventFdTrigger, SerialEventsWrapper, I> {
    #[tracing::instrument(level = "trace", skip(self,ops))]
    fn handle_ewouldblock(&self, ops: &mut EventOps) {
        let buffer_ready_fd = self.buffer_ready_evt_fd();
        let input_fd = self.serial_input_fd();
        if input_fd < 0 || buffer_ready_fd < 0 {
            error!("Serial does not have a configured input source.");
            return;
        }
        match ops.add(Events::new(&input_fd, EventSet::IN)) {
            Err(event_manager::Error::FdAlreadyRegistered) => (),
            Err(err) => {
                error!(
                    "Could not register the serial input to the event manager: {:?}",
                    err
                );
            }
            Ok(()) => {
                // Bytes might had come on the unregistered stdin. Try to consume any.
                self.serial.events().in_buffer_empty()
            }
        };
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn recv_bytes(&mut self) -> io::Result<usize> {
        let avail_cap = self.serial.fifo_capacity();
        if avail_cap == 0 {
            return Err(io::Error::from_raw_os_error(libc::ENOBUFS));
        }

        if let Some(input) = self.input.as_mut() {
            let mut out = vec![0u8; avail_cap];
            let count = input.read(&mut out)?;
            if count > 0 {
                self.serial
                    .raw_input(&out[..count])
                    .map_err(|_| io::Error::from_raw_os_error(libc::ENOBUFS))?;
            }

            return Ok(count);
        }

        Err(io::Error::from_raw_os_error(libc::ENOTTY))
    }

    #[tracing::instrument(level = "trace", skip(self))]
    #[inline]
    fn buffer_ready_evt_fd(&self) -> RawFd {
        self.serial
            .events()
            .buffer_ready_event_fd
            .as_ref()
            .map_or(-1, |buf_ready| buf_ready.as_raw_fd())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    #[inline]
    fn serial_input_fd(&self) -> RawFd {
        self.input.as_ref().map_or(-1, |input| input.as_raw_fd())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn consume_buffer_ready_event(&self) -> io::Result<u64> {
        self.serial
            .events()
            .buffer_ready_event_fd
            .as_ref()
            .map_or(Ok(0), |buf_ready| buf_ready.read())
    }
}

/// Type for representing a serial device.
pub type SerialDevice<I> = SerialWrapper<EventFdTrigger, SerialEventsWrapper, I>;

impl<I: Read + AsRawFd + Send + Debug> MutEventSubscriber
    for SerialWrapper<EventFdTrigger, SerialEventsWrapper, I>
{
    #[tracing::instrument(level = "trace", skip(self,event,ops))]
    /// Handle events on the serial input fd.
    fn process(&mut self, event: Events, ops: &mut EventOps) {
        #[tracing::instrument(level = "trace", skip(ops,source))]
        #[inline]
        fn unregister_source<T: AsRawFd + Debug>(ops: &mut EventOps, source: &T) {
            match ops.remove(Events::new(source, EventSet::IN)) {
                Ok(_) => (),
                Err(_) => error!("Could not unregister source fd: {}", source.as_raw_fd()),
            }
        }

        let input_fd = self.serial_input_fd();
        let buffer_ready_fd = self.buffer_ready_evt_fd();
        if input_fd < 0 || buffer_ready_fd < 0 {
            error!("Serial does not have a configured input source.");
            return;
        }

        if buffer_ready_fd == event.fd() {
            match self.consume_buffer_ready_event() {
                Ok(_) => (),
                Err(err) => {
                    error!(
                        "Detach serial device input source due to error in consuming the buffer \
                         ready event: {:?}",
                        err
                    );
                    unregister_source(ops, &input_fd);
                    unregister_source(ops, &buffer_ready_fd);
                    return;
                }
            }
        }

        // We expect to receive: `EventSet::IN`, `EventSet::HANG_UP` or
        // `EventSet::ERROR`. To process all these events we just have to
        // read from the serial input.
        match self.recv_bytes() {
            Ok(count) => {
                // Handle EOF if the event came from the input source.
                if input_fd == event.fd() && count == 0 {
                    unregister_source(ops, &input_fd);
                    unregister_source(ops, &buffer_ready_fd);
                    warn!("Detached the serial input due to peer close/error.");
                }
            }
            Err(err) => {
                match err.raw_os_error() {
                    Some(errno) if errno == libc::ENOBUFS => {
                        unregister_source(ops, &input_fd);
                    }
                    Some(errno) if errno == libc::EWOULDBLOCK => {
                        self.handle_ewouldblock(ops);
                    }
                    Some(errno) if errno == libc::ENOTTY => {
                        error!("The serial device does not have the input source attached.");
                        unregister_source(ops, &input_fd);
                        unregister_source(ops, &buffer_ready_fd);
                    }
                    Some(_) | None => {
                        // Unknown error, detach the serial input source.
                        unregister_source(ops, &input_fd);
                        unregister_source(ops, &buffer_ready_fd);
                        warn!("Detached the serial input due to peer close/error.");
                    }
                }
            }
        }
    }

    #[tracing::instrument(level = "trace", skip(self,ops))]
    /// Initial registration of pollable objects.
    /// If serial input is present, register the serial input FD as readable.
    fn init(&mut self, ops: &mut EventOps) {
        if self.input.is_some() && self.serial.events().buffer_ready_event_fd.is_some() {
            let serial_fd = self.serial_input_fd();
            let buf_ready_evt = self.buffer_ready_evt_fd();
            if serial_fd != -1 {
                if let Err(err) = ops.add(Events::new(&serial_fd, EventSet::IN)) {
                    warn!("Failed to register serial input fd: {}", err);
                }
            }
            if let Err(err) = ops.add(Events::new(&buf_ready_evt, EventSet::IN)) {
                warn!("Failed to register serial buffer ready event: {}", err);
            }
        }
    }
}

impl<I: Read + AsRawFd + Send + Debug + 'static>
    SerialWrapper<EventFdTrigger, SerialEventsWrapper, I>
{
    #[tracing::instrument(level = "trace", skip(self,offset,data))]
    pub fn bus_read(&mut self, offset: u64, data: &mut [u8]) {
        if data.len() != 1 {
            METRICS.uart.missed_read_count.inc();
            return;
        }
        data[0] = self.serial.read(offset as u8);
    }

    #[tracing::instrument(level = "trace", skip(self,offset,data))]
    pub fn bus_write(&mut self, offset: u64, data: &[u8]) {
        if data.len() != 1 {
            METRICS.uart.missed_write_count.inc();
            return;
        }
        if let Err(err) = self.serial.write(offset as u8, data[0]) {
            // Counter incremented for any handle_write() error.
            error!("Failed the write to serial: {:?}", err);
            METRICS.uart.error_count.inc();
        }
    }
}

#[cfg(test)]
mod tests {
    use utils::eventfd::EventFd;

    use super::*;

    #[test]
    fn test_serial_bus_read() {
        let intr_evt = EventFdTrigger::new(EventFd::new(libc::EFD_NONBLOCK).unwrap());

        let metrics = &METRICS.uart;

        let mut serial = SerialDevice {
            serial: Serial::with_events(
                intr_evt,
                SerialEventsWrapper {
                    buffer_ready_event_fd: None,
                },
                SerialOut::Sink(std::io::sink()),
            ),
            input: None::<std::io::Stdin>,
        };
        serial.serial.raw_input(&[b'a', b'b', b'c']).unwrap();

        let invalid_reads_before = metrics.missed_read_count.count();
        let mut v = [0x00; 2];
        serial.bus_read(0u64, &mut v);

        let invalid_reads_after = metrics.missed_read_count.count();
        assert_eq!(invalid_reads_before + 1, invalid_reads_after);

        let mut v = [0x00; 1];
        serial.bus_read(0u64, &mut v);
        assert_eq!(v[0], b'a');

        let invalid_reads_after_2 = metrics.missed_read_count.count();
        // The `invalid_read_count` metric should be the same as before the one-byte reads.
        assert_eq!(invalid_reads_after_2, invalid_reads_after);
    }
}
