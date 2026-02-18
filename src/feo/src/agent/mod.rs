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

//! FEO agents are processes.
//!
//! In each FEO application there is one primary agent and optional secondary
//! agents. The primary agent is responsible for triggering the execution of all activities distributed
//! across all agents.

use core::net::SocketAddr;
use std::path::PathBuf;

pub mod com_init;
pub mod direct;
pub mod relayed;

/// Node address of a connection
#[derive(Debug, Clone)]
pub enum NodeAddress {
    Tcp(SocketAddr),
    UnixSocket(PathBuf),
}
