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

use crate::SystemTime;

/// Time in seconds and nanoseconds.
#[repr(C)]
struct FeoTimeSpec {
    tv_sec: u64,
    tv_nsec: u32,
}

/// Set the clock speed factor.
#[unsafe(no_mangle)]
extern "C" fn feo_clock_speed(factor: i32) {
    crate::speed(factor);
}

/// Get the current time.
#[unsafe(no_mangle)]
extern "C" fn feo_clock_gettime(ts: *mut FeoTimeSpec) {
    debug_assert!(!ts.is_null());

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("time error");

    unsafe {
        (*ts).tv_sec = now.as_secs();
        (*ts).tv_nsec = now.subsec_nanos();
    }
}
