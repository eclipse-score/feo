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

//! Relays transferring signals between intra- and inter-process channels

use crate::error::Error;
use crate::ids::{ActivityId, AgentId, ChannelId, WorkerId};
use crate::signalling::common::signals::Signal;
use crate::signalling::relayed::interface::{
    Builder, IsChannel, ProtocolMultiRecv, ProtocolMultiSend, ProtocolRecv, ProtocolSend,
};
use crate::timestamp;
use crate::timestamp::sync_info;
use core::time::Duration;
use feo_log::{debug, error, trace};
use feo_time::Instant;
use std::collections::{HashMap, HashSet};
use std::{io::ErrorKind, thread};

/// Relay for the primary agent to receive signals from secondary agents
pub struct PrimaryReceiveRelay<Inter: IsChannel, Intra: IsChannel> {
    intra_sender_builder: Option<Builder<Intra::Sender>>,
    inter_receiver_builder: Option<Builder<Inter::MultiReceiver>>,
    timeout: Duration,
    _thread: Option<thread::JoinHandle<()>>,
}

impl<Inter: IsChannel, Intra: IsChannel> PrimaryReceiveRelay<Inter, Intra> {
    pub fn new(
        intra_sender_builder: Builder<Intra::Sender>,
        inter_receiver_builder: Builder<Inter::MultiReceiver>,
        timeout: Duration,
    ) -> Self {
        Self {
            intra_sender_builder: Some(intra_sender_builder),
            inter_receiver_builder: Some(inter_receiver_builder),
            timeout,
            _thread: None,
        }
    }

    pub fn run_and_connect(&mut self) -> thread::JoinHandle<()> {
        let inter_receiver_builder = self.inter_receiver_builder.take().unwrap();
        let intra_sender_builder = self.intra_sender_builder.take().unwrap();
        let timeout = self.timeout;

        thread::spawn(move || {
            if let Err(e) = Self::thread_main(inter_receiver_builder, intra_sender_builder, timeout)
            {
                // This error is expected during shutdown when the scheduler drops its receiver.
                debug!("[PrimaryReceiveRelay] thread terminated: {:?}", e);
            }
        })
    }

    fn thread_main(
        inter_receiver_builder: Builder<Inter::MultiReceiver>,
        intra_sender_builder: Builder<Intra::Sender>,
        timeout: Duration,
    ) -> Result<(), Error> {
        trace!("PrimaryReceiveRelay thread started");
        let (mut inter_receiver, mut intra_sender) =
            Self::connect(inter_receiver_builder, intra_sender_builder, timeout)
                .expect("PrimaryReceiveRelay not connected");
        loop {
            // Receive from remote workers on inter-process receiver
            let signal = inter_receiver.receive(timeout);

            let signal = match signal {
                Ok(Some(signal)) => signal,
                Ok(None) => {
                    error!("[PrimaryReceiveRelay]Reception timed out");
                    continue;
                }
                Err(Error::ChannelClosed) => {
                    debug!("[PrimaryReceiveRelay]Channel closed. Exiting.");
                    return Ok(());
                }
                Err(Error::Io((e, _))) if e.kind() == ErrorKind::ConnectionReset => {
                    // A single client disconnected. This is expected during shutdown.
                    // Log it and continue listening for other clients.
                    debug!("[PrimaryReceiveRelay]A remote agent connection was reset.");
                    return Ok(());
                }

                Err(e) => {
                    error!(
                        "[PrimaryReceiveRelay]Fatal error during receive: {:?}. Exiting.",
                        e
                    );
                    return Err(e);
                }
            };

            let signal: Signal = match signal.try_into() {
                Ok(signal) => signal,
                Err(_) => {
                    error!(
                        "[PrimaryReceiveRelay]Received unexpected signal {:?}",
                        signal
                    );
                    continue;
                }
            };

            // Forward onto intra-process connection.
            let protocol_signal: Intra::ProtocolSignal = signal.into();
            intra_sender.send(protocol_signal)?;
        }
    }

    fn connect(
        inter_receiver_builder: Builder<Inter::MultiReceiver>,
        intra_sender_builder: Builder<Intra::Sender>,
        timeout: Duration,
    ) -> Result<(Inter::MultiReceiver, Intra::Sender), Error> {
        trace!("PrimaryReceiveRelay connecting...");
        let start_time = Instant::now();
        let mut inter_receiver = inter_receiver_builder();
        let mut intra_sender = intra_sender_builder();
        inter_receiver.connect_senders(timeout)?;

        let elapsed = start_time.elapsed();
        let remaining_timeout = timeout.saturating_sub(elapsed);
        intra_sender.connect_receiver(remaining_timeout)?;
        trace!("PrimaryReceiveRelay connected");
        Ok((inter_receiver, intra_sender))
    }
}

/// Relay for a secondary to receive signals from the primary agent
pub struct SecondaryReceiveRelay<Inter: IsChannel, Intra: IsChannel> {
    inter_receiver_builder: Option<Builder<Inter::Receiver>>,
    intra_sender_builder: Option<Builder<Intra::MultiSender>>,
    activity_worker_map: HashMap<ActivityId, WorkerId>,
    timeout: Duration,
    _thread: Option<thread::JoinHandle<()>>,
}

