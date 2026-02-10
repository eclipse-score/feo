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

//! Placeholder logging daemon that collects logs from various sources. Minimal effort implementation.

use anyhow::Error;
use feo_log::{LevelFilter, info};
use tokio::runtime;

fn main() -> Result<(), Error> {
    // Initialize the logger *without* the logd part logger.
    feo_logger::init(LevelFilter::Debug, true, false);

    info!("Starting logd");

    let logd = logd::run();

    runtime::Builder::new_current_thread()
        .enable_io()
        .build()?
        .block_on(logd)
}
