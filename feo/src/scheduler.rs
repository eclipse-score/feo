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

//! Global activity scheduler

use crate::error::Error;
use crate::ids::{ActivityId, AgentId};
use crate::signalling::common::interface::ConnectScheduler;
use crate::signalling::common::signals::Signal;
use crate::timestamp::timestamp;
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::{boxed::Box, collections::BTreeSet};
use core::sync::atomic::{AtomicBool, Ordering};
use feo_log::{debug, error, info, trace};
use feo_time::Instant;
use std::collections::HashMap;
use std::thread;

/// Global activity scheduler
///
/// The scheduler (aka 'FEO Executor') executes the FEO activities according to the defined order.
pub(crate) struct Scheduler {
    /// The ID of the agent this scheduler is running on.
    agent_id: AgentId,
    /// Target duration of a task chain cycle
    cycle_time: feo_time::Duration,
    /// Timeout of receive function
    receive_timeout: core::time::Duration,
    /// Timeout for waiting on activities to become ready during startup.
    startup_timeout: core::time::Duration,
    /// For each activity: list of activities it depends on
    activity_depends: HashMap<ActivityId, Vec<ActivityId>>,
    /// Map keeping track of activity states
    activity_states: HashMap<ActivityId, ActivityState>,

    /// Helper object connecting to activities in all connected agents
    connector: Box<dyn ConnectScheduler>,

    /// Agent IDs of expected recorders
    recorder_ids: Vec<AgentId>,
    /// Map from recorder agent ID to ready state
    recorders_ready: HashMap<AgentId, bool>,
    /// Flag to signal a shutdown request from an external source (e.g., Ctrl-C).
    shutdown_requested: Arc<AtomicBool>,
}

