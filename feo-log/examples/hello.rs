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

use core::time;
use feo_log::{Level, LevelFilter, error, log, warn};
use std::thread;

fn main() {
    feo_logger::init(LevelFilter::Trace, true, true);

    // Logs a static string on level `trace`.
    log!(Level::Trace, "Kick it");

    // Logs on level `debug` with `target` set to "hello".
    log!(
        target: "hello",
        Level::Debug,
        "You wake up late for school, man you don't want to go"
    );

    // Logs a format string on level `info`.
    log!(
        Level::Info,
        "You ask your mom, please? but she still says, {}!",
        "No"
    );

    // Logs a static string on level `warn`.
    warn!("You missed two classes");

    loop {
        // Logs a static string on level `error`.
        error!("And no homework");
        thread::sleep(time::Duration::from_secs(1));
    }
}
