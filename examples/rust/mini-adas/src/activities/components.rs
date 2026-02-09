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

use crate::config::mw_com_runtime;
use com_api::{
    Builder, CommData, FindServiceSpecifier, InstanceSpecifier, Interface, LolaRuntimeImpl, Producer, Publisher,
    Runtime, SampleContainer, SampleMaybeUninit, SampleMut, ServiceDiscovery, Subscriber, Subscription,
};
use core::hash::{BuildHasher as _, Hasher as _};
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut, Range};
use core::time::Duration;
use feo::activity::Activity;
use feo::error::ActivityError;
use feo::ids::ActivityId;
use feo_tracing::instrument;
use mini_adas_gen::{
    BrakeControllerConsumer, BrakeControllerInterface, CameraConsumer, CameraInterface, NeuralNetConsumer,
    NeuralNetInterface, RadarConsumer, RadarInterface, SteeringControllerConsumer, SteeringControllerInterface,
};
use mini_adas_gen::{BrakeInstruction, CameraImage, RadarScan, Scene, Steering};
use score_log::debug;
use std::hash::RandomState;
use std::thread;
use std::thread::sleep;

const SLEEP_RANGE: Range<i64> = 10..45;

pub struct DebugWrapper<T>(pub T);

impl<T> core::fmt::Debug for DebugWrapper<T> {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Ok(())
    }
}

impl<T> core::ops::Deref for DebugWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for DebugWrapper<T> {
    fn deref_mut(&mut self) -> &mut <Self as core::ops::Deref>::Target {
        &mut self.0
    }
}

type MwComPublisher<T> = <LolaRuntimeImpl as Runtime>::Publisher<T>;
type MwComSubscription<T> =
    <<LolaRuntimeImpl as Runtime>::Subscriber<T> as Subscriber<T, LolaRuntimeImpl>>::Subscription;
type MwComSample<'a, T> = <MwComSubscription<T> as Subscription<T, LolaRuntimeImpl>>::Sample<'a>;

/// Camera activity
///
/// This activity emulates a camera generating a [CameraImage].
#[derive(Debug)]
pub struct Camera {
    /// ID of the activity
    activity_id: ActivityId,
    /// Image output
    output_image: DebugWrapper<MwComPublisher<CameraImage>>,

    // Local state for pseudo-random output generation
    num_people: usize,
    num_cars: usize,
    distance_obstacle: f64,
}

impl Camera {
    pub fn build(activity_id: ActivityId, image_topic: &str) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            output_image: DebugWrapper(create_producer::<CameraInterface>(image_topic).image),
            num_people: 4,
            num_cars: 10,
            distance_obstacle: 40.0,
        })
    }

    fn get_image(&mut self) -> CameraImage {
        const PEOPLE_CHANGE_PROP: f64 = 0.8;
        const CAR_CHANGE_PROP: f64 = 0.8;
        const DISTANCE_CHANGE_PROP: f64 = 1.0;

        self.num_people = random_walk_integer(self.num_people, PEOPLE_CHANGE_PROP, 1);
        self.num_cars = random_walk_integer(self.num_people, CAR_CHANGE_PROP, 2);
        let sample = random_walk_float(self.distance_obstacle, DISTANCE_CHANGE_PROP, 5.0);
        self.distance_obstacle = sample.clamp(20.0, 50.0);

        CameraImage {
            num_people: self.num_people,
            num_cars: self.num_cars,
            distance_obstacle: self.distance_obstacle,
        }
    }
}

impl Activity for Camera {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    #[instrument(name = "Camera startup")]
    fn startup(&mut self) -> Result<(), ActivityError> {
        Ok(())
    }

    #[instrument(name = "Camera")]
    fn step(&mut self) -> Result<(), ActivityError> {
        debug!("Stepping Camera");
        sleep_random();

        let image = self.get_image();
        debug!("Sending image: {:?}", image);
        self.output_image.send(image).unwrap();
        Ok(())
    }

    #[instrument(name = "Camera shutdown")]
    fn shutdown(&mut self) -> Result<(), ActivityError> {
        debug!("Shutting down Camera activity {}", self.activity_id);
        Ok(())
    }
}

/// Radar activity
///
/// This component emulates are radar generating a [RadarScan].
#[derive(Debug)]
pub struct Radar {
    /// ID of the activity
    activity_id: ActivityId,
    /// Radar scan output
    output_scan: DebugWrapper<<LolaRuntimeImpl as Runtime>::Publisher<RadarScan>>,

