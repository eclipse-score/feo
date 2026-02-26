# *******************************************************************************
# Copyright (c) 2025 Contributors to the Eclipse Foundation
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache License Version 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0
#
# SPDX-License-Identifier: Apache-2.0
# *******************************************************************************

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_test")

rust_binary(
    name = "test_agent",
    srcs = [
        "src/activities.rs",
        "src/config.rs",
        "src/main.rs",
        "src/monitor.rs",
        "src/primary.rs",
        "src/scenario.rs",
        "src/secondary.rs",
    ],
    crate_features = [
        "com_linux_shm",
    ],
    rustc_flags = [
        "-Clink-arg=-lstdc++",
        "-Clink-arg=-lm",
        "-Clink-arg=-lc",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//src/feo:libfeo_rust",
        "//src/feo-com:libfeo_com_rust",
        "//src/feo-time:libfeo_time_rust",
        "//src/feo-tracing:libfeo_tracing_rust",
        "@score_baselibs_rust//src/log/score_log",
        "@score_baselibs_rust//src/log/stdout_logger",
        "@score_crates//:clap",
        "@score_crates//:ipc_channel",
        "@score_crates//:libc",
        "@score_crates//:serde",
        "@score_crates//:tracing",
    ],
)
