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

//! TODO: should be merged with ../../src/monitor.rs

use serde::{Deserialize, Serialize};

/// Notification type for Monitor activity to test runner
#[derive(Serialize, Deserialize, Debug)]
pub enum MonitorNotification {
    Startup,
    Step,
    Shutdown,
}

/// Request type for test runner to Monitor activity
#[derive(Serialize, Deserialize, Debug)]
pub enum MonitorRequest {}