    // Local state for pseudo-random output generation
    distance_obstacle: f64,
}

impl Radar {
    pub fn build(activity_id: ActivityId, radar_topic: &str) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            output_scan: DebugWrapper(create_producer::<RadarInterface>(radar_topic).scan),
            distance_obstacle: 40.0,
        })
    }

    fn get_scan(&mut self) -> RadarScan {
        const DISTANCE_CHANGE_PROP: f64 = 1.0;

        let sample = random_walk_float(self.distance_obstacle, DISTANCE_CHANGE_PROP, 6.0);
        self.distance_obstacle = sample.clamp(16.0, 60.0);

        let error_margin = gen_random_in_range(-10..10) as f64 / 10.0;

        RadarScan {
            distance_obstacle: self.distance_obstacle,
            error_margin,
        }
    }
}

impl Activity for Radar {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    #[instrument(name = "Radar startup")]
    fn startup(&mut self) -> Result<(), ActivityError> {
        Ok(())
    }

    #[instrument(name = "Radar")]
    fn step(&mut self) -> Result<(), ActivityError> {
        debug!("Stepping Radar");
        sleep_random();

        let scan = self.get_scan();
        debug!("Sending scan: {:?}", scan);
        self.output_scan.send(scan).unwrap();
        Ok(())
    }

    #[instrument(name = "Radar shutdown")]
    fn shutdown(&mut self) -> Result<(), ActivityError> {
        debug!("Shutting down Radar activity {}", self.activity_id);
        Ok(())
    }
}

struct MwComInput<T: CommData> {
    count: usize,
    /// Subscription
    subscription: MwComSubscription<T>,
    /// Sample container
    sample_container: SampleContainer<MwComSample<'static, T>>,
}

impl<T: CommData> core::fmt::Debug for MwComInput<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MwComInput")
    }
}

impl<T: CommData> MwComInput<T> {
    fn new<C, I: Interface<Consumer<LolaRuntimeImpl> = C>>(
        topic: &str,
        f: impl FnOnce(C) -> <LolaRuntimeImpl as Runtime>::Subscriber<T>,
    ) -> Self {
        Self {
            count: 0,
            subscription: f(create_consumer::<I>(topic)).subscribe(1).unwrap(),
            sample_container: SampleContainer::new(1),
        }
    }

    fn read(&mut self) -> MwComSample<'_, T> {
        debug!("Starting to read {}", self.count);
        self.count += 1;
        assert_eq!(
            1,
            self.subscription
                .try_receive(&mut self.sample_container, 1,)
                .expect("receive failed"),
            "mw com read failed"
        );
        let result = self.sample_container.pop_front().expect("pop_front failed");
        debug!("Reading done");
        result
    }
}

/// Neural network activity
///
/// This component emulates a neural network
/// pseudo-inferring a [Scene] output
/// from the provided [Camera] and [Radar] inputs.
#[derive(Debug)]
pub struct NeuralNet {
    /// ID of the activity
    activity_id: ActivityId,
    /// Image input
    input_image: MwComInput<CameraImage>,
    /// Radar scan input
    input_scan: MwComInput<RadarScan>,
    /// Scene output
    output_scene: DebugWrapper<MwComPublisher<Scene>>,
}

impl NeuralNet {
    pub fn build(activity_id: ActivityId, image_topic: &str, scan_topic: &str, scene_topic: &str) -> Box<dyn Activity> {
        let output_scene = DebugWrapper(create_producer::<NeuralNetInterface>(scene_topic).scene);
        Box::new(Self {
            activity_id,
            input_image: MwComInput::new::<_, CameraInterface>(image_topic, |i: CameraConsumer<_>| i.image),
            input_scan: MwComInput::new::<_, RadarInterface>(scan_topic, |r: RadarConsumer<_>| r.scan),
            output_scene,
        })
    }

    fn infer(image: &CameraImage, radar: &RadarScan, scene: &mut MaybeUninit<Scene>) {
        let CameraImage {
            num_people,
            num_cars,
            distance_obstacle,
        } = *image;

        let distance_obstacle = distance_obstacle.min(radar.distance_obstacle);
        let distance_left_lane = gen_random_in_range(5..10) as f64 / 10.0;
        let distance_right_lane = gen_random_in_range(5..10) as f64 / 10.0;

        // Get raw pointer to payload within `MaybeUninit`.
        let scene_ptr = scene.as_mut_ptr();

        // Safety: `scene_ptr` was create from a `MaybeUninit` of the right type and size.
        // The underlying type `Scene` has `repr(C)` and can be populated field by field.
        unsafe {
            (*scene_ptr).num_people = num_people;
            (*scene_ptr).num_cars = num_cars;
            (*scene_ptr).distance_obstacle = distance_obstacle;
            (*scene_ptr).distance_left_lane = distance_left_lane;
            (*scene_ptr).distance_right_lane = distance_right_lane;
        }
    }
}

