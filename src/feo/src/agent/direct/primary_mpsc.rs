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

//! Implementation of the primary agent for mpsc-only signalling

use crate::activity::ActivityIdAndBuilder;
use crate::debug_fmt::ScoreDebugDebug;
use crate::error::Error;
use crate::ids::{ActivityId, AgentId, WorkerId};
use crate::scheduler::Scheduler;
use crate::signalling::common::interface::{ConnectScheduler, ConnectWorker};
use crate::signalling::direct::mpsc::scheduler::SchedulerConnector;
use crate::timestamp;
use crate::worker::Worker;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::AtomicBool;
use feo_time::Duration;
use score_log::{debug, error, info};
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
    /// Timeout for waiting on activities to become ready during startup.
    pub startup_timeout: Duration,
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
            timeout,
            startup_timeout,
            ..
        } = config;

        let activity_worker_map: HashMap<ActivityId, WorkerId> = config
            .worker_assignments
            .iter()
            .flat_map(|(wid, aid_bld)| aid_bld.iter().map(move |id_b| (id_b.0, *wid)))
            .collect();

        // Create scheduler connector
        let mut connector = Box::new(SchedulerConnector::new(activity_worker_map));

        // Get worker connector builders to be moved into worker threads
        let mut connector_builders = connector.worker_connector_builders();

        // Create worker threads first so that the connector of the scheduler can connect
        let worker_threads = config
            .worker_assignments
            .into_iter()
            .map(|(id, activities)| {
                let connector_builder = connector_builders.remove(&id).expect("missing connector builder");
                let agent_id = config.id;
                thread::spawn(move || {
                    let mut connector = connector_builder();
                    connector.connect_remote().expect("failed to connect");

                    let activity_builders = activities;
                    let worker = Worker::new(id, agent_id, activity_builders, connector, timeout);
                    worker.run().expect("failed to run worker");
                })
            })
            .collect();

        connector.connect_remotes().expect("failed to connect");

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

        // TODO: Bubble up errors
        self.scheduler.run();

        for th in self.worker_threads.drain(..) {
            if let Err(e) = th.join() {
                error!(
                    "A local worker thread in the primary agent panicked: {:?}",
                    ScoreDebugDebug::<_, 1024>(&e)
                );
            }
        }
        debug!("Primary agent finished");
        Ok(())
    }
}
