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

use crate::activities::components::{
    BrakeController, Camera, EmergencyBraking, EnvironmentRenderer, LaneAssist, NeuralNet, Radar, SteeringController,
    TrajectoryVisualizer,
};
#[cfg(feature = "com_mw")]
use com_api::{Builder, LolaRuntimeBuilderImpl, LolaRuntimeImpl, RuntimeBuilder};
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use feo::activity::{ActivityBuilder, ActivityIdAndBuilder};
use feo::ids::{ActivityId, AgentId, WorkerId};
use feo::topicspec::{Direction, TopicSpecification};
#[cfg(not(feature = "com_mw"))]
use feo_com::interface::ComBackend;
use mini_adas_gen::{BrakeInstruction, CameraImage, RadarScan, Scene, Steering};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
#[cfg(feature = "com_mw")]
use std::sync::OnceLock;

pub type WorkerAssignment = (WorkerId, Vec<(ActivityId, Box<dyn ActivityBuilder>)>);

// For each activity, list the activities it needs to wait for
pub type ActivityDependencies = HashMap<ActivityId, Vec<ActivityId>>;

#[cfg(feature = "com_iox2")]
pub const COM_BACKEND: ComBackend = ComBackend::Iox2;
#[cfg(feature = "com_linux_shm")]
pub const COM_BACKEND: ComBackend = ComBackend::LinuxShm;

pub const BIND_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8081);
pub const BIND_ADDR2: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8082);

pub const TOPIC_INFERRED_SCENE: &str = "/feo/com/MiniAdasNeuralNet";
pub const TOPIC_CONTROL_BRAKES: &str = "/feo/com/MiniAdasBrakeController";
pub const TOPIC_CONTROL_STEERING: &str = "/feo/com/MiniAdasSteeringController";
pub const TOPIC_CAMERA_FRONT: &str = "/feo/com/MiniAdasCamera";
pub const TOPIC_RADAR_FRONT: &str = "/feo/com/MiniAdasRadar";

/// Allow up to two recorder processes (that potentially need to subscribe to every topic)
pub const MAX_ADDITIONAL_SUBSCRIBERS: usize = 2;

#[cfg(feature = "com_mw")]
static MW_COM_RUNTIME: OnceLock<LolaRuntimeImpl> = OnceLock::new();

#[cfg(feature = "com_mw")]
pub fn init_mw_com_runtime(agent_id: AgentId) -> &'static LolaRuntimeImpl {
    MW_COM_RUNTIME.get_or_init(|| {
        let mut lola_runtime_builder = LolaRuntimeBuilderImpl::new();
        lola_runtime_builder.load_config(&PathBuf::from(format!(
            "./examples/rust/mini-adas/etc/mw_com_config_{}.json",
            agent_id.id()
        )));
        lola_runtime_builder.build().unwrap()
    })
}

#[cfg(feature = "com_mw")]
pub fn mw_com_runtime() -> &'static LolaRuntimeImpl {
    MW_COM_RUNTIME.get().unwrap()
}

pub fn socket_paths() -> (PathBuf, PathBuf) {
    (
        Path::new("/tmp/feo_listener1.socket").to_owned(),
        Path::new("/tmp/feo_listener2.socket").to_owned(),
    )
}

