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

use feo_time::Duration;
use feo_time::Scaled;
use score_log::{debug, info, LevelFilter};
use std::thread;
use stdout_logger::StdoutLoggerBuilder;

fn main() {
    // Initialize logger.
    StdoutLoggerBuilder::new()
        .context("time-speedrun")
        .show_module(true)
        .show_file(true)
        .show_line(true)
        .log_level(LevelFilter::Debug)
        .set_as_default_logger();

    let start_instant_std = std::time::Instant::now();
    let start_instant_feo = feo_time::Instant::now();
    let start_systemtime_std = std::time::SystemTime::now();
    let start_systemtime_feo = feo_time::SystemTime::now();

    info!("Speeding up time by a factor of 2");
    feo_time::speed(2);

    for _ in 0..5 {
        debug!("Sleeping for 1 \"real\" second...");
        thread::sleep(core::time::Duration::from_secs(1));
        let start_systemtime_feo = start_systemtime_feo.elapsed().expect("time error");
        let start_instant_feo = start_instant_feo.elapsed();
        info!(
            "feo time since start: systemtime: {:?}, instant: {:?}",
            start_systemtime_feo, start_instant_feo
        );
        let start_systemtime_std = Duration(start_systemtime_std.elapsed().expect("time error"));
        let start_instant_std = Duration(start_instant_std.elapsed());
        info!(
            "std time since start: systemtime: {:?}, instant: {:?}",
            start_systemtime_std, start_instant_std
        );
    }

    // Scaling duration for thread::sleep. Use `scaled()` method to get the scaled duration
    // that matches the current time speed factor and feed it into `std::thread::sleep`.
    const SLEEP_DURATION: Duration = Duration::from_secs(1);
    let sleep_duration_scaled = SLEEP_DURATION.scaled();

    for _ in 0..5 {
        debug!(
            "Sleeping for {:?} (scaled: {:?})",
            SLEEP_DURATION, sleep_duration_scaled
        );
        thread::sleep(sleep_duration_scaled.into());
        let start_systemtime_feo = start_systemtime_feo.elapsed().expect("time error");
        let start_instant_feo = start_instant_feo.elapsed();
        info!(
            "feo time since start: systemtime: {:?}, instant: {:?}",
            start_systemtime_feo, start_instant_feo
        );
        let start_systemtime_std = Duration(start_systemtime_std.elapsed().expect("time error"));
        let start_instant_std = Duration(start_instant_std.elapsed());
        info!(
            "std time since start: systemtime: {:?}, instant: {:?}",
            start_systemtime_std, start_instant_std
        );
    }
}
