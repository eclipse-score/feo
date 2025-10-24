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

load("@rules_rust//rust:defs.bzl", "rust_test")

rust_test(
    name = "feo_tests",
    size = "small",
    srcs = [
        "src/lib.rs",
        "src/monitor.rs",
    ],
    data = ["//tests/rust/feo_tests/test_agent"],
    edition = "2021",
    deps = [
        "@score_crates//:clap",
        "@score_crates//:env_logger",
        "@score_crates//:ipc_channel",
        "@score_crates//:log",
        "@score_crates//:nix",
        "@score_crates//:serde",
    ],
)