pub fn agent_assignments() -> HashMap<AgentId, Vec<(WorkerId, Vec<ActivityIdAndBuilder>)>> {
    // Assign activities to different workers
    let w40: WorkerAssignment = (
        40.into(),
        vec![(0.into(), Box::new(|id| Camera::build(id, TOPIC_CAMERA_FRONT)))],
    );
    let w41: WorkerAssignment = (
        41.into(),
        vec![(1.into(), Box::new(|id| Radar::build(id, TOPIC_RADAR_FRONT)))],
    );

    let w42: WorkerAssignment = (
        42.into(),
        vec![
            (
                2.into(),
                Box::new(|id| NeuralNet::build(id, TOPIC_CAMERA_FRONT, TOPIC_RADAR_FRONT, TOPIC_INFERRED_SCENE)),
            ),
            (
                3.into(),
                Box::new(|id| EnvironmentRenderer::build(id, TOPIC_INFERRED_SCENE)),
            ),
        ],
    );

    let w43: WorkerAssignment = (
        43.into(),
        vec![
            (
                4.into(),
                Box::new(|id| EmergencyBraking::build(id, TOPIC_INFERRED_SCENE, TOPIC_CONTROL_BRAKES)),
            ),
            (
                6.into(),
                Box::new(|id| BrakeController::build(id, TOPIC_CONTROL_BRAKES)),
            ),
        ],
    );
    let w44: WorkerAssignment = (
        44.into(),
        vec![
            (5.into(), Box::new(|id| LaneAssist::build(id, TOPIC_CONTROL_STEERING))),
            (
                7.into(),
                Box::new(|id| SteeringController::build(id, TOPIC_CONTROL_STEERING)),
            ),
            (8.into(), Box::new(|id| TrajectoryVisualizer::build(id))),
        ],
    );

    // Assign workers to pools with exactly one pool belonging to one agent
    #[cfg(any(
        feature = "signalling_direct_tcp",
        feature = "signalling_direct_unix",
        feature = "signalling_relayed_tcp",
        feature = "signalling_relayed_unix"
    ))]
    let assignment = [
        (100.into(), vec![w40, w41]),
        (101.into(), vec![w42]),
        (102.into(), vec![w43, w44]),
    ]
    .into_iter()
    .collect();
    #[cfg(feature = "signalling_direct_mpsc")]
    let assignment = [(100.into(), vec![w40, w41, w42, w43, w44])].into_iter().collect();

    assignment
}

pub fn activity_dependencies() -> ActivityDependencies {
    //      Primary              |       Secondary1         |                  Secondary2
    // ---------------------------------------------------------------------------------------------------
    //
    //   Camera(40)   Radar(41)
    //        \           \
    //                                 NeuralNet(42)
    //                                      |                           \                     \
    //                             EnvironmentRenderer(42)       EmergencyBraking(43)    LaneAssist(44)
    //                                                                   |                     |
    //                                                            BrakeController(43)   SteeringController(44)

    let dependencies = [
        // Camera
        (0.into(), vec![]),
        // Radar
        (1.into(), vec![]),
        // NeuralNet
        (2.into(), vec![0.into(), 1.into()]),
        // EnvironmentRenderer
        (3.into(), vec![2.into()]),
        // EmergencyBraking
        (4.into(), vec![2.into()]),
        // LaneAssist
        (5.into(), vec![2.into()]),
        // BrakeController
        (6.into(), vec![4.into()]),
        // SteeringController
        (7.into(), vec![5.into()]),
        // TrajectoryVisualizer
        (8.into(), vec![5.into()]),
    ];

    dependencies.into()
}

pub fn topic_dependencies<'a>() -> Vec<TopicSpecification<'a>> {
    use Direction::*;

    vec![
        TopicSpecification::new::<CameraImage>(TOPIC_CAMERA_FRONT, vec![(0.into(), Outgoing), (2.into(), Incoming)]),
        TopicSpecification::new::<RadarScan>(TOPIC_RADAR_FRONT, vec![(1.into(), Outgoing), (2.into(), Incoming)]),
        TopicSpecification::new::<Scene>(
            TOPIC_INFERRED_SCENE,
            vec![
                (2.into(), Outgoing),
                (3.into(), Incoming),
                (4.into(), Incoming),
                (5.into(), Incoming),
            ],
        ),
        TopicSpecification::new::<BrakeInstruction>(
            TOPIC_CONTROL_BRAKES,
            vec![(4.into(), Outgoing), (6.into(), Incoming)],
        ),
        TopicSpecification::new::<Steering>(TOPIC_CONTROL_STEERING, vec![(5.into(), Outgoing), (7.into(), Incoming)]),
    ]
}

pub fn worker_agent_map() -> HashMap<WorkerId, AgentId> {
    agent_assignments()
        .iter()
        .flat_map(|(aid, w)| w.iter().map(move |(wid, _)| (*wid, *aid)))
        .collect()
}

pub fn agent_assignments_ids() -> HashMap<AgentId, Vec<(WorkerId, Vec<ActivityId>)>> {
    agent_assignments()
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
