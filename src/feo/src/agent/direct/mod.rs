/********************************************************************************
 * Copyright (c) 2025 Contributors to the Eclipse Foundation
 *
 * See the NOTICE file(s) distributed with this work for additional
 * information regarding copyright ownership.
 *
 * This program and the accompanying materials are made available under the
 * terms of the Apache License Version 2.0 which is available at
 * https://www.apache.org/licenses/LICENSE-2.0
 *
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/

//! Agent implementation for direct scheduler-to-worker signalling
//!
//! The scheduler connects directly to each worker, independent of whether
//! the worker runs in the same process or another process.

pub mod primary;
pub mod primary_mpsc;
#[cfg(feature = "recording")]
pub mod recorder;
pub mod secondary;
