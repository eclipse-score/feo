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

//! Implementation of the primary agent for direct scheduler-to-worker signalling

use crate::activity::ActivityIdAndBuilder;
use crate::agent::NodeAddress;
use crate::error::Error;
use crate::ids::{ActivityId, AgentId, WorkerId};
use crate::scheduler::Scheduler;
use crate::signalling::common::interface::{ConnectScheduler, ConnectWorker};
use crate::signalling::direct::scheduler::{TcpSchedulerConnector, UnixSchedulerConnector};
use crate::signalling::direct::worker::{TcpWorkerConnector, UnixWorkerConnector};
use crate::timestamp;
use crate::worker::Worker;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::AtomicBool;
use core::time::Duration;
use feo_log::info;
use std::collections::HashMap;
use std::thread::{self, JoinHandle};

/// Configuration of the primary agent
pub struct PrimaryConfig {
    /// Id of the primary agent
    pub id: AgentId,
    /// Cycle time of the step loop
    pub cycle_time: Duration,
    /// Dependencies per activity
    pub activity_dependencies: HashMap<ActivityId, Vec<ActivityId>>,
    /// IDs of all recorders for which the scheduler waits
    pub recorder_ids: Vec<AgentId>,
    /// Worker assignments to be run in this agent
    pub worker_assignments: Vec<(WorkerId, Vec<ActivityIdAndBuilder>)>,
    /// Receive timeout of the scheduler's connector
    pub timeout: Duration,
    /// Timeout for waiting on initial connections from workers/recorders.
    pub connection_timeout: Duration,
    /// Timeout for waiting on activities to become ready during startup.
    pub startup_timeout: Duration,
    /// Endpoint on which the connector of the scheduler waits for connections
    pub endpoint: NodeAddress,
    /// Map of all activities to agent ids
    pub activity_agent_map: HashMap<ActivityId, AgentId>,
}

/// Primary agent
pub struct Primary {
    /// Scheduler
    scheduler: Scheduler,
    /// Handles to the worker threads
    worker_threads: Vec<JoinHandle<()>>,
}

impl Primary {
    /// Create a new instance
    pub fn new(config: PrimaryConfig) -> Result<Self, Error> {
        let PrimaryConfig {
            cycle_time,
            activity_dependencies,
            recorder_ids,
            endpoint,
            timeout,
            connection_timeout,
            startup_timeout,
            activity_agent_map,
            ..
        } = config;

        // Create worker threads first so that the connector of the scheduler can connect
        let worker_threads = config
            .worker_assignments
            .into_iter()
            .map(|(id, activities)| {
                let endpoint = endpoint.clone();
                let agent_id = config.id;
                thread::spawn(move || match endpoint {
                    NodeAddress::Tcp(addr) => {
                        let mut connector =
                            TcpWorkerConnector::new(addr, activities.iter().map(|(id, _)| *id));
                        connector.connect_remote().expect("failed to connect");

                        let activity_builders = activities;
                        let worker =
                            Worker::new(id, agent_id, activity_builders, connector, timeout);

                        if let Err(e) = worker.run() {
                            feo_log::error!("Worker {} in primary agent failed: {:?}", id, e);
                        }
                    }
                    NodeAddress::UnixSocket(path) => {
                        let mut connector =
                            UnixWorkerConnector::new(path, activities.iter().map(|(id, _)| *id));
                        connector.connect_remote().expect("failed to connect");

                        let activity_builders = activities;
                        let worker =
                            Worker::new(id, agent_id, activity_builders, connector, timeout);

                        if let Err(e) = worker.run() {
                            feo_log::error!("Worker {} in primary agent failed: {:?}", id, e);
                        }
                    }
                })
            })
            .collect();

        let mut connector = match endpoint {
            NodeAddress::Tcp(addr) => Box::new(TcpSchedulerConnector::new(
                addr,
                activity_dependencies.keys().cloned(),
                recorder_ids.iter().cloned(),
                activity_agent_map,
                connection_timeout,
            )) as Box<dyn ConnectScheduler>,
            NodeAddress::UnixSocket(path) => Box::new(UnixSchedulerConnector::new(
                &path,
                activity_dependencies.keys().cloned(),
                recorder_ids.iter().cloned(),
                activity_agent_map,
                connection_timeout,
            )) as Box<dyn ConnectScheduler>,
        };
        connector.connect_remotes()?;

        // Create a shared flag to signal shutdown from an OS signal (e.g., Ctrl-C).
        let shutdown_requested = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown_requested.clone();
        ctrlc::set_handler(move || {
            info!("Ctrl-C detected. Requesting graceful shutdown...");
            shutdown_clone.store(true, core::sync::atomic::Ordering::Relaxed);
        })
        .expect("Error setting Ctrl-C handler");

        let scheduler = Scheduler::new(
            config.id,
            cycle_time,
            timeout,
            startup_timeout,
            activity_dependencies,
            connector,
            recorder_ids,
            shutdown_requested,
        );

        Ok(Self {
            scheduler,
            worker_threads,
        })
    }

    /// Run the agent
    pub fn run(&mut self) -> Result<(), Error> {
        // Initialize local time
        timestamp::initialize();

        // Sync time on remotes
        self.scheduler.sync_remotes()?;

        // This will block until the scheduler decides to shut down.
        self.scheduler.run();

        // After the scheduler returns, we know the shutdown sequence has completed.
        // We can now safely join our local worker threads.
        for th in self.worker_threads.drain(..) {
            if let Err(e) = th.join() {
                feo_log::error!(
                    "A local worker thread in the primary agent panicked: {:?}",
                    e
                );
            }
        }

        Ok(())
    }
}