impl Activity for NeuralNet {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    #[instrument(name = "NeuralNet startup")]
    fn startup(&mut self) -> Result<(), ActivityError> {
        Ok(())
    }

    #[instrument(name = "NeuralNet")]
    fn step(&mut self) -> Result<(), ActivityError> {
        debug!("Stepping NeuralNet");
        sleep_random();

        let camera = self.input_image.read();
        let radar = self.input_scan.read();

        debug!("Inferring scene with neural network");

        let mut scene = self.output_scene.allocate().unwrap();
        Self::infer(&camera, &radar, scene.as_mut());
        // Safety: `Scene` has `repr(C)` and was fully initialized by `Self::infer` above.
        let scene = unsafe { scene.assume_init() };
        debug!("Sending Scene {:?}", scene.deref());
        scene.send().unwrap();
        Ok(())
    }

    #[instrument(name = "NeuralNet shutdown")]
    fn shutdown(&mut self) -> Result<(), ActivityError> {
        debug!("Shutting down NeuralNet activity {}", self.activity_id);
        Ok(())
    }
}

/// Emergency braking activity
///
/// This component emulates an emergency braking function
/// which sends instructions to activate the brakes
/// if the distance to the closest obstacle becomes too small.
/// The level of brake engagement depends on the distance.
#[derive(Debug)]
pub struct EmergencyBraking {
    /// ID of the activity
    activity_id: ActivityId,
    /// Scene input
    input_scene: MwComInput<Scene>,
    /// Brake instruction output
    output_brake_instruction: DebugWrapper<<LolaRuntimeImpl as Runtime>::Publisher<BrakeInstruction>>,
}

impl EmergencyBraking {
    pub fn build(activity_id: ActivityId, scene_topic: &str, brake_instruction_topic: &str) -> Box<dyn Activity> {
        let output_brake_instruction =
            DebugWrapper(create_producer::<BrakeControllerInterface>(brake_instruction_topic).brake_instruction);
        Box::new(Self {
            activity_id,
            input_scene: MwComInput::new::<_, NeuralNetInterface>(scene_topic, |n: NeuralNetConsumer<_>| n.scene),
            output_brake_instruction,
        })
    }
}

impl Activity for EmergencyBraking {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    #[instrument(name = "EmergencyBraking startup")]
    fn startup(&mut self) -> Result<(), ActivityError> {
        Ok(())
    }

    #[instrument(name = "EmergencyBraking")]
    fn step(&mut self) -> Result<(), ActivityError> {
        debug!("Stepping EmergencyBraking");
        sleep_random();

        let scene = self.input_scene.read();
        let brake_instruction = self.output_brake_instruction.allocate().unwrap();

        const ENGAGE_DISTANCE: f64 = 30.0;
        const MAX_BRAKE_DISTANCE: f64 = 15.0;

        if scene.distance_obstacle < ENGAGE_DISTANCE {
            // Map distances ENGAGE_DISTANCE..MAX_BRAKE_DISTANCE to intensities 0.0..1.0
            let level = f64::min(
                1.0,
                (ENGAGE_DISTANCE - scene.distance_obstacle) / (ENGAGE_DISTANCE - MAX_BRAKE_DISTANCE),
            );

            let brake_instruction = brake_instruction.write(BrakeInstruction { active: true, level });
            brake_instruction.send().unwrap();
        } else {
            let brake_instruction = brake_instruction.write(BrakeInstruction {
                active: false,
                level: 0.0,
            });
            brake_instruction.send().unwrap();
        }
        Ok(())
    }

    #[instrument(name = "EmergencyBraking shutdown")]
    fn shutdown(&mut self) -> Result<(), ActivityError> {
        debug!("Shutting down EmergencyBraking activity {}", self.activity_id);
        Ok(())
    }
}

/// Brake controller activity
///
/// This component emulates a brake controller
/// which triggers the brakes based on an instruction
/// and therefore might run in a separate process
/// with only other ASIL-D activities.
#[derive(Debug)]
pub struct BrakeController {
    /// ID of the activity
    activity_id: ActivityId,
    /// Brake instruction input
    input_brake_instruction: MwComInput<BrakeInstruction>,
}