impl<Inter: IsChannel, Intra: IsChannel> SecondaryReceiveRelay<Inter, Intra> {
    pub fn new(
        activity_worker_map: HashMap<ActivityId, WorkerId>,
        intra_sender_builder: Builder<<Intra as IsChannel>::MultiSender>,
        inter_receiver_builder: Builder<<Inter as IsChannel>::Receiver>,
        timeout: Duration,
    ) -> Self {
        Self {
            inter_receiver_builder: Some(inter_receiver_builder),
            intra_sender_builder: Some(intra_sender_builder),
            activity_worker_map,
            timeout,
            _thread: None,
        }
    }

    pub fn run_and_connect(&mut self) {
        let inter_receiver_builder = self.inter_receiver_builder.take().unwrap();
        let intra_sender_builder = self.intra_sender_builder.take().unwrap();
        let activity_worker_map = self.activity_worker_map.clone();
        let timeout = self.timeout;
        let thread = thread::spawn(move || {
            Self::thread_main(
                intra_sender_builder,
                inter_receiver_builder,
                activity_worker_map,
                timeout,
            )
        });
        self._thread = Some(thread);
    }

    fn receive_helper(
        receiver: &mut Inter::Receiver,
        timeout: Duration,
    ) -> Result<Option<<Inter as IsChannel>::ProtocolSignal>, Error> {
        let received = receiver.receive(timeout);
        match received {
            Ok(Some(s)) => Ok(Some(s)),
            Ok(None) => {
                error!("[SecondaryReceiveRelay]Reception timed out");
                Ok(None)
            }
            Err(Error::ChannelClosed) => Err(Error::ChannelClosed),
            Err(Error::Io((e, _))) if e.kind() == ErrorKind::ConnectionReset => {
                debug!("SecondaryReceiveRelay detected connection to primary reset (expected during shutdown)");
                Err(Error::Io((e, "connection reset")))
            }
            Err(e) => {
                error!("Failed to receive: {:?}", e);
                Err(e)
            }
        }
    }

    fn thread_main(
        intra_sender_builder: Builder<Intra::MultiSender>,
        inter_receiver_builder: Builder<Inter::Receiver>,
        activity_worker_map: HashMap<ActivityId, WorkerId>,
        timeout: Duration,
    ) {
        trace!("SecondaryReceiveRelay thread started");
        let mut intra_sender = intra_sender_builder();
        let mut inter_receiver = inter_receiver_builder();

        intra_sender
            .connect_receivers(timeout)
            .expect("failed to connect intra-process sender");
        inter_receiver
            .connect_sender(timeout)
            .expect("failed to connect inter-process receiver");
        trace!("SecondaryReceiveRelay connected, waiting for time synchronization");

        // Wait for startup sync
        loop {
            let protocol_signal = match Self::receive_helper(&mut inter_receiver, timeout) {
                Ok(Some(s)) => s,
                Ok(None) => {
                    // Timeout is logged by helper, just continue waiting.
                    continue;
                }
                Err(e) => {
                    error!(
                        "[SecondaryReceiveRelay]Fatal error during startup sync: {:?}. Exiting.",
                        e
                    );
                    return;
                }
            };

            let sync_info = match protocol_signal.try_into() {
                Ok(Signal::StartupSync(s)) => s,
                Ok(_) | Err(_) => {
                    error!("[SecondaryReceiveRelay]Unexpected signal {protocol_signal:?}");
                    continue;
                }
            };

            timestamp::initialize_from(sync_info);
            break;
        }
        debug!("Time synchronization done");

        loop {
            let protocol_signal = match Self::receive_helper(&mut inter_receiver, timeout) {
                Ok(Some(s)) => s,
                Ok(None) => {
                    // Timeout is logged by helper, just continue waiting.
                    continue;
                }
                Err(Error::ChannelClosed) => {
                    debug!("[SecondaryReceiveRelay]Connection to primary lost. Initiating self-shutdown.");
                    return; // Exit the relay thread. The workers will detect the closed channel.
                }
                Err(Error::Io((e, _))) if e.kind() == ErrorKind::ConnectionReset => {
                    debug!("[SecondaryReceiveRelay]Connection to primary was reset (expected during shutdown). Exiting.");
                    return; // Graceful exit
                }
                Err(e) => {
                    error!(
                        "[SecondaryReceiveRelay]Fatal error during receive: {:?}. Exiting.",
                        e
                    );
                    return;
                }
            };

            let core_signal: Signal = match protocol_signal.try_into() {
                Ok(signal) => signal,
                Err(_) => {
                    error!("[SecondaryReceiveRelay]Received unexpected signal {protocol_signal:?}");
                    continue;
                }
            };

            // Handle targeted signals vs. broadcast signals
            match core_signal {
                Signal::Startup((act_id, _))
                | Signal::Step((act_id, _))
                | Signal::Shutdown((act_id, _)) => {
                    // This is a targeted signal for a specific activity.
                    // Lookup corresponding worker id.
                    let Some(worker_id) = activity_worker_map.get(&act_id) else {
                        error!("[SecondaryReceiveRelay]Received unexpected activity id {act_id:?} in {core_signal:?}");
                        continue;
                    };

                    // Forward signal to the specific worker.
                    let channel_id = ChannelId::Worker(*worker_id);
                    let protocol_signal: Intra::ProtocolSignal = core_signal.into();
                    if let Err(e) = intra_sender.send(channel_id, protocol_signal) {
                        error!(
                            "Failed to send signal {:?} to worker {}: {:?}",
                            protocol_signal, worker_id, e
                        );
                    }
                }
                Signal::Terminate(_) | Signal::StartupSync(_) => {
                    // This is a broadcast signal for all local workers.
                    debug!(
                        "[SecondaryReceiveRelay]Broadcasting {:?} signal to all local workers.",
                        core_signal
                    );
                    let protocol_signal: Intra::ProtocolSignal = core_signal.into();
                    if let Err(e) = intra_sender.broadcast(protocol_signal) {
                        error!("Failed to broadcast signal to local workers: {:?}", e);
                    }
                }
                other => {
                    error!("[SecondaryReceiveRelay]Received unexpected signal '{:?}' from primary, discarding.", other);
                }
            }
        }
    }
}

