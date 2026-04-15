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
#[cfg(feature = "mw_com")]
mod mini_adas_gen_mw_com;

#[cfg(feature = "mw_com")]
pub use mini_adas_gen_mw_com::*;

#[cfg(feature = "mw_com")]
use com_api::{PlacementDefault, Reloc};

use score_log::ScoreDebug;

/// Camera image
///
/// A neural network could detect the number of people,
/// number of cars and the distance to the closest obstacle.
/// Given that we do not have a real neural network,
/// we already include information to be dummy inferred.
#[derive(Debug, Default, ScoreDebug)]
#[repr(C)]
#[cfg_attr(feature = "mw_com", derive(Reloc))]
pub struct CameraImage {
    pub num_people: libc::size_t,
    pub num_cars: libc::size_t,
    pub distance_obstacle: f64,
}

#[cfg(feature = "mw_com")]
// SAFETY: only writes via field access
unsafe impl PlacementDefault for CameraImage {
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // part of MW COM API
    fn placement_default(s: *mut Self) {
        unsafe {
            (*s).num_people = 0;
            (*s).num_cars = 0;
            (*s).distance_obstacle = 0.;
        }
    }
}

/// Radar scan
///
/// With post-processing, we could detect the closest object
/// from a real radar scan. In this example,
/// the message type already carries the information to be dummy extracted.
#[derive(Debug, Default, ScoreDebug)]
#[repr(C)]
#[cfg_attr(feature = "mw_com", derive(Reloc))]
pub struct RadarScan {
    pub distance_obstacle: f64,
    pub error_margin: f64,
}

#[cfg(feature = "mw_com")]
// SAFETY: only writes via field access
unsafe impl PlacementDefault for RadarScan {
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // part of MW COM API
    fn placement_default(s: *mut Self) {
        unsafe {
            (*s).distance_obstacle = 0.;
            (*s).error_margin = 0.;
        }
    }
}

/// Scene
///
/// The scene is the result of fusing the camera image and the radar scan
/// with a neural network. In our example, we just extract the information.
#[derive(Debug, Default, ScoreDebug)]
#[repr(C)]
#[cfg_attr(feature = "mw_com", derive(Reloc))]
pub struct Scene {
    pub num_people: usize,
    pub num_cars: usize,
    pub distance_obstacle: f64,
    pub distance_left_lane: f64,
    pub distance_right_lane: f64,
}

#[cfg(feature = "mw_com")]
// SAFETY: only writes via field access
unsafe impl PlacementDefault for Scene {
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // part of MW COM API
    fn placement_default(s: *mut Self) {
        unsafe {
            (*s).num_people = 0;
            (*s).num_cars = 0;
            (*s).distance_obstacle = 0.;
            (*s).distance_left_lane = 0.;
            (*s).distance_right_lane = 0.;
        }
    }
}

/// Brake instruction
///
/// This is an instruction whether to engage the brakes and at which level.
#[derive(Debug, Default, ScoreDebug)]
#[repr(C)]
#[cfg_attr(feature = "mw_com", derive(Reloc))]
pub struct BrakeInstruction {
    pub active: bool,
    pub level: f64,
}

#[cfg(feature = "mw_com")]
// SAFETY: only writes via field access
unsafe impl PlacementDefault for BrakeInstruction {
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // part of MW COM API
    fn placement_default(s: *mut Self) {
        unsafe {
            (*s).active = false;
            (*s).level = 0.;
        }
    }
}

/// Steering
///
/// This carries the angle of steering.
#[derive(Debug, Default, ScoreDebug)]
#[repr(C)]
#[cfg_attr(feature = "mw_com", derive(Reloc))]
pub struct Steering {
    pub angle: f64,
}

#[cfg(feature = "mw_com")]
// SAFETY: only writes via field access
unsafe impl PlacementDefault for Steering {
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // part of MW COM API
    fn placement_default(s: *mut Self) {
        unsafe {
            (*s).angle = 0.;
        }
    }
}
