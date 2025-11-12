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
load("@rules_rust//rust:defs.bzl", "rust_library_group")
load("@rules_rust_prost//:defs.bzl", "rust_prost_toolchain")

rust_library_group(
    name = "prost_runtime",
    deps = ["@score_crates//:prost"],
)

rust_library_group(
    name = "tonic_runtime",
    deps = [
        ":prost_runtime",
        "@score_crates//:tonic",
    ],
)

# Setup custom prost toolchain using crate versions defined in score_crates instead of default.
rust_prost_toolchain(
    name = "prost_toolchain_impl",
    prost_plugin = "@score_crates//:protoc_gen_prost__protoc_gen_prost",
    prost_runtime = ":prost_runtime",
    prost_types = "@score_crates//:prost_types",
    tonic_plugin = "@score_crates//:protoc_gen_tonic__protoc_gen_tonic",
    tonic_runtime = ":tonic_runtime",
)

toolchain(
    name = "prost_toolchain",
    toolchain = "prost_toolchain_impl",
    toolchain_type = "@rules_rust_prost//:toolchain_type",
)
