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

use crate::activities::{Monitor, Receiver, Sender};
use crate::config::TOPIC_COUNTER;
use crate::Scenario;
use feo::activity::{ActivityBuilder, ActivityIdAndBuilder};
use feo::ids::{ActivityId, AgentId, WorkerId};
use feo::topicspec::{Direction, TopicSpecification};
use std::collections::HashMap;

// For each activity, list the activities it needs to wait for
pub type ActivityDependencies = HashMap<ActivityId, Vec<ActivityId>>;
pub type WorkerAssignment = (WorkerId, Vec<(ActivityId, Box<dyn ActivityBuilder>)>);

/// Scenario-specific FEO configuration (agents, workers, activities, com, et.c)
pub trait ScenarioConfig {
    fn activity_dependencies(&self) -> ActivityDependencies;

    fn agent_assignments(&self, monitor_server: String)
        -> HashMap<AgentId, Vec<(WorkerId, Vec<ActivityIdAndBuilder>)>>;

    fn topic_dependencies(&self) -> Vec<TopicSpecification<'static>>;

    fn worker_agent_map(&self) -> HashMap<WorkerId, AgentId> {
        self.agent_assignments_ids()
            .iter()
            .flat_map(|(aid, w)| w.iter().map(move |(wid, _)| (*wid, *aid)))
            .collect()
    }

    fn activity_worker_map(&self) -> HashMap<ActivityId, WorkerId> {
        self.agent_assignments_ids()
            .values()
            .flat_map(|vec| {
                vec.iter()
                    .flat_map(move |(wid, aid_b)| aid_b.iter().map(|v| (v.id().into(), *wid)))
            })
            .collect()
    }

    fn activity_agent_map(&self) -> HashMap<ActivityId, AgentId> {
        let worker_agent_map = self.worker_agent_map();
        self.activity_worker_map()
            .iter()
            .map(|(activity_id, worker_id)| {
                let agent_id = worker_agent_map.get(worker_id).copied().unwrap();
                (*activity_id, agent_id)
            })
            .collect()
    }

    fn agent_assignments_ids(&self) -> HashMap<AgentId, Vec<(WorkerId, Vec<ActivityId>)>> {
        self.agent_assignments("STUB".to_string())
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    v.into_iter()
                        .map(|(w, a)| (w, a.into_iter().map(|(a, _)| a).collect()))
                        .collect(),
                )
            })
            .collect()
    }
}

impl ScenarioConfig for Scenario {
    fn activity_dependencies(&self) -> ActivityDependencies {
        match self {
            Scenario::SingleAgent | Scenario::MultipleAgents => [
                (0.into(), vec![]),
                (1.into(), vec![0.into()]),
                (2.into(), vec![1.into()]),
            ]
            .into(),
        }
    }

    fn agent_assignments(
        &self,
        monitor_server: String,
    ) -> HashMap<AgentId, Vec<(WorkerId, Vec<ActivityIdAndBuilder>)>> {
        match self {
            Scenario::SingleAgent => {
                let w40: WorkerAssignment = (40.into(), vec![(0.into(), Box::new(|id| Sender::build(id)))]);
                let w41: WorkerAssignment = (
                    41.into(),
                    vec![
                        (1.into(), Box::new(|id| Receiver::build(id))),
                        (2.into(), Box::new(|id| Monitor::build(id, monitor_server))),
                    ],
                );
                [(100.into(), vec![w40, w41])].into_iter().collect()
            },
            Scenario::MultipleAgents => {
                let w40: WorkerAssignment = (40.into(), vec![(0.into(), Box::new(|id| Sender::build(id)))]);
                let w41: WorkerAssignment = (
                    41.into(),
                    vec![
                        (1.into(), Box::new(|id| Receiver::build(id))),
                        (2.into(), Box::new(|id| Monitor::build(id, monitor_server))),
                    ],
                );
                [(100.into(), vec![w41]), (101.into(), vec![w40])].into_iter().collect()
            },
        }
    }

    fn topic_dependencies(&self) -> Vec<TopicSpecification<'static>> {
        use Direction::*;

        vec![TopicSpecification::new::<Counter>(
            TOPIC_COUNTER,
            vec![(0.into(), Outgoing), (1.into(), Incoming)],
        )]
    }
}

/// COM type for Sender and Receiver activities' data exchange
#[derive(Debug, Default)]
pub struct Counter {
    pub counter: usize,
}
