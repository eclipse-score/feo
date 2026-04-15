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

#[cfg(not(feature = "com_mw"))]
use feo_com::interface::ActivityInput;

#[cfg(feature = "com_mw")]
use crate::config::mw_com_runtime;

#[cfg(feature = "com_mw")]
use score_log::debug;

#[cfg(not(feature = "com_mw"))]
use feo_com::interface::FeoComData;

#[cfg(feature = "com_mw")]
use com_api::{
    Builder, FindServiceSpecifier, InstanceSpecifier, Interface, LolaRuntimeImpl, Runtime, ServiceDiscovery,
};

#[cfg(feature = "com_mw")]
macro_rules! input {
    ($interface:ident, $topic:ident, $mapping_fn:expr) => {
        Box::new(feo_com::mw_com::MwComInput::new(
            ($mapping_fn)($crate::activities::input::create_consumer::<$interface>($topic))
                .subscribe(1)
                .unwrap(),
            feo_com::interface::DebugWrapper(core::cell::RefCell::new(SampleContainer::new(1))),
        ))
    };
}

#[cfg(not(feature = "com_mw"))]
macro_rules! input {
    ($interface:ident, $topic:ident, $mapping_fn:expr) => {
        $crate::activities::input::activity_input($topic)
    };
}

pub(crate) use input;

#[cfg(feature = "com_mw")]
pub fn create_consumer<I: Interface + Send>(topic: &str) -> <I as Interface>::Consumer<LolaRuntimeImpl> {
    use feo_time::Duration;
    use std::thread::sleep;

    debug!("Creating MW COM Consumer for {}...", topic);
    let mut tries = 0;
    loop {
        let service_id = InstanceSpecifier::new(topic).expect("Failed to create InstanceSpecifier");
        let consumer_discovery = mw_com_runtime().find_service::<I>(FindServiceSpecifier::Specific(service_id));
        let available_service_instances = consumer_discovery.get_available_instances().unwrap();

        if available_service_instances.is_empty() {
            debug!("No producer yet available for {}, waiting...", topic);
            sleep(Duration::from_millis(500).into());
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

/// Create an activity input.
#[cfg(not(feature = "com_mw"))]
pub fn activity_input<T>(topic: &str) -> Box<dyn ActivityInput<T>>
where
    T: FeoComData + 'static,
{
    #[cfg(feature = "com_iox2")]
    use feo_com::iox2::Iox2Input;
    #[cfg(feature = "com_linux_shm")]
    use feo_com::linux_shm::LinuxShmInput;

    #[cfg(feature = "com_iox2")]
    return Box::new(Iox2Input::new(topic));
    #[cfg(feature = "com_linux_shm")]
    return Box::new(LinuxShmInput::new(topic));
}