impl BrakeController {
    pub fn build(activity_id: ActivityId, brake_instruction_topic: &str) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            input_brake_instruction: MwComInput::new::<_, BrakeControllerInterface>(
                brake_instruction_topic,
                |b: BrakeControllerConsumer<_>| b.brake_instruction,
            ),
        })
    }
}

impl Activity for BrakeController {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    #[instrument(name = "BrakeController startup")]
    fn startup(&mut self) -> Result<(), ActivityError> {
        Ok(())
    }

    #[instrument(name = "BrakeController")]
    fn step(&mut self) -> Result<(), ActivityError> {
        debug!("Stepping BrakeController");
        sleep_random();

        let brake_instruction = self.input_brake_instruction.read();
        if brake_instruction.active {
            debug!(
                "BrakeController activating brakes with level {:.3}",
                brake_instruction.level
            )
        }
        Ok(())
    }

    #[instrument(name = "BrakeController shutdown")]
    fn shutdown(&mut self) -> Result<(), ActivityError> {
        debug!("Shutting down BrakeController activity {}", self.activity_id);
        Ok(())
    }
}

/// Environment renderer activity
///
/// This component emulates a renderer to display a scene
/// in the infotainment display.
/// In this example, it does not do anything with the scene input.
#[derive(Debug)]
pub struct EnvironmentRenderer {
    /// ID of the activity
    activity_id: ActivityId,
    /// Scene input
    input_scene: MwComInput<Scene>,
}

impl EnvironmentRenderer {
    pub fn build(activity_id: ActivityId, scene_topic: &str) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            input_scene: MwComInput::new::<_, NeuralNetInterface>(scene_topic, |n: NeuralNetConsumer<_>| n.scene),
        })
    }
}

impl Activity for EnvironmentRenderer {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    #[instrument(name = "EnvironmentRenderer startup")]
    fn startup(&mut self) -> Result<(), ActivityError> {
        Ok(())
    }

    #[instrument(name = "EnvironmentRenderer")]
    fn step(&mut self) -> Result<(), ActivityError> {
        debug!("Stepping EnvironmentRenderer");
        sleep_random();

        let _scene = self.input_scene.read();
        debug!("Rendering scene");
        Ok(())
    }

    #[instrument(name = "EnvironmentRenderer shutdown")]
    fn shutdown(&mut self) -> Result<(), ActivityError> {
        debug!("Shutting down EnvironmentRenderer activity {}", self.activity_id);
        Ok(())
    }
}

/// Steering controller activity
///
/// This component emulates a steering controller
/// which adjusts the steering angle to control the heading of the car.
/// Therefore, it might run in a separate process
/// with only other ASIL-D activities.
#[derive(Debug)]
pub struct SteeringController {
    /// ID of the activity
    activity_id: ActivityId,
    /// Steering input
    input_steering: MwComInput<Steering>,
}

impl SteeringController {
    pub fn build(activity_id: ActivityId, steering_topic: &str) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            input_steering: MwComInput::new::<_, SteeringControllerInterface>(
                steering_topic,
                |s: SteeringControllerConsumer<_>| s.steering,
            ),
        })
    }
}

impl Activity for SteeringController {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    #[instrument(name = "SteeringController startup")]
    fn startup(&mut self) -> Result<(), ActivityError> {
        Ok(())
    }

    #[instrument(name = "SteeringController")]
    fn step(&mut self) -> Result<(), ActivityError> {
        debug!("Stepping SteeringController");
        sleep_random();

        let steering = self.input_steering.read();
        debug!("SteeringController adjusting angle to {:.3}", steering.angle);
        Ok(())
    }

    #[instrument(name = "SteeringController shutdown")]
    fn shutdown(&mut self) -> Result<(), ActivityError> {
        debug!("Shutting down SteeringController activity {}", self.activity_id);
        Ok(())
    }
}

/// LaneAssist stub activity
///
#[derive(Debug)]
pub struct LaneAssist {
    /// ID of the activity
    activity_id: ActivityId,
    /// Brake instruction output
    steering_controller: DebugWrapper<<LolaRuntimeImpl as Runtime>::Publisher<Steering>>,
}

impl LaneAssist {
    pub fn build(activity_id: ActivityId, steering_topic: &str) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            steering_controller: DebugWrapper(create_producer::<SteeringControllerInterface>(steering_topic).steering),
        })
    }
}

impl Activity for LaneAssist {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    #[instrument(name = "LaneAssist startup")]
    fn startup(&mut self) -> Result<(), ActivityError> {
        Ok(())
    }

