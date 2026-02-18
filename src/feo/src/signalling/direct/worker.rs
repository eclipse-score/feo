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

use crate::error::Error;
use crate::ids::ActivityId;
use crate::signalling::common::interface::ConnectWorker;
use crate::signalling::common::signals::Signal;
use crate::signalling::common::socket::client::{SocketClient, TcpClient, UnixClient};
use crate::signalling::common::socket::ProtocolSignal;
use alloc::vec::Vec;
use core::net::SocketAddr;
use core::time::Duration;
use mio::net::{TcpStream, UnixStream};
use mio::Events;
use std::io;
use std::path::PathBuf;

/// TCP based connector for a worker
pub(crate) type TcpWorkerConnector = WorkerConnector<SocketAddr, TcpStream>;

/// Unix socket based connector for a worker
pub(crate) type UnixWorkerConnector = WorkerConnector<PathBuf, UnixStream>;

/// Connector for a worker
pub(crate) struct WorkerConnector<E, S>
where
    S: io::Read + io::Write,
{
    /// Endpoint on which the connector of the scheduler is listening
    endpoint: E,
    /// Pre-allocated events buffer
    events: Events,
    /// Wrapped socket client
    client: Option<SocketClient<S>>,
    /// [ActivityId]s to announce when connecting
    activity_ids: Vec<ActivityId>,
}

impl<E, S> WorkerConnector<E, S>
where
    S: io::Read + io::Write,
{
    fn receive(&mut self, timeout: Duration) -> Result<Option<Signal>, Error> {
        match self
            .client
            .as_mut()
            .expect("socket client not connected")
            .receive(&mut self.events, timeout)
        {
            Ok(Some(ProtocolSignal::Core(signal))) => Ok(Some(signal)),
            Ok(Some(_signal)) => Err(Error::UnexpectedProtocolSignal),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn send_to_scheduler(&mut self, signal: &Signal) -> Result<(), Error> {
        self.client
            .as_mut()
            .expect("socket client not connected")
            .send(&ProtocolSignal::Core(*signal))
    }
}

impl TcpWorkerConnector {
    /// Create a new instance
    pub(crate) fn new(address: SocketAddr, activity_ids: impl IntoIterator<Item = ActivityId>) -> Self {
        let activity_ids = activity_ids.into_iter().collect();
        Self {
            endpoint: address,
            events: Events::with_capacity(32),
            client: None,
            activity_ids,
        }
    }

    fn connect_remote(&mut self) -> Result<(), Error> {
        let connect_signals = self.activity_ids.iter().map(|id| ProtocolSignal::ActivityHello(*id));
        let tcp_client = TcpClient::connect(connect_signals, self.endpoint);
        self.client = Some(tcp_client);
        Ok(())
    }
}

impl UnixWorkerConnector {
    /// Create a new instance
    pub(crate) fn new(path: PathBuf, activity_ids: impl IntoIterator<Item = ActivityId>) -> Self {
        let activity_ids = activity_ids.into_iter().collect();
        Self {
            endpoint: path,
            events: Events::with_capacity(32),
            client: None,
            activity_ids,
        }
    }

    fn connect_remote(&mut self) -> Result<(), Error> {
        let connect_signals = self.activity_ids.iter().map(|id| ProtocolSignal::ActivityHello(*id));
        let unix_client = UnixClient::connect(connect_signals, &self.endpoint);
        self.client = Some(unix_client);
        Ok(())
    }
}

impl ConnectWorker for TcpWorkerConnector {
    fn connect_remote(&mut self) -> Result<(), Error> {
        self.connect_remote()
    }

    fn receive(&mut self, timeout: Duration) -> Result<Option<Signal>, Error> {
        self.receive(timeout)
    }

    fn send_to_scheduler(&mut self, signal: &Signal) -> Result<(), Error> {
        self.send_to_scheduler(signal)
    }
}

impl ConnectWorker for UnixWorkerConnector {
    fn connect_remote(&mut self) -> Result<(), Error> {
        self.connect_remote()
    }

    fn receive(&mut self, timeout: Duration) -> Result<Option<Signal>, Error> {
        self.receive(timeout)
    }

    fn send_to_scheduler(&mut self, signal: &Signal) -> Result<(), Error> {
        self.send_to_scheduler(signal)
    }
}
