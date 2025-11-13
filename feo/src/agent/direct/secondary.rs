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

//! Implementation of a secondary agent for direct scheduler-to-worker signalling

use crate::activity::ActivityIdAndBuilder;
use crate::agent::NodeAddress;
use crate::ids::{AgentId, WorkerId};
use crate::signalling::common::interface::ConnectWorker;
use crate::signalling::direct::worker::{TcpWorkerConnector, UnixWorkerConnector};
use crate::worker::Worker;
use alloc::vec::Vec;
use core::time::Duration;
use feo_log::{debug, error};
use std::thread::{self, JoinHandle};

/// Configuration of a secondary agent
pub struct SecondaryConfig {
    /// ID of this agent
    pub id: AgentId,
    /// Activity IDs with builders to run per [WorkerId]
    pub worker_assignments: Vec<(WorkerId, Vec<ActivityIdAndBuilder>)>,
    /// Maximum time for a worker to make no progress without panicking
    pub timeout: Duration,
    /// Endpoint on which the scheduler connector is listening
    pub endpoint: NodeAddress,
}

/// Secondary agent
pub struct Secondary {
    /// ID
    id: AgentId,
    /// Handles to the worker threads
    worker_threads: Vec<JoinHandle<()>>,
}

impl Secondary {
    /// Create a new instance
    pub fn new(config: SecondaryConfig) -> Self {
        let SecondaryConfig {
            id: _,
            worker_assignments,
            timeout,
            endpoint,
        } = config;

        let worker_threads = worker_assignments
            .into_iter()
            .map(|(id, activities)| {
                let endpoint = endpoint.clone();
                let agent_id = config.id; // Use the correct AgentId from the config.
                thread::spawn(move || match endpoint {
                    NodeAddress::Tcp(addr) => {
                        let mut connector =
                            TcpWorkerConnector::new(addr, activities.iter().map(|(id, _)| *id));
                        if let Err(e) = connector.connect_remote() {
                            error!("Worker {} failed to connect to primary: {:?}", id, e);
                            return;
                        }
                        let worker = Worker::new(id, agent_id, activities, connector, timeout);
                        if let Err(e) = worker.run() {
                            error!("Worker {} failed with error: {:?}", id, e);
                        }
                    }
                    NodeAddress::UnixSocket(path) => {
                        let mut connector =
                            UnixWorkerConnector::new(path, activities.iter().map(|(id, _)| *id));
                        if let Err(e) = connector.connect_remote() {
                            error!("Worker {} failed to connect to primary: {:?}", id, e);
                            return;
                        }
                        let worker = Worker::new(id, agent_id, activities, connector, timeout);
                        if let Err(e) = worker.run() {
                            error!("Worker {} failed with error: {:?}", id, e);
                        }
                    }
                })
            })
            .collect();

        Self {
            id: config.id,
            worker_threads,
        }
    }

    /// Run the agent
    pub fn run(self) {
        debug!("Running secondary with ID {:?}", self.id);

        for th in self.worker_threads {
            if let Err(e) = th.join() {
                error!("Worker thread for agent {:?} panicked: {:?}", self.id, e);
            }
        }

        debug!("Secondary with ID {:?} finished", self.id);
    }
}
