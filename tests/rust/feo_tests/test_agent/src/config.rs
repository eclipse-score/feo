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

use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use feo::ids::AgentId;
use feo_com::interface::ComBackend;
use std::time::Duration;

pub const PRIMARY_AGENT_ID: AgentId = AgentId::new(100);
pub const BIND_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8081);
pub const BIND_ADDR2: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8082);
pub const DEFAULT_FEO_CYCLE_TIME: Duration = Duration::from_millis(100);
pub const SOCKET_PATH: &str = "/tmp/feo_listener1.socket";
pub const SOCKET_PATH2: &str = "/tmp/feo_listener2.socket";

#[cfg(feature = "com_iox2")]
pub const COM_BACKEND: ComBackend = ComBackend::Iox2;
#[cfg(feature = "com_linux_shm")]
pub const COM_BACKEND: ComBackend = ComBackend::LinuxShm;

pub const TOPIC_COUNTER: &str = "feo/com/test/counter";

/// Allow up to two recorder processes (that potentially need to subscribe to every topic)
pub const MAX_ADDITIONAL_SUBSCRIBERS: usize = 2;
