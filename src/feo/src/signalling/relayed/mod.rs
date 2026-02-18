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

//! Relayed signalling module
//!
//! This module provides variants of relayed signalling.
//! In a relayed signalling setup, different communication channels are used for inter-process
//! and intra-process communication. Every secondary agent (or recorder) uses two inter-process
//! channels for sending and receiving signals to or from the primary agent. Two relay threads
//! in each agent are used to transfer signals between inter- and intra-process channels. In the
//! secondary agents, each worker thread is attached to the respective relays by means of two
//! intra-process connections. In the primary agent, workers are connected directly to the
//! through intra-process channels.

mod connectors;
mod interface;
mod mpsc;
mod sockets;
pub(crate) mod sockets_mpsc;

/// Trait for the connector of a secondary agent in relayed signalling setups
pub(crate) trait ConnectSecondary {
    fn run_and_connect(&mut self);
}
