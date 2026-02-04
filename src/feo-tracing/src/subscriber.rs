// *******************************************************************************
// Copyright (c) 2025 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// <https://www.apache.org/licenses/LICENSE-2.0>
//
// SPDX-License-Identifier: Apache-2.0
// *******************************************************************************

use crate::protocol::{truncate, EventInfo, TraceData, TracePacket, MAX_INFO_SIZE, MAX_PACKET_SIZE};
use core::sync::atomic;
use core::sync::atomic::AtomicBool;
use core::time::Duration;
use score_log::error;
use score_log::fmt::{FormatSpec, ScoreDebug, ScoreWrite};
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::sync::mpsc::SendError;
use std::sync::{mpsc, Arc};
use std::thread::JoinHandle;
use std::{io, thread};
use tracing::level_filters::LevelFilter;
use tracing::span;
use tracing::subscriber::set_global_default;

/// The unix socket path used by the tracing daemon to receive trace packets
pub const UNIX_PACKET_PATH: &str = "/tmp/feo-tracer.sock";

/// Size of the channel (number of packets) for transmitting trace packets to the serializing thread
const MPSC_CHANNEL_BOUND: usize = 512;

/// Size of the buffer (bytes) for transmitting serialized packets to the trace daemon
const BUFWRITER_SIZE: usize = 512 * MAX_PACKET_SIZE;

/// Size of the maximal time interval after which to flush packets to the daemon
const FLUSH_INTERVAL: Duration = Duration::from_millis(500);

/// Initialize the tracing subscriber with the given level
pub fn init(level: LevelFilter) {
    let (sender, receiver) = mpsc::sync_channel::<TracePacket>(MPSC_CHANNEL_BOUND);
    let enabled = Arc::new(AtomicBool::new(true));

    // Spawn thread for serializing trace packets and sending to the trace daemon
    let _thread = {
        let enabled = Arc::clone(&enabled);
        thread::spawn(|| Subscriber::thread_main(receiver, enabled))
    };

    let subscriber = Subscriber {
        max_level: level,
        enabled,
        _thread,
        sender,
    };
    set_global_default(subscriber).expect("setting tracing default failed");
}

/// ScoreDebug support for std::io::Error
#[derive(Debug)]
pub struct ScoreDebugIoError(pub std::io::Error);

impl std::fmt::Display for ScoreDebugIoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ScoreDebug for ScoreDebugIoError {
    fn fmt(
        &self,
        f: &mut dyn score_log::fmt::ScoreWrite,
        spec: &score_log::fmt::FormatSpec,
    ) -> Result<(), score_log::fmt::Error> {
        use std::io::ErrorKind;

        match self.0.kind() {
            ErrorKind::NotFound => f.write_str("NotFound", spec),
            ErrorKind::PermissionDenied => f.write_str("PermissionDenied", spec),
            ErrorKind::ConnectionRefused => f.write_str("ConnectionRefused", spec),
            ErrorKind::ConnectionReset => f.write_str("ConnectionReset", spec),
            ErrorKind::HostUnreachable => f.write_str("HostUnreachable", spec),
            ErrorKind::NetworkUnreachable => f.write_str("NetworkUnreachable", spec),
            ErrorKind::ConnectionAborted => f.write_str("ConnectionAborted", spec),
            ErrorKind::NotConnected => f.write_str("NotConnected", spec),
            ErrorKind::AddrInUse => f.write_str("AddrInUse", spec),
            ErrorKind::AddrNotAvailable => f.write_str("AddrNotAvailable", spec),
            ErrorKind::NetworkDown => f.write_str("NetworkDown", spec),
            ErrorKind::BrokenPipe => f.write_str("BrokenPipe", spec),
            ErrorKind::AlreadyExists => f.write_str("AlreadyExists", spec),
            ErrorKind::WouldBlock => f.write_str("WouldBlock", spec),
            ErrorKind::NotADirectory => f.write_str("NotADirectory", spec),
            ErrorKind::IsADirectory => f.write_str("IsADirectory", spec),
            ErrorKind::DirectoryNotEmpty => f.write_str("DirectoryNotEmpty", spec),
            ErrorKind::ReadOnlyFilesystem => f.write_str("ReadOnlyFilesystem", spec),
            ErrorKind::StaleNetworkFileHandle => f.write_str("StaleNetworkFileHandle", spec),
            ErrorKind::InvalidInput => f.write_str("InvalidInput", spec),
            ErrorKind::InvalidData => f.write_str("InvalidData", spec),
            ErrorKind::TimedOut => f.write_str("TimedOut", spec),
            ErrorKind::WriteZero => f.write_str("WriteZero", spec),
            ErrorKind::StorageFull => f.write_str("StorageFull", spec),
            ErrorKind::NotSeekable => f.write_str("NotSeekable", spec),
            ErrorKind::QuotaExceeded => f.write_str("QuotaExceeded", spec),
            ErrorKind::FileTooLarge => f.write_str("FileTooLarge", spec),
            ErrorKind::ResourceBusy => f.write_str("ResourceBusy", spec),
            ErrorKind::ExecutableFileBusy => f.write_str("ExecutableFileBusy", spec),
            ErrorKind::Deadlock => f.write_str("Deadlock", spec),
            ErrorKind::CrossesDevices => f.write_str("CrossesDevices", spec),
            ErrorKind::TooManyLinks => f.write_str("TooManyLinks", spec),
            ErrorKind::InvalidFilename => f.write_str("InvalidFilename", spec),
            ErrorKind::ArgumentListTooLong => f.write_str("ArgumentListTooLong", spec),
            ErrorKind::Interrupted => f.write_str("Interrupted", spec),
            ErrorKind::Unsupported => f.write_str("Unsupported", spec),
            ErrorKind::UnexpectedEof => f.write_str("UnexpectedEof", spec),
            ErrorKind::OutOfMemory => f.write_str("OutOfMemory", spec),
            ErrorKind::Other => f.write_str("Other", spec),
            _ => f.write_str("IO error", spec),
        }
    }
}

