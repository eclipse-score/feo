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

use com_api::{
    CommData, Consumer, Interface, OfferedProducer, Producer, ProviderInfo, Publisher, Reloc, Runtime, Subscriber,
};
use score_log::ScoreDebug;

/// Camera image
///
/// A neural network could detect the number of people,
/// number of cars and the distance to the closest obstacle.
/// Given that we do not have a real neural network,
/// we already include information to be dummy inferred.
#[derive(Debug, Default, Reloc, ScoreDebug)]
#[repr(C)]
pub struct CameraImage {
    pub num_people: libc::size_t,
    pub num_cars: libc::size_t,
    pub distance_obstacle: f64,
}

impl CommData for CameraImage {
    const ID: &'static str = "CameraImage";
}

pub struct CameraInterface;

impl Interface for CameraInterface {
    const INTERFACE_ID: &'static str = "CameraInterface";
    type Consumer<R: Runtime + ?Sized> = CameraConsumer<R>;
    type Producer<R: Runtime + ?Sized> = CameraProducer<R>;
}

pub struct CameraConsumer<R: Runtime + ?Sized> {
    pub image: R::Subscriber<CameraImage>,
}

impl<R: Runtime + ?Sized> Consumer<R> for CameraConsumer<R> {
    fn new(instance_info: R::ConsumerInfo) -> Self {
        CameraConsumer {
            image: R::Subscriber::new("image", instance_info.clone()).expect("Failed to create subscriber"),
        }
    }
}

pub struct CameraProducer<R: Runtime + ?Sized> {
    _runtime: core::marker::PhantomData<R>,
    instance_info: R::ProviderInfo,
}

impl<R: Runtime + ?Sized> Producer<R> for CameraProducer<R> {
    type Interface = CameraInterface;
    type OfferedProducer = CameraOfferedProducer<R>;

    fn offer(self) -> com_api::Result<Self::OfferedProducer> {
        let offered_producer = CameraOfferedProducer {
            image: R::Publisher::new("image", self.instance_info.clone()).expect("Failed to create publisher"),
            instance_info: self.instance_info.clone(),
        };
        // Offer the service instance to make it discoverable
        // this is called after skeleton created using producer_builder API
        self.instance_info.offer_service()?;
        Ok(offered_producer)
    }

    fn new(instance_info: R::ProviderInfo) -> com_api::Result<Self> {
        Ok(CameraProducer {
            _runtime: core::marker::PhantomData,
            instance_info,
        })
    }
}

pub struct CameraOfferedProducer<R: Runtime + ?Sized> {
    pub image: R::Publisher<CameraImage>,
    instance_info: R::ProviderInfo,
}

impl<R: Runtime + ?Sized> OfferedProducer<R> for CameraOfferedProducer<R> {
    type Interface = CameraInterface;
    type Producer = CameraProducer<R>;

    fn unoffer(self) -> com_api::Result<Self::Producer> {
        let producer = CameraProducer {
            _runtime: std::marker::PhantomData,
            instance_info: self.instance_info.clone(),
        };
        // Stop offering the service instance to withdraw it from system availability
        self.instance_info.stop_offer_service()?;
        Ok(producer)
    }
}

/// Radar scan
///
/// With post-processing, we could detect the closest object
/// from a real radar scan. In this example,
/// the message type already carries the information to be dummy extracted.
#[derive(Debug, Default, Reloc, ScoreDebug)]
#[repr(C)]
pub struct RadarScan {
    pub distance_obstacle: f64,
    pub error_margin: f64,
}

impl CommData for RadarScan {
    const ID: &'static str = "RadarScan";
}

pub struct RadarInterface;

impl Interface for RadarInterface {
    const INTERFACE_ID: &'static str = "RadarInterface";
    type Consumer<R: Runtime + ?Sized> = RadarConsumer<R>;
    type Producer<R: Runtime + ?Sized> = RadarProducer<R>;
}

pub struct RadarConsumer<R: Runtime + ?Sized> {
    pub scan: R::Subscriber<RadarScan>,
}

impl<R: Runtime + ?Sized> Consumer<R> for RadarConsumer<R> {
    fn new(instance_info: R::ConsumerInfo) -> Self {
        RadarConsumer {
            scan: R::Subscriber::new("scan", instance_info.clone()).expect("Failed to create subscriber"),
        }
    }
}

pub struct RadarProducer<R: Runtime + ?Sized> {
    _runtime: core::marker::PhantomData<R>,
    instance_info: R::ProviderInfo,
}

impl<R: Runtime + ?Sized> Producer<R> for RadarProducer<R> {
    type Interface = RadarInterface;
    type OfferedProducer = RadarOfferedProducer<R>;

    fn offer(self) -> com_api::Result<Self::OfferedProducer> {
        let offered_producer = RadarOfferedProducer {
            scan: R::Publisher::new("scan", self.instance_info.clone()).expect("Failed to create publisher"),
            instance_info: self.instance_info.clone(),
        };
        // Offer the service instance to make it discoverable
        // this is called after skeleton created using producer_builder API
        self.instance_info.offer_service()?;
        Ok(offered_producer)
    }

