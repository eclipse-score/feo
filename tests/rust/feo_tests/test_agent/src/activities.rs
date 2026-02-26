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

use crate::config::TOPIC_COUNTER;
use crate::monitor::{MonitorNotification, MonitorRequest};
use crate::scenario::Counter;
use feo::activity::Activity;
use feo::error::ActivityError;
use feo::ids::ActivityId;
use feo_com::interface::{ActivityInput, ActivityOutput};
use feo_tracing::instrument;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use score_log::debug;
use score_log::fmt::ScoreDebug;
use std::ops::Deref;

/// Sender activity
///
/// This activity emulates a sender activity for testing.
#[derive(Debug)]
pub struct Sender {
    /// ID of the activity
    activity_id: ActivityId,
    /// Output
    output: Box<dyn ActivityOutput<Counter>>,
    /// Counter
    counter: usize,
}

impl Sender {
    pub fn build(activity_id: ActivityId) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            output: activity_output(TOPIC_COUNTER),
            counter: 0,
        })
    }
}

impl Activity for Sender {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    #[instrument(name = "Sender startup")]
    fn startup(&mut self) -> Result<(), ActivityError> {
        Ok(())
    }

    #[instrument(name = "Sender")]
    fn step(&mut self) -> Result<(), ActivityError> {
        debug!("Stepping Sender");
        if let Ok(counter) = self.output.write_uninit() {
            let camera = counter.write_payload(Counter { counter: self.counter });
            camera.send().unwrap();
        }
        self.counter += 1;
        Ok(())
    }

    #[instrument(name = "Sender shutdown")]
    fn shutdown(&mut self) -> Result<(), ActivityError> {
        Ok(())
    }
}

/// Receiver activity
///
/// Emulates a receiver activity for testing.
#[derive(Debug)]
pub struct Receiver {
    /// ID of the activity
    activity_id: ActivityId,
    /// Input
    input: Box<dyn ActivityInput<Counter>>,
    counter: usize,
}

impl Receiver {
    pub fn build(activity_id: ActivityId) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            input: activity_input(TOPIC_COUNTER),
            counter: 0,
        })
    }
}

impl Activity for Receiver {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    #[instrument(name = "Receiver startup")]
    fn startup(&mut self) -> Result<(), ActivityError> {
        Ok(())
    }

    #[instrument(name = "Receiver")]
    fn step(&mut self) -> Result<(), ActivityError> {
        debug!("Stepping Receiver");

        let counter = self.input.read().unwrap();
        if counter.counter != self.counter {
            panic!(
                "Recived unexpected value, expected {}, got {}",
                self.counter, counter.counter
            )
        }
        self.counter += 1;

        debug!("Received {:?}", counter.deref());
        Ok(())
    }

    #[instrument(name = "Receiver shutdown")]
    fn shutdown(&mut self) -> Result<(), ActivityError> {
        Ok(())
    }
}

/// Monitor activity
///
/// Connects to the testing app to server testing requests and provide
/// notifications on FEO behavior (setup/step/shutdown method calls etc.)
#[derive(Debug)]
pub struct Monitor {
    /// ID of the activity
    activity_id: ActivityId,
    /// Sender for notifications to testing app
    sender: IpcSender<MonitorNotification>,
    /// Receiver for requests from testing app
    receiver: IpcReceiver<MonitorRequest>,
}

impl Monitor {
    pub fn build(activity_id: ActivityId, server_name: String) -> Box<dyn Activity> {
        let (request_tx, request_rx): (IpcSender<MonitorRequest>, IpcReceiver<MonitorRequest>) =
            ipc::channel().unwrap();
        let (notification_tx, notification_rx): (IpcSender<MonitorNotification>, IpcReceiver<MonitorNotification>) =
            ipc::channel().unwrap();
        let sender = IpcSender::connect(server_name).unwrap();
        sender.send((request_tx, notification_rx)).unwrap();
        Box::new(Self {
            activity_id,
            sender: notification_tx,
            receiver: request_rx,
        })
    }
}

impl Activity for Monitor {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    #[instrument(name = "Monitor startup")]
    fn startup(&mut self) -> Result<(), ActivityError> {
        // Notify testing app of a startup call
        self.sender.send(MonitorNotification::Startup).unwrap();
        Ok(())
    }

    #[instrument(name = "Monitor")]
    fn step(&mut self) -> Result<(), ActivityError> {
        debug!("Stepping Monitor");
        // Notify testing app of a step call
        self.sender.send(MonitorNotification::Step).unwrap();
        if let Ok(r) = self.receiver.try_recv() {
            // Handing testing app requests
            match r {}
        }
        Ok(())
    }

    #[instrument(name = "Monitor shutdown")]
    fn shutdown(&mut self) -> Result<(), ActivityError> {
        // Notify testing app of a shutdown call
        self.sender.send(MonitorNotification::Shutdown).unwrap();
        Ok(())
    }
}

/// Create an activity input.
fn activity_input<T>(topic: &str) -> Box<dyn ActivityInput<T>>
where
    T: core::fmt::Debug + ScoreDebug + 'static,
{
    #[cfg(feature = "com_iox2")]
    {
        Box::new(Iox2Input::new(topic))
    }
    #[cfg(feature = "com_linux_shm")]
    {
        Box::new(feo_com::linux_shm::LinuxShmInput::new(topic))
    }
}

/// Create an activity output.
fn activity_output<T>(topic: &str) -> Box<dyn ActivityOutput<T>>
where
    T: core::fmt::Debug + ScoreDebug + 'static,
{
    #[cfg(feature = "com_iox2")]
    {
        Box::new(Iox2Output::new(topic))
    }
    #[cfg(feature = "com_linux_shm")]
    {
        Box::new(feo_com::linux_shm::LinuxShmOutput::new(topic))
    }
}