impl From<std::io::Error> for ScoreDebugIoError {
    fn from(err: std::io::Error) -> Self {
        ScoreDebugIoError(err)
    }
}

struct ScoreDebugPostcardError(pub postcard::Error);

impl ScoreDebug for ScoreDebugPostcardError {
    fn fmt(&self, f: &mut dyn ScoreWrite, spec: &FormatSpec) -> Result<(), score_log::fmt::Error> {
        use postcard::Error;
        match self.0 {
            Error::WontImplement => f.write_str("WontImplement", spec),
            Error::NotYetImplemented => f.write_str("NotYetImplemented", spec),
            Error::SerializeBufferFull => f.write_str("SerializeBufferFull", spec),
            Error::SerializeSeqLengthUnknown => f.write_str("SerializeSeqLengthUnknown", spec),
            Error::DeserializeUnexpectedEnd => f.write_str("DeserializeUnexpectedEnd", spec),
            Error::DeserializeBadVarint => f.write_str("DeserializeBadVarint", spec),
            Error::DeserializeBadBool => f.write_str("DeserializeBadBool", spec),
            Error::DeserializeBadChar => f.write_str("DeserializeBadChar", spec),
            Error::DeserializeBadUtf8 => f.write_str("DeserializeBadUtf8", spec),
            Error::DeserializeBadOption => f.write_str("DeserializeBadOption", spec),
            Error::DeserializeBadEnum => f.write_str("DeserializeBadEnum", spec),
            Error::DeserializeBadEncoding => f.write_str("DeserializeBadEncoding", spec),
            Error::DeserializeBadCrc => f.write_str("DeserializeBadCrc", spec),
            Error::SerdeSerCustom => f.write_str("SerdeSerCustom", spec),
            Error::SerdeDeCustom => f.write_str("SerdeDeCustom", spec),
            Error::CollectStrError => f.write_str("CollectStrError", spec),
            _ => f.write_str("postcard error", spec),
        }
    }
}