    #[instrument(name = "LaneAssist")]
    fn step(&mut self) -> Result<(), ActivityError> {
        debug!("Stepping LaneAssist");
        sleep_random();
        self.steering_controller.send(Steering { angle: 2.34 }).unwrap();
        Ok(())
    }

    #[instrument(name = "LaneAssist shutdown")]
    fn shutdown(&mut self) -> Result<(), ActivityError> {
        Ok(())
    }
}

/// TrajectoryVisualizer stub activity
///
#[derive(Debug)]
pub struct TrajectoryVisualizer {
    /// ID of the activity
    activity_id: ActivityId,
}

impl TrajectoryVisualizer {
    pub fn build(activity_id: ActivityId) -> Box<dyn Activity> {
        Box::new(Self { activity_id })
    }
}

impl Activity for TrajectoryVisualizer {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    #[instrument(name = "TrajectoryVisualizer startup")]
    fn startup(&mut self) -> Result<(), ActivityError> {
        Ok(())
    }

    #[instrument(name = "TrajectoryVisualizer")]
    fn step(&mut self) -> Result<(), ActivityError> {
        debug!("Stepping TrajectoryVisualizer");
        sleep_random();
        Ok(())
    }

    #[instrument(name = "TrajectoryVisualizer shutdown")]
    fn shutdown(&mut self) -> Result<(), ActivityError> {
        Ok(())
    }
}

/// Generate a pseudo-random number in the specified range.
fn gen_random_in_range(range: Range<i64>) -> i64 {
    let rand = RandomState::new().build_hasher().finish();
    let rand = (rand % (i64::MAX as u64)) as i64;
    rand % (range.end - range.start + 1) + range.start
}

/// Random walk from `previous` with a probability of `change_prop` in a range of +/-`max_delta`
fn random_walk_float(previous: f64, change_prop: f64, max_delta: f64) -> f64 {
    if gen_random_in_range(0..100) as f64 / 100.0 < change_prop {
        const SCALE_FACTOR: f64 = 1000.0;

        // Scale delta to work in integers
        let scaled_max_delta = (max_delta * SCALE_FACTOR) as i64;
        let scaled_delta = gen_random_in_range(-scaled_max_delta..scaled_max_delta) as f64;

        return previous + (scaled_delta / SCALE_FACTOR);
    }

    previous
}

/// Random walk from `previous` with a probability of `change_prop` in a range of +/-`max_delta`
fn random_walk_integer(previous: usize, change_prop: f64, max_delta: usize) -> usize {
    let max_delta = max_delta as i64;

    if gen_random_in_range(0..100) as f64 / 100.0 < change_prop {
        let delta = gen_random_in_range(-max_delta..max_delta);

        return i64::max(0, previous as i64 + delta) as usize;
    }

    previous
}

/// Sleep for a random amount of time
fn sleep_random() {
    thread::sleep(Duration::from_millis(gen_random_in_range(SLEEP_RANGE) as u64));
}

fn create_producer<I: Interface>(
    topic: &str,
) -> <<I as Interface>::Producer<LolaRuntimeImpl> as Producer<LolaRuntimeImpl>>::OfferedProducer {
    debug!("Creating MW COM Producer for {}...", topic);
    let service_id = InstanceSpecifier::new(topic).expect("Failed to create InstanceSpecifier");
    let producer_builder = mw_com_runtime().producer_builder::<I>(service_id);
    let producer = producer_builder.build().unwrap();
    producer.offer().expect("can't offer a producer")
}

fn create_consumer<I: Interface>(topic: &str) -> <I as Interface>::Consumer<LolaRuntimeImpl> {
    debug!("Creating MW COM Consumer for {}...", topic);
    let mut tries = 0;
    loop {
        let service_id = InstanceSpecifier::new(topic).expect("Failed to create InstanceSpecifier");
        let consumer_discovery = mw_com_runtime().find_service::<I>(FindServiceSpecifier::Specific(service_id));
        let available_service_instances = consumer_discovery.get_available_instances().unwrap();

        if available_service_instances.is_empty() {
            debug!("No producer yet available for {}, waiting...", topic);
            sleep(Duration::from_millis(500));
            tries += 1;
            if tries > 100 {
                break;
            }
            continue;
        }

        // Select service instance at specific handle_index
        let handle_index = 0; // or any index you need from vector of instances
        let consumer_builder = available_service_instances.into_iter().nth(handle_index).unwrap();

        return consumer_builder.build().unwrap();
    }
    panic!("can't create consumer for {topic}")
}