/// Relay for the primary agent to send signals to secondary agents and recorders
pub struct PrimarySendRelay<Inter: IsChannel> {
    /// Set of all remote agents (secondaries and recorders)
    remote_agents: HashSet<AgentId>,
    /// Inter-process sender
    inter_sender: Inter::MultiSender,
    /// Connecting timeout
    timeout: Duration,
}

impl<Inter: IsChannel> PrimarySendRelay<Inter> {
    pub fn new(
        remote_agents: HashSet<AgentId>,
        inter_sender: Inter::MultiSender,
        timeout: Duration,
    ) -> Self {
        Self {
            remote_agents,
            inter_sender,
            timeout,
        }
    }

    pub fn connect(&mut self) -> Result<(), Error> {
        self.inter_sender.connect_receivers(self.timeout)?;
        trace!("PrimarySendRelay connected");
        Ok(())
    }

    pub fn get_remote_agents(&self) -> impl Iterator<Item = AgentId> + '_ {
        self.remote_agents.iter().copied()
    }

    pub fn send_to_agent(
        &mut self,
        agent_id: AgentId,
        signal: Inter::ProtocolSignal,
    ) -> Result<(), Error> {
        let channel_id = ChannelId::Agent(agent_id);
        self.inter_sender.send(channel_id, signal)
    }

    pub fn sync_time(&mut self) -> Result<(), Error> {
        let signal = Signal::StartupSync(sync_info());

        // Send sync info to all remote agents
        for id in self.remote_agents.iter() {
            let channel_id = ChannelId::Agent(*id);
            self.inter_sender.send(channel_id, signal.into())?;
        }
        Ok(())
    }
    pub fn broadcast(&mut self, signal: Inter::ProtocolSignal) -> Result<(), Error> {
        // Send signal to all remote agents
        for id in self.remote_agents.iter() {
            let channel_id = ChannelId::Agent(*id);
            self.inter_sender.send(channel_id, signal)?;
        }
        Ok(())
    }
}

/// Relay for a secondary to send signals to the primary agent
pub struct SecondarySendRelay<Inter: IsChannel, Intra: IsChannel> {
    inter_sender: Inter::Sender,
    intra_receiver: Intra::MultiReceiver,
    timeout: Duration,
}

impl<Inter: IsChannel, Intra: IsChannel> SecondarySendRelay<Inter, Intra> {
    pub fn new(
        inter_sender: Inter::Sender,
        intra_receiver: Intra::MultiReceiver,
        timeout: Duration,
    ) -> Self {
        Self {
            inter_sender,
            intra_receiver,
            timeout,
        }
    }

    pub fn connect(&mut self) -> Result<(), Error> {
        self.intra_receiver.connect_senders(self.timeout)?;
        self.inter_sender.connect_receiver(self.timeout)?;
        trace!("SecondarySendRelay connected");
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            // Receive signal from the intra-process receiver
            let signal = self.intra_receiver.receive(self.timeout);

            let signal = match signal {
                Ok(Some(signal)) => signal,
                Ok(None) => {
                    error!("Reception timed out");
                    continue;
                }
                Err(Error::ChannelClosed) => {
                    debug!(
                        "[SecondarySendRelay] detected closed channel from local workers, exiting."
                    );
                    return Ok(());
                }
                Err(_) => {
                    error!("[SecondarySendRelay]Failed to receive");
                    continue;
                }
            };

            // Forward signal to the primary agent
            let core_signal: Signal = match signal.try_into() {
                Ok(signal) => signal,
                Err(_) => {
                    error!("[SecondarySendRelay]Received unexpected signal {signal:?}");
                    continue;
                }
            };

            let protocol_signal: Inter::ProtocolSignal = core_signal.into();
            self.inter_sender.send(protocol_signal)?;
        }
    }
}