impl Scheduler {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        agent_id: AgentId,
        feo_cycle_time: feo_time::Duration,
        receive_timeout: core::time::Duration,
        startup_timeout: core::time::Duration,
        activity_depends: HashMap<ActivityId, Vec<ActivityId>>,
        connector: Box<dyn ConnectScheduler>,
        recorder_ids: Vec<AgentId>,
        shutdown_requested: Arc<AtomicBool>,
    ) -> Self {
        // Pre-allocate state map
        let activity_states: HashMap<ActivityId, ActivityState> = activity_depends
            .keys()
            .map(|k| {
                (
                    *k,
                    ActivityState {
                        triggered: false,
                        ready: false,
                        ever_ready: false,
                    },
                )
            })
            .collect();
        let recorders_ready = recorder_ids.iter().map(|id| (*id, false)).collect();

        Self {
            agent_id,
            cycle_time: feo_cycle_time,
            receive_timeout,
            startup_timeout,
            activity_depends,
            connector,
            activity_states,
            recorder_ids,
            recorders_ready,
            shutdown_requested,
        }
    }

    /// Synchronize all remote agents and recorders
    pub(crate) fn sync_remotes(&mut self) -> Result<(), Error> {
        self.connector.sync_time()?;
        info!("Time synchronization of remote agents done");
        Ok(())
    }

    /// Run the task lifecycle, i.e. startup, stepping, shutdown
    pub(crate) fn run(&mut self) {
        #[cfg(feature = "loop_duration_meter")]
        let mut meter = loop_duration_meter::LoopDurationMeter::<1000>::default();

        // Sort activity ids
        let mut activity_ids: Vec<_> = self.activity_states.keys().collect();
        activity_ids.sort();

        // Call startup on all activities sorted according to their ids
        // Note: Actual startup may occur in different order, depending on the assignment
        // of activities to worker threads. (A worker with greater id value may start up in
        // one thread before an activity with smaller id value in another thread.)
        for activity_id in activity_ids {
            Self::startup_activity(activity_id, &self.recorder_ids, &mut self.connector).unwrap();
        }

        // Wait until all activities have returned their ready signal, with a timeout.
        let startup_start = Instant::now();
        while !self.all_ready() {
            if startup_start.elapsed() > self.startup_timeout {
                let reason = alloc::format!(
                    "Startup timeout of {:?} exceeded. Not all activities became ready.",
                    self.startup_timeout
                );
                error!("{}", reason);
                self.shutdown_gracefully(&reason);
                return;
            }
            if self.wait_next_ready().is_err() {
                // An error here (like a timeout on receive) can also be a startup failure.
                let reason = alloc::format!(
                    "Failed to receive ready signal from all activities within startup timeout {:?}.",
                    self.startup_timeout
                );
                error!("{}", reason);
                self.shutdown_gracefully(&reason);
                return;
            }
        }

        // Loop the FEO task chain
        loop {
            let task_chain_start = Instant::now();

            // Record start of task chain on registered recorders
            self.record_task_chain_start().unwrap();

            // Clear ready and triggered signals
            self.activity_states.values_mut().for_each(|v| {
                v.ready = false;
                v.triggered = false;
            });

            debug!("Starting task chain");

            while !self.all_ready() {
                // Step all activities that have their dependencies met
                self.step_ready_activities();
                // Wait until a new ready signal has been received.
                // If we receive an error (i.e., an ActivityFailed signal), proceed to graceful shutdown.
                if let Err(e) = self.wait_next_ready() {
                    let reason =
                        alloc::format!("A failure occurred during step execution: {:?}", e);
                    error!("{}", &reason);
                    self.shutdown_gracefully(&reason);
                    return;
                }
            }

            // Record end of task chain on registered recorders => recorders will flush
            // => wait until all recorders have signalled to be ready
            trace!("Flushing recorders");
            let start_flush = Instant::now();
            self.record_task_chain_end().unwrap();
            self.wait_recorders_ready().unwrap();
            let flush_duration = start_flush.elapsed();
            trace!("Flushing recorders took {flush_duration:?}");

            let task_chain_duration = task_chain_start.elapsed();

            #[cfg(feature = "loop_duration_meter")]
            meter.track(&task_chain_duration);

            let time_left = self.cycle_time.saturating_sub(task_chain_duration);
            if time_left.is_zero() {
                error!(
                    "Finished task chain after {task_chain_duration:?}. Expected to be less than {:?}",
                    self.cycle_time
                );
            } else {
                debug!(
                    "Finished task chain after {task_chain_duration:?}. Sleeping for {time_left:?}"
                );
                thread::sleep(time_left);
            }

            // Check for an external shutdown request (e.g., from Ctrl-C).
            if self.shutdown_requested.load(Ordering::Relaxed) {
                info!("External shutdown signal received, initiating graceful shutdown.");
                break;
            }
        } // end loop

        // Once the loop is broken, always perform a graceful shutdown.
        self.shutdown_gracefully("Main loop concluded or external signal received.");
    }

    /// Step all activities whose dependencies have signalled ready
    fn step_ready_activities(&mut self) {
        // Get data from activity_depends in self so that we can iterate over it
        // and at the same time modify another member of self
        for (act_id, dependencies) in self.activity_depends.iter() {
            // skip activity if already triggered
            if self.activity_states[act_id].triggered {
                continue;
            }

            // If dependencies are fulfilled
            let is_ready = self
                .activity_states
                .iter()
                .filter(|(id, _)| dependencies.contains(id))
                .all(|(_, state)| state.ready);
            if is_ready {
                Self::step_activity(act_id, &self.recorder_ids, &mut self.connector)
                    .expect("failed to step activity");
                self.activity_states.get_mut(act_id).unwrap().triggered = true;
            }
        }
    }

    /// Send startup signal to the given activity
    fn startup_activity(
        id: &ActivityId,
        recorder_ids: &[AgentId],
        connector: &mut Box<dyn ConnectScheduler>,
    ) -> Result<(), Error> {
        debug!("Triggering startup for activity {}", id);
        let signal = Signal::Startup((*id, timestamp()));
        Self::trigger_activity(id, &signal, recorder_ids, connector)
    }

    /// Send step signal to the given activity
    fn step_activity(
        id: &ActivityId,
        recorder_ids: &[AgentId],
        connector: &mut Box<dyn ConnectScheduler>,
    ) -> Result<(), Error> {
        debug!("Triggering step for activity {}", id);
        let signal = Signal::Step((*id, timestamp()));
        Self::trigger_activity(id, &signal, recorder_ids, connector)
    }

    /// Send shutdown signal to the given activity
    fn shutdown_activity(
        id: &ActivityId,
        recorder_ids: &[AgentId],
        connector: &mut Box<dyn ConnectScheduler>,
    ) -> Result<(), Error> {
        debug!("Triggering shutdown for activity {}", id);
        let signal = Signal::Shutdown((*id, timestamp()));
        Self::trigger_activity(id, &signal, recorder_ids, connector)
    }

    /// Manages the graceful shutdown of all started activities and agents.
    /// This function is intended to be called when the application needs to exit.
    // #[allow(dead_code)] // This is called from the main binary, not within the library.
    pub(crate) fn shutdown_gracefully(&mut self, reason: &str) {
        info!("Shutting down... Reason: {}", reason);

        // --- PHASE 1: Shut down started activities ---

        // 1. Identify which activities have successfully started.
        // We consider any activity that has ever been ready as "started".
        let started_activities: BTreeSet<_> = self
            .activity_states
            .iter()
            .filter(|(_, state)| state.ever_ready)
            .map(|(id, _)| *id)
            .collect();

        if !started_activities.is_empty() {
            // 2. Send a shutdown signal to only the started activities.
            info!(
                "Sending Shutdown signal to started activities: {:?}",
                started_activities
            );
            for activity_id in &started_activities {
                Self::shutdown_activity(activity_id, &self.recorder_ids, &mut self.connector)
                    .unwrap_or_else(|e| {
                        error!(
                            "Failed to send Shutdown to activity {}: {:?}",
                            activity_id, e
                        )
                    });
            }

            // 3. Wait for confirmation from the activities that were told to shut down.
            // A worker sends a `Ready` signal after completing its shutdown.
            let mut pending_shutdown_ack = started_activities;
            info!(
                "Waiting for shutdown confirmation from: {:?}",
                pending_shutdown_ack
            );

            let shutdown_timeout = self.receive_timeout * (pending_shutdown_ack.len() as u32 + 2);
            let start = Instant::now();

            while !pending_shutdown_ack.is_empty() {
                if start.elapsed() > shutdown_timeout {
                    error!(
                        "Timeout waiting for shutdown confirmation. Still waiting for: {:?}",
                        pending_shutdown_ack
                    );
                    break;
                }
                match self.connector.receive(self.receive_timeout) {
                    Ok(Some(Signal::Ready((id, _)))) => {
                        if pending_shutdown_ack.remove(&id) {
                            info!("Received shutdown confirmation from activity {:?}", id);
                        }
                    }
                    Ok(Some(Signal::ActivityFailed((id, err)))) => {
                        // This handles "Activity shutdown error".
                        error!(
                            "Activity {} failed during shutdown: {:?}. Continuing.",
                            id, err
                        );
                        // Remove it from the pending list so we don't wait forever.
                        pending_shutdown_ack.remove(&id);
                    }
                    Ok(_) => {} // Ignore other signals or timeouts
                    Err(e) => error!("Error receiving shutdown confirmation: {:?}", e),
                }
            }
        } else {
            info!("No activities were successfully started. Skipping activity shutdown.");
        }

        // --- PHASE 2: Terminate all agents ---
        self.terminate_all_agents();
    }

    fn terminate_all_agents(&mut self) {
        // Broadcast Terminate signal to all agents.
        info!("Broadcasting Terminate signal to all agents.");
        if let Err(e) = self
            .connector
            .broadcast_terminate(&Signal::Terminate(timestamp()))
        {
            error!("Failed to broadcast Terminate signal: {:?}", e);
        }

        // Wait for TerminateAck from all connected agents.
        // We only wait for remote agents, so we filter out our own agent ID.
        let mut pending_agent_acks: BTreeSet<_> = self
            .connector
            .get_connected_agent_ids()
            .into_iter()
            .filter(|id| *id != self.agent_id)
            .collect();
        if pending_agent_acks.is_empty() {
            info!("No remote agents to wait for. Agent shutdown complete.");
            return;
        }

        info!(
            "Waiting for TerminateAck from agents: {:?}",
            pending_agent_acks
        );
        let agent_ack_timeout = self.receive_timeout * (pending_agent_acks.len() as u32 + 4);
        let start = Instant::now();

        while !pending_agent_acks.is_empty() {
            if start.elapsed() > agent_ack_timeout {
                error!(
                    "Timeout waiting for TerminateAck. Still waiting for: {:?}",
                    pending_agent_acks
                );
                break;
            }
            if let Ok(Some(Signal::TerminateAck(agent_id))) =
                self.connector.receive(self.receive_timeout)
            {
                if pending_agent_acks.remove(&agent_id) {
                    info!("Received TerminateAck from agent {}", agent_id);
                }
            }
        }

        info!("Finished waiting for all acknowledgements. Shutdown complete.");
    }

    /// Trigger activity by forwarding the signal to the activity and all recorders
    fn trigger_activity(
        id: &ActivityId,
        signal: &Signal,
        recorder_ids: &[AgentId],
        connector: &mut Box<dyn ConnectScheduler>,
    ) -> Result<(), Error> {
        connector.send_to_activity(*id, signal)?;
        for id in recorder_ids {
            connector.send_to_recorder(*id, signal)?;
        }
        Ok(())
    }

    /// Wait for the next incoming ready signal
    fn wait_next_ready(&mut self) -> Result<ActivityId, Error> {
        // Wait for next intra-process ready signal from one of the workers
        let activity_id = loop {
            match self.connector.receive(self.receive_timeout)? {
                None => {
                    return Err(Error::Timeout(
                        self.receive_timeout,
                        "waiting for ready signal",
                    ))
                }
                Some(signal @ Signal::Ready((id, _))) => {
                    for recorder_id in self.recorder_ids.iter() {
                        self.connector.send_to_recorder(*recorder_id, &signal)?;
                    }
                    break id;
                }
                Some(Signal::ActivityFailed((id, err))) => {
                    error!(
                        "Received failure signal {:?} from activity {}. Initiating graceful shutdown.",
                        err, id
                    );
                    return Err(Error::ActivityFailed(id, err));
                }
                Some(Signal::TerminateAck(agent_id)) => {
                    trace!(
                        "Ignoring TerminateAck from agent {} during normal operation",
                        agent_id
                    );
                    continue;
                }
                Some(other) => {
                    error!("Received unexpected signal {other:?} while waiting for ready signal");
                }
            }
        };

        // Set corresponding ready flag
        let state = self.activity_states.get_mut(&activity_id).unwrap();
        state.ready = true;
        state.ever_ready = true;
        Ok(activity_id)
    }

    /// Check if all activities have signalled 'ready'
    fn all_ready(&self) -> bool {
        self.activity_states.values().all(|v| v.ready)
    }

    fn record_task_chain_start(&mut self) -> Result<(), Error> {
        trace!("Recording task chain start");
        let signal = &Signal::TaskChainStart(timestamp());
        for id in self.recorder_ids.iter() {
            self.connector.send_to_recorder(*id, signal)?;
        }
        Ok(())
    }

    fn record_task_chain_end(&mut self) -> Result<(), Error> {
        trace!("Recording task chain end");
        let signal = &Signal::TaskChainEnd(timestamp());
        for id in self.recorder_ids.iter() {
            self.connector.send_to_recorder(*id, signal)?;
        }
        Ok(())
    }

    fn wait_recorders_ready(&mut self) -> Result<(), Error> {
        // If there are no recorders registered, return immediately
        if self.recorder_ids.is_empty() {
            return Ok(());
        }

        // Clear all ready flags
        for value in self.recorders_ready.values_mut() {
            *value = false;
        }

        // Loop until all recorders have signalled ready
        while !self.recorders_ready.values().all(|v| *v) {
            let received = self.connector.receive(self.receive_timeout)?;
            match received {
                None => continue,
                Some(Signal::RecorderReady((id, _))) => {
                    if let Some(value) = self.recorders_ready.get_mut(&id) {
                        *value = true;
                    } else {
                        error!("Received unexpected id {id} in recorder ready signal");
                    }
                }
                Some(other) => {
                    error!("Received unexpected signal {other} while waiting for recorder ready signal");
                }
            }
        }

        Ok(())
    }
}

/// Current state of an activity
struct ActivityState {
    /// Whether the activity has been triggered for an action
    triggered: bool,

    /// Whether the activity has finished its previously triggered operation
    ready: bool,
    /// Whether the activity has ever been ready (i.e., has started)
    ever_ready: bool,
}

#[cfg(feature = "loop_duration_meter")]
mod loop_duration_meter {
    use std::println;

    /// Meter to track and periodically print average durations
    #[derive(Default)]
    pub(super) struct LoopDurationMeter<const NUM_STEPS: usize> {
        duration_micros: usize,
        num_steps: usize,
    }

    impl<const NUM_STEPS: usize> LoopDurationMeter<NUM_STEPS> {
        pub fn track(&mut self, duration: &feo_time::Duration) {
            self.duration_micros +=
                duration.subsec_micros() as usize + 1000000 * duration.as_secs() as usize;
            self.num_steps += 1;

            if self.num_steps == NUM_STEPS {
                let avg_micros = self.duration_micros / NUM_STEPS;
                println!("Loop duration {avg_micros}µs (average of {NUM_STEPS} steps)");
                self.duration_micros = 0;
                self.num_steps = 0;
            }
        }
    }
}
