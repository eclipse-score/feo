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

use crate::error::Error;
use crate::ids::{ActivityId, AgentId};
use crate::signalling::common::interface::ConnectScheduler;
use crate::signalling::common::signals::Signal;
use crate::signalling::common::socket::server::{Listen, SocketServer, TcpServer, UnixServer};
use crate::signalling::common::socket::ProtocolSignal;
use crate::timestamp::sync_info;
use alloc::vec::Vec;
use core::net::SocketAddr;
use core::time::Duration;
use feo_log::warn;
use feo_time::Instant;
use mio::net::{TcpListener, UnixListener};
use mio::{Events, Token};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// TCP based connector for the scheduler
pub(crate) type TcpSchedulerConnector = SchedulerConnector<TcpListener>;

/// Unix socket based connector for the scheduler
pub(crate) type UnixSchedulerConnector = SchedulerConnector<UnixListener>;

/// Connector for the scheduler
pub(crate) struct SchedulerConnector<L>
where
    L: Listen<ProtocolSignal>,
{
    events: Events,
    server: SocketServer<L>,

    activity_id_token_map: HashMap<ActivityId, Token>,
    recorder_id_token_map: HashMap<AgentId, Token>,
    activity_agent_map: HashMap<ActivityId, AgentId>,

    all_activities: Vec<ActivityId>,
    all_recorders: Vec<AgentId>,
    connection_timeout: Duration,
}

impl<L> SchedulerConnector<L>
where
    L: Listen<ProtocolSignal>,
{
    /// Create a new instance
    fn new_with_server(
        server: SocketServer<L>,
        activity_ids: impl IntoIterator<Item = ActivityId>,
        recorder_ids: impl IntoIterator<Item = AgentId>,
        activity_agent_map: HashMap<ActivityId, AgentId>,
        connection_timeout: Duration,
    ) -> Self {
        let events = Events::with_capacity(32);

        let activity_id_token_map = HashMap::new();
        let recorder_id_token_map = HashMap::new();

        let all_activities = activity_ids.into_iter().collect::<Vec<_>>();
        let all_recorders = recorder_ids.into_iter().collect::<Vec<_>>();

        Self {
            events,
            server,
            activity_id_token_map,
            recorder_id_token_map,
            activity_agent_map,
            all_activities,
            all_recorders,
            connection_timeout,
        }
    }
}

impl TcpSchedulerConnector {
    /// Create a new instance
    pub(crate) fn new(
        bind_address: SocketAddr,
        activity_ids: impl IntoIterator<Item = ActivityId>,
        recorder_ids: impl IntoIterator<Item = AgentId>,
        activity_agent_map: HashMap<ActivityId, AgentId>,
        connection_timeout: Duration,
    ) -> Self {
        let tcp_server = TcpServer::new(bind_address);
        Self::new_with_server(
            tcp_server,
            activity_ids,
            recorder_ids,
            activity_agent_map,
            connection_timeout,
        )
    }
}

impl UnixSchedulerConnector {
    /// Create a new instance
    pub(crate) fn new(
        path: &Path,
        activity_ids: impl IntoIterator<Item = ActivityId>,
        recorder_ids: impl IntoIterator<Item = AgentId>,
        activity_agent_map: HashMap<ActivityId, AgentId>,
        connection_timeout: Duration,
    ) -> Self {
        let unix_server = UnixServer::new(path);
        Self::new_with_server(
            unix_server,
            activity_ids,
            recorder_ids,
            activity_agent_map,
            connection_timeout,
        )
    }
}

impl<L> ConnectScheduler for SchedulerConnector<L>
where
    L: Listen<ProtocolSignal>,
{
    fn connect_remotes(&mut self) -> Result<(), Error> {
        let mut missing_activities: HashSet<ActivityId> =
            self.all_activities.iter().cloned().collect();
        let mut missing_recorders: HashSet<AgentId> = self.all_recorders.iter().cloned().collect();
        let start_time = Instant::now();

        while !missing_activities.is_empty() || !missing_recorders.is_empty() {
            let elapsed = start_time.elapsed();
            if elapsed >= self.connection_timeout {
                return Err(Error::Io((
                    std::io::ErrorKind::TimedOut.into(),
                    "CONNECTION_TIMEOUT",
                )));
            }
            let remaining_timeout = self.connection_timeout.saturating_sub(elapsed);
            // Wait for a new connection, but no longer than the remaining overall timeout.
            if let Ok(Some((token, signal))) =
                self.server.receive(&mut self.events, remaining_timeout)
            {
                match signal {
                    ProtocolSignal::ActivityHello(activity_id) => {
                        self.activity_id_token_map.insert(activity_id, token);
                        missing_activities.remove(&activity_id);
                    }
                    ProtocolSignal::RecorderHello(agent_id) => {
                        self.recorder_id_token_map.insert(agent_id, token);
                        missing_recorders.remove(&agent_id);
                    }
                    other => {
                        warn!("received unexpected signal {other:?} from connection with token {token:?}");
                    }
                }
            }
        }

        Ok(())
    }

    fn sync_time(&mut self) -> Result<(), Error> {
        let signal = Signal::StartupSync(sync_info());

        // Send startup time to all workers and recorders
        for token in self
            .activity_id_token_map
            .values()
            .chain(self.recorder_id_token_map.values())
        {
            self.server
                .send(token, &ProtocolSignal::Core(signal))
                .map_err(|e| Error::Io((e, "failed to send")))?;
        }

        Ok(())
    }

    fn get_connected_agent_ids(&self) -> Vec<AgentId> {
        let mut agent_ids: HashSet<AgentId> = self.recorder_id_token_map.keys().copied().collect();
        for activity_id in self.activity_id_token_map.keys() {
            if let Some(agent_id) = self.activity_agent_map.get(activity_id) {
                agent_ids.insert(*agent_id);
            }
        }
        agent_ids.into_iter().collect()
    }

    fn receive(&mut self, timeout: Duration) -> Result<Option<Signal>, Error> {
        match self.server.receive(&mut self.events, timeout) {
            Ok(Some((_, ProtocolSignal::Core(signal)))) => Ok(Some(signal)),
            Ok(Some((_, other))) => {
                warn!("received unexpected protocol signal {:?}", other);
                Ok(None)
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn send_to_activity(&mut self, activity_id: ActivityId, signal: &Signal) -> Result<(), Error> {
        let token = self
            .activity_id_token_map
            .get(&activity_id)
            .unwrap_or_else(|| panic!("failed to find token for activity ID {activity_id}"));
        self.server
            .send(token, &ProtocolSignal::Core(*signal))
            .map_err(|e| Error::Io((e, "failed to send")))
    }

    fn send_to_recorder(&mut self, recorder_id: AgentId, signal: &Signal) -> Result<(), Error> {
        let token = self
            .recorder_id_token_map
            .get(&recorder_id)
            .unwrap_or_else(|| panic!("failed to find token for recorder ID {recorder_id}"));
        self.server
            .send(token, &ProtocolSignal::Core(*signal))
            .map_err(|e| Error::Io((e, "failed to send")))
    }
    fn broadcast_terminate(&mut self, signal: &Signal) -> Result<(), Error> {
        let protocol_signal = ProtocolSignal::Core(*signal);

        // Collect unique tokens to avoid sending the same message multiple times to the same worker.
        let unique_tokens: HashSet<_> = self
            .activity_id_token_map
            .values()
            .chain(self.recorder_id_token_map.values())
            .collect();

        for token in unique_tokens {
            self.server.send(token, &protocol_signal)?;
        }
        Ok(())
    }
}
