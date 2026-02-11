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

use clap::Parser;
use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, IpcSender};
use log::{info, trace};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::mem;
use std::path::PathBuf;
use std::process::Child;
use std::sync::MutexGuard;
use std::thread::sleep;
use std::time::Duration;
use std::{process::Command, sync::Mutex};

use crate::monitor::{MonitorNotification, MonitorRequest};

mod monitor;

fn aquire_lock() -> MutexGuard<'static, ()> {
    static MUTEX: Mutex<()> = Mutex::new(());
    let _ = env_logger::try_init();
    MUTEX.lock().unwrap()
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    test_agent: PathBuf,
}

enum MonitorConnection {
    Unconnected(IpcOneShotServer<(IpcSender<MonitorRequest>, IpcReceiver<MonitorNotification>)>),
    Connected {
        _sender: IpcSender<MonitorRequest>,
        receiver: IpcReceiver<MonitorNotification>,
    },
}

enum ChildHandle {
    Primary { child: Child, monitor: MonitorConnection },
    Secondary { child: Child },
}

/// FEO agent starting and controlling handle
struct FeoRunner {
    cli: Cli,
    /// Number of agents to spawn
    agents: u32,
    /// FEO signalling implementation to use
    signalling: String,
    /// Test scenario to run
    scenario: String,
    /// Spawned agents' handles
    children: Vec<ChildHandle>,
}

impl FeoRunner {
    /// Configure for a single agent setup
    pub fn single_agent(cli: Cli, signalling: impl Into<String>, scenario: impl Into<String>) -> Self {
        Self {
            cli,
            agents: 1,
            signalling: signalling.into(),
            scenario: scenario.into(),
            children: vec![],
        }
    }

    /// Configure for multiple agent setup
    pub fn multiple_agents(cli: Cli, agents: u32, signalling: impl Into<String>, scenario: impl Into<String>) -> Self {
        Self {
            cli,
            agents,
            signalling: signalling.into(),
            scenario: scenario.into(),
            children: vec![],
        }
    }

    /// Launch FEO agents
    fn launch(&mut self) {
        for i in 0..self.agents {
            if i == 1 {
                // Primary may need a sec to initialize COM entities
                sleep(Duration::from_secs(1));
            }
            // let com_config = self.cli.com_config.to_string_lossy();
            // let mut args = vec!["--com-config", &com_config];
            let i_str = i.to_string();
            let mut args = vec![];
            args.push(self.signalling.as_str());
            args.push(self.scenario.as_str());
            if i == 0 {
                let (server, monitor_server) = ipc_channel::ipc::IpcOneShotServer::new().unwrap();
                trace!("Starting primary agent");
                args.push("primary");
                args.push(&monitor_server);
                let child = Command::new(&self.cli.test_agent)
                    .args(args)
                    .spawn()
                    .expect("failed to run test-agent");
                self.children.push(ChildHandle::Primary {
                    child,
                    monitor: MonitorConnection::Unconnected(server),
                });
            } else {
                trace!("Starting secondary agent");
                args.push("secondary");
                args.push(i_str.as_str());
                let child = Command::new(&self.cli.test_agent)
                    .args(args)
                    .spawn()
                    .expect("failed to run test-agent");
                self.children.push(ChildHandle::Secondary { child })
            }
        }
        trace!("Connecting monitor");
        let primary = &mut self.children[0];
        if let ChildHandle::Primary { monitor, .. } = primary {
            let (_, (request_tx, notification_rx)): (_, (IpcSender<MonitorRequest>, IpcReceiver<MonitorNotification>)) =
                if let MonitorConnection::Unconnected(server) = mem::replace(
                    monitor,
                    MonitorConnection::Unconnected(IpcOneShotServer::new().unwrap().0),
                ) {
                    server.accept().unwrap()
                } else {
                    panic!("shouldn't happen")
                };
            *monitor = MonitorConnection::Connected {
                _sender: request_tx,
                receiver: notification_rx,
            };
        } else {
            panic!("shouldn't happen")
        }
    }

    /// Wait for Startup notification from Monitor activity
    pub fn expect_startup(&mut self) {
        for child in self.children.iter() {
            if let ChildHandle::Primary {
                monitor: MonitorConnection::Connected { receiver, .. },
                ..
            } = child
            {
                match receiver.recv().unwrap() {
                    MonitorNotification::Startup => {},
                    v => panic!("unexpected {v:?}"),
                }
            }
        }
    }

