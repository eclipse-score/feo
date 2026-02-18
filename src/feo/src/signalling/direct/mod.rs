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

//! Signalling layer with direct connection between scheduler and workers
//!
//! This module provides connector implementations which establish direct connections
//! between the scheduler and every worker.
//! The layer does not differentiate between connecting workers in other processes
//! or in the same process as the scheduler.

pub(crate) mod mpsc;
#[cfg(feature = "recording")]
pub(crate) mod recorder;
pub(crate) mod scheduler;
pub(crate) mod worker;
