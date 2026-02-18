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

/// The type of a thread id
pub type ThreadId = u32;

/// Get the current thread id
pub fn id() -> ThreadId {
    // Safety: gettid(2) says this never fails
    unsafe { libc::gettid() as u32 }
}
