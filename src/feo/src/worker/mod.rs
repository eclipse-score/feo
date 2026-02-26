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

//! Worker thread running FEO activities

use crate::activity::{Activity, ActivityBuilder};
use crate::error::Error;
use crate::ids::{ActivityId, AgentId, WorkerId};
use crate::signalling::common::interface::ConnectWorker;
use crate::signalling::common::signals::Signal;
use crate::timestamp;
use alloc::boxed::Box;
use feo_time::Duration;
use feo_time::Instant;
use score_log::{debug, error};
use std::collections::HashMap;
use std::thread;

/// Worker
///
/// Activities are built in the worker thread with the passed builders
/// and never move to another thread after being built.
/// The connector passed to the worker is already connected to the scheduler.
pub(crate) struct Worker<T: ConnectWorker> {
    /// ID of this worker
    id: WorkerId,
    /// ID of the agent this worker belongs to
    agent_id: AgentId,
    /// Map from [ActivityId] to the activity
    activities: HashMap<ActivityId, Box<dyn Activity>>,
    /// Connector to the scheduler
    connector: T,
    /// Timeout on `receive` calls
    timeout: Duration,
}

impl<T: ConnectWorker> Worker<T> {
    /// Create a new instance
    pub(crate) fn new(
        id: WorkerId,
        agent_id: AgentId,
        activity_builders: impl IntoIterator<Item = (ActivityId, Box<dyn ActivityBuilder>)>,
        connector: T,
        timeout: Duration,
    ) -> Self {
        // Build activities
        let activities: HashMap<ActivityId, _> = activity_builders.into_iter().map(|(id, b)| (id, b(id))).collect();

        Self {
            id,
            agent_id,
            activities,
            connector,
            timeout,
        }
    }

    /// Run the worker
    pub(crate) fn run(mut self) -> Result<(), Error> {
        debug!("Running worker {}", self.id);

        loop {
            let signal = match self.connector.receive(self.timeout) {
                Ok(Some(s)) => s,
                Ok(None) => {
                    // TODO: Manage timeout
                    continue;
                },
                Err(Error::ChannelClosed) => {
                    debug!(
                        "Worker {} detected closed channel from scheduler/relay. Exiting.",
                        self.id
                    );
                    return Ok(()); // Graceful exit
                },
                Err(e) => return Err(e), // Propagate other errors
            };

            match signal {
                Signal::Startup((id, _)) | Signal::Step((id, _)) | Signal::Shutdown((id, _)) => {
                    self.handle_activity_signal(&id, &signal)?;
                },
                Signal::StartupSync(sync_info) => {
                    timestamp::initialize_from(sync_info);
                },
                Signal::Terminate(_) => {
                    debug!(
                        "Worker {} received Terminate signal. Acknowledging and exiting.",
                        self.id
                    );
                    //connection reset may happen if primary terminated and closed its sockets
                    if let Err(e) = self.connector.send_to_scheduler(&Signal::TerminateAck(self.agent_id)) {
                        debug!(
                            "Worker {} failed to send TerminateAck (this is often expected during shutdown): {:?}",
                            self.id, e
                        );
                    }
                    debug!("Worker {} sent termination ack. Exiting.", self.id);
                    // Linger for a moment to ensure  TerminateAck has time to be sent
                    // over the network before the thread exits and closes the socket.
                    thread::sleep(Duration::from_millis(100).into());
                    return Ok(()); // Graceful exit
                },
                other => return Err(Error::UnexpectedSignal(other)),
            }
        }
    }

    fn handle_activity_signal(&mut self, id: &ActivityId, signal: &Signal) -> Result<(), Error> {
        let activity = self.activities.get_mut(id).ok_or(Error::ActivityNotFound(*id))?;
        let start = Instant::now();

        match signal {
            Signal::Startup((activity_id, _ts)) => {
                let response_signal = match activity.startup() {
                    Ok(()) => Signal::Ready((*activity_id, timestamp::timestamp())),
                    Err(e) => {
                        error!("Activity {} failed during startup: {:?}", id, e);
                        Signal::ActivityFailed((*id, e))
                    },
                };
                let elapsed = start.elapsed();
                debug!("Ran startup of activity {:?} in {:?}", id, elapsed);
                self.connector.send_to_scheduler(&response_signal)
            },
            Signal::Step((activity_id, _ts)) => {
                let response_signal = match activity.step() {
                    Ok(()) => Signal::Ready((*activity_id, timestamp::timestamp())),
                    Err(e) => {
                        error!("Activity {} failed during step: {:?}", id, e);
                        Signal::ActivityFailed((*id, e))
                    },
                };
                let elapsed = start.elapsed();
                debug!("Stepped activity {:?} in {:?}", id, elapsed);
                self.connector.send_to_scheduler(&response_signal)
            },
            Signal::Shutdown((activity_id, _ts)) => {
                let response_signal = match activity.shutdown() {
                    Ok(()) => Signal::Ready((*activity_id, timestamp::timestamp())),
                    Err(e) => {
                        error!("Activity {} failed during shutdown: {:?}", id, e);
                        Signal::ActivityFailed((*id, e))
                    },
                };
                let elapsed = start.elapsed();
                debug!("Ran shutdown of activity {:?} in {}", id, elapsed);
                self.connector.send_to_scheduler(&response_signal)
            },
            other => Err(Error::UnexpectedSignal(*other)),
        }
    }
}
