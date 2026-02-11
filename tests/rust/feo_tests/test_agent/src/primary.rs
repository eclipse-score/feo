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

use crate::config::COM_BACKEND;
use crate::config::MAX_ADDITIONAL_SUBSCRIBERS;
use crate::config::PRIMARY_AGENT_ID;
use crate::config::{BIND_ADDR, BIND_ADDR2, DEFAULT_FEO_CYCLE_TIME, SOCKET_PATH, SOCKET_PATH2};
use crate::scenario::ScenarioConfig;
use crate::{Scenario, Signalling};
use feo::agent::com_init::initialize_com_primary;
use feo::error::Error;
use feo_log::info;
use feo_time::Duration;
use std::path::PathBuf;

pub trait PrimaryLauncher {
    /// Launch primary FEO agent
    fn launch_primary(&self, scenario: Scenario, server_name: String) -> Result<(), Error>;
}

impl PrimaryLauncher for Signalling {
    fn launch_primary(&self, scenario: Scenario, server_name: String) -> Result<(), Error> {
        info!("Starting primary agent {PRIMARY_AGENT_ID}");

        // Initialize topics. Do not drop.
        let _topic_guards = initialize_com_primary(
            COM_BACKEND,
            PRIMARY_AGENT_ID,
            scenario.topic_dependencies(),
            &scenario.agent_assignments_ids(),
            MAX_ADDITIONAL_SUBSCRIBERS,
        );

        match self {
            Signalling::DirectMpsc => {
                pub(super) use feo::agent::direct::primary_mpsc::{Primary, PrimaryConfig};

                let config = PrimaryConfig {
                    cycle_time: DEFAULT_FEO_CYCLE_TIME,
                    activity_dependencies: scenario.activity_dependencies(),
                    recorder_ids: vec![],
                    id: PRIMARY_AGENT_ID,
                    worker_assignments: scenario
                        .agent_assignments(server_name)
                        .remove(&PRIMARY_AGENT_ID)
                        .unwrap(),
                    timeout: Duration::from_secs(10),
                    startup_timeout: Duration::from_secs(10),
                };

                Primary::new(config).unwrap().run().unwrap();
            },
            Signalling::DirectTcp => {
                use feo::agent::direct::primary::{Primary, PrimaryConfig};
                use feo::agent::NodeAddress;

                let config = PrimaryConfig {
                    cycle_time: DEFAULT_FEO_CYCLE_TIME,
                    activity_dependencies: scenario.activity_dependencies(),
                    recorder_ids: vec![],
                    id: PRIMARY_AGENT_ID,
                    worker_assignments: scenario
                        .agent_assignments(server_name)
                        .remove(&PRIMARY_AGENT_ID)
                        .unwrap(),
                    timeout: Duration::from_secs(10),
                    connection_timeout: Duration::from_secs(10),
                    startup_timeout: Duration::from_secs(10),
                    endpoint: NodeAddress::Tcp(BIND_ADDR),
                    activity_agent_map: scenario.activity_agent_map(),
                };

                Primary::new(config).unwrap().run().unwrap();
            },
            Signalling::DirectUnix => {
                use feo::agent::direct::primary::{Primary, PrimaryConfig};
                use feo::agent::NodeAddress;

                let config = PrimaryConfig {
                    cycle_time: DEFAULT_FEO_CYCLE_TIME,
                    activity_dependencies: scenario.activity_dependencies(),
                    recorder_ids: vec![],
                    id: PRIMARY_AGENT_ID,
                    worker_assignments: scenario
                        .agent_assignments(server_name)
                        .remove(&PRIMARY_AGENT_ID)
                        .unwrap(),
                    timeout: Duration::from_secs(10),
                    connection_timeout: Duration::from_secs(10),
                    startup_timeout: Duration::from_secs(10),
                    endpoint: NodeAddress::UnixSocket(PathBuf::from(SOCKET_PATH)),
                    activity_agent_map: scenario.activity_agent_map(),
                };

                Primary::new(config).unwrap().run().unwrap();
            },
            Signalling::RelayedTcp => {
                use feo::agent::NodeAddress;

                use feo::agent::relayed::primary::{Primary, PrimaryConfig};

                let config = PrimaryConfig {
                    cycle_time: DEFAULT_FEO_CYCLE_TIME,
                    activity_dependencies: scenario.activity_dependencies(),
                    recorder_ids: vec![],
                    worker_assignments: scenario
                        .agent_assignments(server_name)
                        .remove(&PRIMARY_AGENT_ID)
                        .unwrap(),
                    timeout: Duration::from_secs(10),
                    connection_timeout: Duration::from_secs(10),
                    startup_timeout: Duration::from_secs(10),
                    bind_address_senders: NodeAddress::Tcp(BIND_ADDR),
                    bind_address_receivers: NodeAddress::Tcp(BIND_ADDR2),
                    id: PRIMARY_AGENT_ID,
                    worker_agent_map: scenario.worker_agent_map(),
                    activity_worker_map: scenario.activity_worker_map(),
                };

                Primary::new(config).unwrap().run().unwrap();
            },
            Signalling::RelayedUnix => {
                use feo::agent::NodeAddress;

                use feo::agent::relayed::primary::{Primary, PrimaryConfig};

                let config = PrimaryConfig {
                    cycle_time: DEFAULT_FEO_CYCLE_TIME,
                    activity_dependencies: scenario.activity_dependencies(),
                    recorder_ids: vec![],
                    worker_assignments: scenario
                        .agent_assignments(server_name)
                        .remove(&PRIMARY_AGENT_ID)
                        .unwrap(),
                    timeout: Duration::from_secs(10),
                    connection_timeout: Duration::from_secs(10),
                    startup_timeout: Duration::from_secs(10),
                    bind_address_senders: NodeAddress::UnixSocket(PathBuf::from(SOCKET_PATH)),
                    bind_address_receivers: NodeAddress::UnixSocket(PathBuf::from(SOCKET_PATH2)),
                    id: PRIMARY_AGENT_ID,
                    worker_agent_map: scenario.worker_agent_map(),
                    activity_worker_map: scenario.activity_worker_map(),
                };

                Primary::new(config).unwrap().run().unwrap();
            },
        }
        Ok(())
    }
}
