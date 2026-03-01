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

#[cfg(cargo_build)]
mod perfetto_proto {
    pub mod perfetto {
        pub mod protos {
            include!(concat!(env!("OUT_DIR"), "/perfetto.protos.rs"));
        }
    }
}

#[cfg(not(cargo_build))]
pub use perfetto_proto::perfetto::protos::*;

#[cfg(cargo_build)]
pub use perfetto_proto::perfetto::protos::*;