    fn new(instance_info: R::ProviderInfo) -> com_api::Result<Self> {
        Ok(RadarProducer {
            _runtime: core::marker::PhantomData,
            instance_info,
        })
    }
}

pub struct RadarOfferedProducer<R: Runtime + ?Sized> {
    pub scan: R::Publisher<RadarScan>,
    instance_info: R::ProviderInfo,
}

impl<R: Runtime + ?Sized> OfferedProducer<R> for RadarOfferedProducer<R> {
    type Interface = RadarInterface;
    type Producer = RadarProducer<R>;

    fn unoffer(self) -> com_api::Result<Self::Producer> {
        let producer = RadarProducer {
            _runtime: std::marker::PhantomData,
            instance_info: self.instance_info.clone(),
        };
        // Stop offering the service instance to withdraw it from system availability
        self.instance_info.stop_offer_service()?;
        Ok(producer)
    }
}

/// Scene
///
/// The scene is the result of fusing the camera image and the radar scan
/// with a neural network. In our example, we just extract the information.
#[derive(Debug, Default, Reloc, ScoreDebug)]
#[repr(C)]
pub struct Scene {
    pub num_people: usize,
    pub num_cars: usize,
    pub distance_obstacle: f64,
    pub distance_left_lane: f64,
    pub distance_right_lane: f64,
}

impl CommData for Scene {
    const ID: &'static str = "Scene";
}

pub struct NeuralNetInterface;

impl Interface for NeuralNetInterface {
    const INTERFACE_ID: &'static str = "NeuralNetInterface";
    type Consumer<R: Runtime + ?Sized> = NeuralNetConsumer<R>;
    type Producer<R: Runtime + ?Sized> = NeuralNetProducer<R>;
}

pub struct NeuralNetConsumer<R: Runtime + ?Sized> {
    pub scene: R::Subscriber<Scene>,
}

impl<R: Runtime + ?Sized> Consumer<R> for NeuralNetConsumer<R> {
    fn new(instance_info: R::ConsumerInfo) -> Self {
        NeuralNetConsumer {
            scene: R::Subscriber::new("scene", instance_info.clone()).expect("Failed to create subscriber"),
        }
    }
}

pub struct NeuralNetProducer<R: Runtime + ?Sized> {
    _runtime: core::marker::PhantomData<R>,
    instance_info: R::ProviderInfo,
}

impl<R: Runtime + ?Sized> Producer<R> for NeuralNetProducer<R> {
    type Interface = NeuralNetInterface;
    type OfferedProducer = NeuralNetOfferedProducer<R>;

    fn offer(self) -> com_api::Result<Self::OfferedProducer> {
        let offered_producer = NeuralNetOfferedProducer {
            scene: R::Publisher::new("scene", self.instance_info.clone()).expect("Failed to create publisher"),
            instance_info: self.instance_info.clone(),
        };
        // Offer the service instance to make it discoverable
        // this is called after skeleton created using producer_builder API
        self.instance_info.offer_service()?;
        Ok(offered_producer)
    }

    fn new(instance_info: R::ProviderInfo) -> com_api::Result<Self> {
        Ok(NeuralNetProducer {
            _runtime: core::marker::PhantomData,
            instance_info,
        })
    }
}

pub struct NeuralNetOfferedProducer<R: Runtime + ?Sized> {
    pub scene: R::Publisher<Scene>,
    instance_info: R::ProviderInfo,
}

impl<R: Runtime + ?Sized> OfferedProducer<R> for NeuralNetOfferedProducer<R> {
    type Interface = NeuralNetInterface;
    type Producer = NeuralNetProducer<R>;

    fn unoffer(self) -> com_api::Result<Self::Producer> {
        let producer = NeuralNetProducer {
            _runtime: std::marker::PhantomData,
            instance_info: self.instance_info.clone(),
        };
        // Stop offering the service instance to withdraw it from system availability
        self.instance_info.stop_offer_service()?;
        Ok(producer)
    }
}

/// Brake instruction
///
/// This is an instruction whether to engage the brakes and at which level.
#[derive(Debug, Default, Reloc, ScoreDebug)]
#[repr(C)]
pub struct BrakeInstruction {
    pub active: bool,
    pub level: f64,
}

impl CommData for BrakeInstruction {
    const ID: &'static str = "BrakeInstruction";
}

pub struct BrakeControllerInterface;

impl Interface for BrakeControllerInterface {
    const INTERFACE_ID: &'static str = "BrakeControllerInterface";
    type Consumer<R: Runtime + ?Sized> = BrakeControllerConsumer<R>;
    type Producer<R: Runtime + ?Sized> = BrakeControllerProducer<R>;
}

pub struct BrakeControllerConsumer<R: Runtime + ?Sized> {
    pub brake_instruction: R::Subscriber<BrakeInstruction>,
}

