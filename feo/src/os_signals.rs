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

use alloc::sync::Arc;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;

#[derive(Clone, Debug)]
pub struct SignalTriggeredFlagRef(Arc<AtomicBool>);

impl SignalTriggeredFlagRef {
    pub fn sigterm() -> Self {
        let flag = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&flag))
            .expect("can't register SIGTERM handler");
        Self(flag)
    }

    pub fn is_triggered(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}
