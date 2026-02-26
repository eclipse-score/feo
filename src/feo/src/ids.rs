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

//! IDs activities, agents, channels, relays, and workers

use core::fmt;
use core::hash::Hash;
use core::marker::PhantomData;
#[cfg(feature = "recording")]
use postcard::experimental::max_size::MaxSize;
use score_log::fmt::ScoreDebug;
use score_log::ScoreDebug;
#[cfg(feature = "recording")]
use serde::{Deserialize, Serialize};

/// Identifies an activity / task
pub type ActivityId = GenericId<ActivityIdMarker>;

#[cfg_attr(feature = "recording", derive(Serialize, Deserialize, MaxSize))]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActivityIdMarker;

impl GetPrefix for ActivityIdMarker {
    fn prefix() -> &'static str {
        "A"
    }
}

/// Identifies an agent / process
pub type AgentId = GenericId<AgentIdMarker>;

#[cfg_attr(feature = "recording", derive(Serialize, Deserialize, MaxSize))]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AgentIdMarker;

impl GetPrefix for AgentIdMarker {
    fn prefix() -> &'static str {
        "Agt-"
    }
}

/// Identifies a communication channel peer
#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, ScoreDebug)]
#[non_exhaustive]
pub enum ChannelId {
    Activity(ActivityId),
    Agent(AgentId),
    Worker(WorkerId),
    Relay(RelayId),
}

impl fmt::Display for ChannelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t: (&str, u64) = match self {
            ChannelId::Activity(id) => (ActivityIdMarker::prefix(), id.into()),
            ChannelId::Agent(id) => (AgentIdMarker::prefix(), id.into()),
            ChannelId::Worker(id) => (WorkerIdMarker::prefix(), id.into()),
            ChannelId::Relay(id) => (RelayIdMarker::prefix(), id.into()),
        };
        write!(f, "C({}{})", t.0, t.1)
    }
}

/// Identifies a worker
pub type WorkerId = GenericId<WorkerIdMarker>;
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct WorkerIdMarker;

impl GetPrefix for WorkerIdMarker {
    fn prefix() -> &'static str {
        "W"
    }
}

/// Identifies a signalling relay
pub type RelayId = GenericId<RelayIdMarker>;

#[cfg_attr(feature = "recording", derive(Serialize, Deserialize, MaxSize))]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct RelayIdMarker;

impl GetPrefix for RelayIdMarker {
    fn prefix() -> &'static str {
        "R"
    }
}

pub trait GetPrefix: fmt::Debug + Clone + Copy + Hash + PartialEq + Eq {
    fn prefix() -> &'static str {
        // Provide an empty string slice by default, can be overwritten
        ""
    }
}

#[cfg_attr(feature = "recording", derive(Serialize, Deserialize, MaxSize))]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GenericId<T: GetPrefix> {
    id: u64,
    // Marker to make distinct ID types not interchangeable
    _discriminator: PhantomData<T>,
}

impl<T: GetPrefix> ScoreDebug for GenericId<T> {
    fn fmt(
        &self,
        f: &mut dyn score_log::fmt::ScoreWrite,
        spec: &score_log::fmt::FormatSpec,
    ) -> Result<(), score_log::fmt::Error> {
        f.write_u64(&self.id, spec)
    }
}

impl<T: GetPrefix> GenericId<T> {
    pub const fn new(id: u64) -> Self {
        Self {
            id,
            _discriminator: PhantomData,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

impl<T: GetPrefix> From<u64> for GenericId<T> {
    fn from(value: u64) -> Self {
        Self {
            id: value,
            _discriminator: PhantomData,
        }
    }
}

impl<T: GetPrefix> From<&GenericId<T>> for u64 {
    fn from(value: &GenericId<T>) -> Self {
        value.id
    }
}

impl<T: GetPrefix> From<GenericId<T>> for u64 {
    fn from(value: GenericId<T>) -> Self {
        Self::from(&value)
    }
}

impl<T: GetPrefix> fmt::Display for GenericId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", T::prefix(), self.id)
    }
}
