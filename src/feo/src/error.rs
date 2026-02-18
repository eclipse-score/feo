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

//! FEO Error implementation

use crate::ids::{ActivityId, ChannelId, WorkerId};
use crate::signalling::common::signals::Signal;
use core::time::Duration;
#[cfg(feature = "recording")]
use postcard::experimental::max_size::MaxSize;
#[cfg(feature = "recording")]
use serde::Deserialize;
#[cfg(feature = "recording")]
use serde::Serialize;

/// FEO Error type
#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    ActivityFailed(ActivityId, ActivityError),
    ActivityNotFound(ActivityId),
    Channel(&'static str),
    ChannelClosed,
    ChannelNotFound(ChannelId),
    Io((std::io::Error, &'static str)),
    Timeout(Duration, &'static str),
    UnexpectedProtocolSignal,
    UnexpectedSignal(Signal),
    WorkerNotFound(WorkerId),
}

/// Defines the types of failures an Activity can report.
#[cfg_attr(feature = "recording", derive(Serialize, Deserialize, MaxSize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActivityError {
    /// A failure during the `startup()` method.
    Startup,
    /// A failure during a regular `step()` execution.
    Step,
    /// A failure during the `shutdown()` method.
    Shutdown,
}

impl core::error::Error for Error {}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::ActivityFailed(id, err) => {
                write!(f, "activity {id} reported a failure: {err:?}")
            },
            Error::ActivityNotFound(id) => write!(f, "failed to find activity with ID {id}"),
            Error::Channel(description) => write!(f, "channel error: {description}"),
            Error::ChannelClosed => write!(f, "channel closed by peer"),
            Error::ChannelNotFound(id) => write!(f, "failed to find channel with ID {id}"),
            Error::Io((e, description)) => write!(f, "{description}: io error: {e}"),
            Error::Timeout(duration, action) => {
                write!(f, "timeout reached ({:0.3}s) while {action}", duration.as_secs_f64())
            },
            Error::UnexpectedProtocolSignal => write!(f, "received unexpected protocol signal"),
            Error::UnexpectedSignal(signal) => write!(f, "received unexpected signal {signal}"),
            Error::WorkerNotFound(id) => write!(f, "failed to find worker with ID {id}"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io((err, "failed"))
    }
}
