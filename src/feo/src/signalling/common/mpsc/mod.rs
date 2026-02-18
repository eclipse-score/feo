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

//! Mpsc-only signalling module
//!
//! Mpsc-only signalling does not allow for inter-process communication. Therefore
//! it is not applicable for multi-process setups.

pub(crate) use crate::signalling::common::mpsc::worker::WorkerConnector;
use alloc::boxed::Box;

pub(crate) mod endpoint;
pub(crate) mod primitives;
pub(crate) mod worker;

pub type Builder<T> = Box<dyn FnOnce() -> T + Send>;
pub type WorkerConnectorBuilder = Builder<WorkerConnector>;
