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

use core::time::Duration;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::config::{BIND_ADDR, BIND_ADDR2, COM_BACKEND, SOCKET_PATH, SOCKET_PATH2};
use feo::agent::com_init::initialize_com_secondary;
use feo::agent::NodeAddress;
use feo::error::Error;
use feo::ids::{ActivityId, AgentId};
use feo_log::info;

use crate::scenario::ScenarioConfig as _;
use crate::{config::PRIMARY_AGENT_ID, Scenario, Signalling};

pub trait SecondaryLauncher {
    /// Launch secondary FEO agent
    fn launch_secondary(&self, scenario: Scenario, num: usize) -> Result<(), Error>;
}

impl SecondaryLauncher for Signalling {
    fn launch_secondary(&self, scenario: Scenario, num: usize) -> Result<(), Error> {
        let agent_id: AgentId = (PRIMARY_AGENT_ID.id() + num as u64).into();
        info!("Starting secondary agent {}", agent_id);

        let server_name = "UNUSED".to_string(); // STUB

        // determine set of activity ids belonging to this agent
        let local_activities: HashSet<ActivityId> = scenario
            .agent_assignments_ids()
            .remove(&agent_id)
            .unwrap()
            .iter()
            .flat_map(|(_, acts)| acts.iter())
            .copied()
            .collect();

        // Initialize topics. Do not drop.
        let _topic_guards = initialize_com_secondary(COM_BACKEND, scenario.topic_dependencies(), &local_activities);

        match self {
            Signalling::DirectMpsc => {
                panic!("Secondaries are not supported with this feature flag")
            },
            Signalling::DirectTcp => {
                use feo::agent::direct::secondary::{Secondary, SecondaryConfig};

                let config = SecondaryConfig {
                    id: agent_id,
                    worker_assignments: scenario.agent_assignments(server_name).remove(&agent_id).unwrap(),
                    timeout: Duration::from_secs(1),
                    endpoint: NodeAddress::Tcp(BIND_ADDR),
                };

                Secondary::new(config).run();
            },
            Signalling::DirectUnix => {
                use feo::agent::direct::secondary::{Secondary, SecondaryConfig};

                let config = SecondaryConfig {
                    id: agent_id,
                    worker_assignments: scenario.agent_assignments(server_name).remove(&agent_id).unwrap(),
                    timeout: Duration::from_secs(1),
                    endpoint: NodeAddress::UnixSocket(PathBuf::from(SOCKET_PATH)),
                };

                Secondary::new(config).run();
            },
            Signalling::RelayedTcp => {
                use feo::agent::relayed::secondary::{Secondary, SecondaryConfig};

                let config = SecondaryConfig {
                    id: agent_id,
                    worker_assignments: scenario.agent_assignments(server_name).remove(&agent_id).unwrap(),
                    timeout: Duration::from_secs(10),
                    bind_address_senders: NodeAddress::Tcp(BIND_ADDR),
                    bind_address_receivers: NodeAddress::Tcp(BIND_ADDR2),
                };

                Secondary::new(config).run();
            },
            Signalling::RelayedUnix => {
                use feo::agent::relayed::secondary::{Secondary, SecondaryConfig};

                let config = SecondaryConfig {
                    id: agent_id,
                    worker_assignments: scenario.agent_assignments(server_name).remove(&agent_id).unwrap(),
                    timeout: Duration::from_secs(10),
                    bind_address_senders: NodeAddress::UnixSocket(PathBuf::from(SOCKET_PATH)),
                    bind_address_receivers: NodeAddress::UnixSocket(PathBuf::from(SOCKET_PATH2)),
                };

                Secondary::new(config).run();
            },
        }

        Ok(())
    }
}
