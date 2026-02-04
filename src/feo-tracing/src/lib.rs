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

//! Tracing library for the FEO project.

/// The `subscriber` module contains the `Subscriber` struct, which is a custom `tracing` subscriber.
/// The tracing data is forward to `feo-tracer`
#[path = "subscriber.rs"]
mod feo_subscriber;
pub mod protocol;

/// Initialize tracing
pub use feo_subscriber::init;
pub use feo_subscriber::ScoreDebugIoError;
/// Re-export of the `tracing` crate.
pub use tracing::{self, event, instrument, level_filters::LevelFilter, span, Level};