impl<R: Runtime + ?Sized> Consumer<R> for BrakeControllerConsumer<R> {
    fn new(instance_info: R::ConsumerInfo) -> Self {
        BrakeControllerConsumer {
            brake_instruction: R::Subscriber::new("brake_instruction", instance_info.clone())
                .expect("Failed to create subscriber"),
        }
    }
}

pub struct BrakeControllerProducer<R: Runtime + ?Sized> {
    _runtime: core::marker::PhantomData<R>,
    instance_info: R::ProviderInfo,
}

impl<R: Runtime + ?Sized> Producer<R> for BrakeControllerProducer<R> {
    type Interface = BrakeControllerInterface;
    type OfferedProducer = BrakeControllerOfferedProducer<R>;

    fn offer(self) -> com_api::Result<Self::OfferedProducer> {
        let offered_producer = BrakeControllerOfferedProducer {
            brake_instruction: R::Publisher::new("brake_instruction", self.instance_info.clone())
                .expect("Failed to create publisher"),
            instance_info: self.instance_info.clone(),
        };
        // Offer the service instance to make it discoverable
        // this is called after skeleton created using producer_builder API
        self.instance_info.offer_service()?;
        Ok(offered_producer)
    }

    fn new(instance_info: R::ProviderInfo) -> com_api::Result<Self> {
        Ok(BrakeControllerProducer {
            _runtime: core::marker::PhantomData,
            instance_info,
        })
    }
}

pub struct BrakeControllerOfferedProducer<R: Runtime + ?Sized> {
    pub brake_instruction: R::Publisher<BrakeInstruction>,
    instance_info: R::ProviderInfo,
}

impl<R: Runtime + ?Sized> OfferedProducer<R> for BrakeControllerOfferedProducer<R> {
    type Interface = BrakeControllerInterface;
    type Producer = BrakeControllerProducer<R>;

    fn unoffer(self) -> com_api::Result<Self::Producer> {
        let producer = BrakeControllerProducer {
            _runtime: std::marker::PhantomData,
            instance_info: self.instance_info.clone(),
        };
        // Stop offering the service instance to withdraw it from system availability
        self.instance_info.stop_offer_service()?;
        Ok(producer)
    }
}

/// Steering
///
/// This carries the angle of steering.
#[derive(Debug, Default, Reloc, ScoreDebug)]
#[repr(C)]
pub struct Steering {
    pub angle: f64,
}

impl CommData for Steering {
    const ID: &'static str = "Steering";
}

pub struct SteeringControllerInterface;

impl Interface for SteeringControllerInterface {
    const INTERFACE_ID: &'static str = "SteeringControllerInterface";
    type Consumer<R: Runtime + ?Sized> = SteeringControllerConsumer<R>;
    type Producer<R: Runtime + ?Sized> = SteeringControllerProducer<R>;
}

pub struct SteeringControllerConsumer<R: Runtime + ?Sized> {
    pub steering: R::Subscriber<Steering>,
}

impl<R: Runtime + ?Sized> Consumer<R> for SteeringControllerConsumer<R> {
    fn new(instance_info: R::ConsumerInfo) -> Self {
        SteeringControllerConsumer {
            steering: R::Subscriber::new("steering", instance_info.clone()).expect("Failed to create subscriber"),
        }
    }
}

pub struct SteeringControllerProducer<R: Runtime + ?Sized> {
    _runtime: core::marker::PhantomData<R>,
    instance_info: R::ProviderInfo,
}

impl<R: Runtime + ?Sized> Producer<R> for SteeringControllerProducer<R> {
    type Interface = SteeringControllerInterface;
    type OfferedProducer = SteeringControllerOfferedProducer<R>;

    fn offer(self) -> com_api::Result<Self::OfferedProducer> {
        let offered_producer = SteeringControllerOfferedProducer {
            steering: R::Publisher::new("steering", self.instance_info.clone()).expect("Failed to create publisher"),
            instance_info: self.instance_info.clone(),
        };
        // Offer the service instance to make it discoverable
        // this is called after skeleton created using producer_builder API
        self.instance_info.offer_service()?;
        Ok(offered_producer)
    }

    fn new(instance_info: R::ProviderInfo) -> com_api::Result<Self> {
        Ok(SteeringControllerProducer {
            _runtime: core::marker::PhantomData,
            instance_info,
        })
    }
}

pub struct SteeringControllerOfferedProducer<R: Runtime + ?Sized> {
    pub steering: R::Publisher<Steering>,
    instance_info: R::ProviderInfo,
}

impl<R: Runtime + ?Sized> OfferedProducer<R> for SteeringControllerOfferedProducer<R> {
    type Interface = SteeringControllerInterface;
    type Producer = SteeringControllerProducer<R>;

    fn unoffer(self) -> com_api::Result<Self::Producer> {
        let producer = SteeringControllerProducer {
            _runtime: std::marker::PhantomData,
            instance_info: self.instance_info.clone(),
        };
        // Stop offering the service instance to withdraw it from system availability
        self.instance_info.stop_offer_service()?;
        Ok(producer)
    }
}
