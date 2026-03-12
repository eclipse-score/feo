/********************************************************************************
 * Copyright (c) 2026 Contributors to the Eclipse Foundation
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

#[cfg(feature = "com_mw")]
use crate::config::mw_com_runtime;
#[cfg(feature = "com_mw")]
use com_api::{Builder, InstanceSpecifier, Interface, LolaRuntimeImpl, Producer, Runtime};
#[cfg(not(feature = "com_mw"))]
use core::fmt::Debug;
#[cfg(not(feature = "com_mw"))]
use feo_com::interface::ActivityOutput;
#[cfg(feature = "com_mw")]
use score_log::debug;
#[cfg(not(feature = "com_mw"))]
use score_log::fmt::ScoreDebug;

#[cfg(feature = "com_iox2")]
use feo_com::iox2::Iox2Output;
#[cfg(feature = "com_linux_shm")]
use feo_com::linux_shm::LinuxShmOutput;

#[cfg(feature = "com_mw")]
macro_rules! output {
    ($interface:ident, $topic:ident, $mapping_fn:expr) => {
        Box::new(feo_com::mw_com::MwComOutput::new(feo_com::interface::DebugWrapper(
            ($mapping_fn)($crate::activities::output::create_producer::<$interface>($topic)),
        )))
    };
}

#[cfg(not(feature = "com_mw"))]
macro_rules! output {
    ($interface:ident, $topic:ident, $mapping_fn:expr) => {
        $crate::activities::output::activity_output($topic)
    };
}

pub(crate) use output;

#[cfg(feature = "com_mw")]
pub fn create_producer<I: Interface>(
    topic: &str,
) -> <<I as Interface>::Producer<LolaRuntimeImpl> as Producer<LolaRuntimeImpl>>::OfferedProducer {
    debug!("Creating MW COM Producer for {}...", topic);
    let service_id = InstanceSpecifier::new(topic).expect("Failed to create InstanceSpecifier");
    let producer_builder = mw_com_runtime().producer_builder::<I>(service_id);
    let producer = producer_builder.build().unwrap();
    producer.offer().expect("can't offer a producer")
}

/// Create an activity output.
#[cfg(not(feature = "com_mw"))]
pub fn activity_output<T>(topic: &str) -> Box<dyn ActivityOutput<T>>
where
    T: Debug + ScoreDebug + 'static,
{
    #[cfg(feature = "com_iox2")]
    return Box::new(Iox2Output::new(topic));
    #[cfg(feature = "com_linux_shm")]
    return Box::new(LinuxShmOutput::new(topic));
}
