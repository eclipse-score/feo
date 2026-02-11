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

//! Activity and related structs and traits
use crate::error::ActivityError;
use crate::ids::ActivityId;
use alloc::boxed::Box;

/// Activity trait, to be implemented by any activity intended to run in a WorkerPool
pub trait Activity {
    /// Get the ID of the activity
    fn id(&self) -> ActivityId;

    /// Called upon startup
    fn startup(&mut self) -> Result<(), ActivityError>;

    /// Called upon each step
    fn step(&mut self) -> Result<(), ActivityError>;

    /// Called upon shutdown
    fn shutdown(&mut self) -> Result<(), ActivityError>;
}

/// Activity Builder trait.
///
/// To instantiate a worker pool with activities, an ActivityBuilder
/// shall be passed for each activity. At startup of the worker threads are started the
/// activities will be built within their respective thread.
/// In this way, activities can avoid implementing the Send trait, which may not
/// always be possible.
pub trait ActivityBuilder: FnOnce(ActivityId) -> Box<dyn Activity> + Send {}

impl<T: FnOnce(ActivityId) -> Box<dyn Activity> + Send> ActivityBuilder for T {}

/// [ActivityId] coupled with an [ActivityBuilder].
pub type ActivityIdAndBuilder = (ActivityId, Box<dyn ActivityBuilder>);