// The field is unused, but kept for consistency
struct ScoreDebugSendError(#[allow(dead_code)] pub SendError<TracePacket>);

impl ScoreDebug for ScoreDebugSendError {
    fn fmt(&self, f: &mut dyn ScoreWrite, spec: &FormatSpec) -> Result<(), score_log::fmt::Error> {
        // A send operation can only fail if the receiving end of a channel is
        // disconnected (according to Ferrocene docs)
        f.write_str("disconnected", spec)
    }
}

/// A subscriber sending trace data to the feo-tracer via unix socket and postcard serialized data.
///
/// See the `TraceData` and `TracePacket` types for the data format.
struct Subscriber {
    max_level: LevelFilter,
    enabled: Arc<AtomicBool>,
    _thread: JoinHandle<()>,
    sender: mpsc::SyncSender<TracePacket>,
}

impl Subscriber {
    /// Generate a new span id
    fn new_span_id(&self) -> span::Id {
        /// Next span id. This is a global counter. Span ids must not be 0.
        static NEXT_ID: atomic::AtomicU64 = atomic::AtomicU64::new(1);

        // Generate next span id
        let id = NEXT_ID.fetch_add(1, atomic::Ordering::Relaxed);

        span::Id::from_u64(id)
    }

    fn thread_main(receiver: mpsc::Receiver<TracePacket>, enabled: Arc<AtomicBool>) {
        let connection = match UnixStream::connect(UNIX_PACKET_PATH) {
            Ok(connection) => connection,
            Err(e) => {
                error!("Failed to connect to feo-tracer: {:?}, aborting", ScoreDebugIoError(e));
                // disable further tracing (TODO: add a time period of retrying)
                enabled.store(false, atomic::Ordering::Relaxed);
                return;
            },
        };

        // Create buffer for serialization
        let mut buffer = [0u8; MAX_PACKET_SIZE];

        // Create BufferedWriter for socket
        let mut socket_writer = io::BufWriter::with_capacity(BUFWRITER_SIZE, connection);
        let mut last_flush = std::time::Instant::now();

        loop {
            let packet = receiver.recv().expect("trace subscriber failed to receive, aborting");

            let serialized = match postcard::to_slice_cobs(&packet, &mut buffer[..]) {
                Ok(serialized) => serialized,
                Err(e) => {
                    error!("Failed to serialize trace packet: {:?}", ScoreDebugPostcardError(e));
                    continue;
                },
            };

            let ret = socket_writer.write_all(serialized);
            if let Err(error) = ret {
                error!("Failed to send to feo-tracer: {:?}, aborting", ScoreDebugIoError(error));
                enabled.store(false, atomic::Ordering::Relaxed);
                return;
            }

            // Flush, if pre-defined time interval elapsed or insufficient spare capacity
            if last_flush.elapsed() > FLUSH_INTERVAL {
                socket_writer.flush().expect("failed to flush");
                last_flush = std::time::Instant::now();
            }
        }
    }

    // Send a value to the tracer
    fn send(&self, packet: TracePacket) {
        if !self.enabled.load(atomic::Ordering::Relaxed) {
            return;
        }
        if let Err(e) = self.sender.send(packet) {
            error!(
                "Failed to connect to feo-tracer: {:?}, aborting",
                ScoreDebugSendError(e)
            );
            self.enabled.store(false, atomic::Ordering::Relaxed);
        }
    }
}

impl tracing::Subscriber for Subscriber {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        // A span or event is enabled if it is at or below the configured
        // maximum level
        metadata.level() <= &self.max_level
    }

    fn max_level_hint(&self) -> Option<LevelFilter> {
        Some(self.max_level)
    }

    fn new_span(&self, span: &span::Attributes) -> span::Id {
        let id = self.new_span_id();
        let mut name = [0u8; MAX_INFO_SIZE];
        let name_len = truncate(span.metadata().name(), &mut name);
        let mut info = EventInfo::default();
        span.record(&mut info);
        let trace_data = TraceData::NewSpan {
            id: id.into_u64(),
            name,
            name_len,
            info,
        };
        let trace_packet = TracePacket::now_with_data(trace_data);
        self.send(trace_packet);
        id
    }

    fn record(&self, span: &span::Id, _: &span::Record) {
        let trace_data = TraceData::Record { span: span.into_u64() };
        let trace_packet = TracePacket::now_with_data(trace_data);
        self.send(trace_packet);
    }

    fn record_follows_from(&self, _span: &span::Id, _follows: &span::Id) {}

    fn event(&self, event: &tracing::Event) {
        let mut name = [0u8; MAX_INFO_SIZE];
        let name_len = truncate(event.metadata().name(), &mut name);
        let mut info = EventInfo::default();
        event.record(&mut info);
        let trace_data = TraceData::Event {
            parent_span: self.current_span().id().map(|id| id.into_u64()),
            name,
            name_len,
            info,
        };
        let trace_packet = TracePacket::now_with_data(trace_data);
        self.send(trace_packet);
    }

    fn enter(&self, span: &span::Id) {
        let trace_data = TraceData::Enter { span: span.into_u64() };
        let trace_packet = TracePacket::now_without_process(trace_data);
        self.send(trace_packet);
    }

    fn exit(&self, span: &span::Id) {
        let trace_data = TraceData::Exit { span: span.into_u64() };
        let trace_packet = TracePacket::now_without_process(trace_data);
        self.send(trace_packet);
    }
}