    /// Wait for Step notification from Monitor activity
    pub fn expect_step(&mut self) {
        for child in self.children.iter() {
            if let ChildHandle::Primary {
                monitor: MonitorConnection::Connected { receiver, .. },
                ..
            } = child
            {
                match receiver.recv().unwrap() {
                    MonitorNotification::Step => {},
                    v => panic!("unexpected {v:?}"),
                }
            }
        }
    }

    /// Wait for Shutdown notification from Monitor activity
    pub fn expect_shutdown(&mut self) {
        for child in self.children.iter() {
            if let ChildHandle::Primary {
                monitor: MonitorConnection::Connected { receiver, .. },
                ..
            } = child
            {
                match receiver.recv().unwrap() {
                    MonitorNotification::Shutdown => {},
                    v => panic!("unexpected {v:?}"),
                }
            }
        }
    }

    /// Send SIGTERM to the primary FEO agent
    pub fn trigger_shutdown(&mut self) {
        for child in self.children.iter() {
            if let ChildHandle::Primary { child, .. } = child {
                signal::kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM).unwrap();
            }
        }
    }

    /// Wait for all agents to exit
    pub fn wait_for_completion(self) {
        for child in self.children {
            match child {
                ChildHandle::Primary { mut child, .. } | ChildHandle::Secondary { mut child } => {
                    let status = child.wait().unwrap();
                    assert!(status.success(), "FEO agent failed");
                },
            }
        }
    }
}

/// Single agent scenario
fn single_agent(cli: Cli, signalling: &str) {
    info!("Starting single-agent test run...");
    let mut runner = FeoRunner::single_agent(cli, signalling, "single-agent");
    runner.launch();
    runner.expect_startup();
    for _ in 0..5 {
        runner.expect_step();
    }
    runner.trigger_shutdown();
    runner.expect_shutdown();
    runner.wait_for_completion();
    info!("Single-agent test run finished");
}

/// Multiple agent scenario
fn multiple_agents(cli: Cli, signalling: &str) {
    info!("Starting multiple-agents test run...");
    let mut runner = FeoRunner::multiple_agents(cli, 2, signalling, "multiple-agents");
    runner.launch();
    runner.expect_startup();
    for _ in 0..5 {
        runner.expect_step();
    }
    runner.trigger_shutdown();
    runner.expect_shutdown();
    runner.wait_for_completion();
    info!("Multiple-agents test run finished");
}

/// Bazel-specific path to test_agent binary
pub const TEST_AGENT_PATH: &str = "./tests/rust/feo_tests/test_agent/test_agent";

impl Default for Cli {
    fn default() -> Self {
        Self {
            test_agent: PathBuf::from(TEST_AGENT_PATH),
        }
    }
}

#[test]
fn single_agent_direct_tcp() {
    let _guard = aquire_lock();
    info!("Starting single_agent_direct_tcp");
    single_agent(Cli::default(), "direct-tcp")
}

#[test]
fn single_agent_direct_mpsc() {
    let _guard = aquire_lock();
    info!("Starting single_agent_direct_mpsc");
    single_agent(Cli::default(), "direct-mpsc")
}

#[test]
fn single_agent_direct_unix() {
    let _guard = aquire_lock();
    info!("Starting single_agent_direct_unix");
    single_agent(Cli::default(), "direct-unix")
}

// Disabled due to timeout issue
// #[test]
// fn single_agent_relayed_tcp() {
//     let _guard = aquire_lock();
//     info!("Starting single_agent_relayed_tcp");
//     single_agent(Cli::default(), "relayed-tcp")
// }

// Disabled due to timeout issue
// #[test]
// fn single_agent_relayed_unix() {
//     let _guard = aquire_lock();
//     info!("Starting single_agent_relayed_unix");
//     single_agent(Cli::default(), "relayed-unix")
// }

#[test]
fn multiple_agents_direct_tcp() {
    let _guard = aquire_lock();
    info!("Starting multiple_agents_direct_tcp");
    multiple_agents(Cli::default(), "direct-tcp")
}

#[test]
fn multiple_agents_direct_unix() {
    let _guard = aquire_lock();
    info!("Starting multiple_agents_direct_unix");
    multiple_agents(Cli::default(), "direct-unix")
}

#[test]
fn multiple_agents_relayed_unix() {
    let _guard = aquire_lock();
    info!("Starting multiple_agents_relayed_unix");
    multiple_agents(Cli::default(), "relayed-unix")
}

#[test]
fn multiple_agents_relayed_tcp() {
    let _guard = aquire_lock();
    info!("Starting multiple_agents_relayed_tcp");
    multiple_agents(Cli::default(), "relayed-tcp")
}
