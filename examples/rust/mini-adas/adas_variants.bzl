# *******************************************************************************
# Copyright (c) 2026 Contributors to the Eclipse Foundation
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

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library")

COM_BACKENDS = [
    "com_iox2",
    "com_linux_shm",
    "com_mw",
]

SIGNALLINGS = [
    "direct_tcp",
    "direct_unix",
    "relayed_tcp",
    "relayed_unix",
    # Not yet merged
    # "direct_mw_com",
]

COMMON_DEPS = [
    "//src/feo:libfeo_rust",
    "//src/feo-com:libfeo_com_rust_mw_com",  # libfeo uses _mw_com feo-com, so we must use the same
    "//src/feo-time:libfeo_time_rust",
    "//src/feo-tracing:libfeo_tracing_rust",
    "@score_baselibs_rust//src/log/score_log",
    "@score_communication//score/mw/com/impl/rust/com-api/com-api",
    "//examples/rust/mini-adas/mini-adas-gen:mini_adas_gen_rs_mw_com",
]

LIBRARY_COMMON_DEPS = COMMON_DEPS + [
    "@score_crates//:tracing",
]

BINARY_COMMON_DEPS = COMMON_DEPS + [
    "@score_baselibs_rust//src/log/stdout_logger",
]

# Generate mini-adas bazel targets for each and every combination of COM backend
# and signalling implementation
# Input:
#   lib_srcs - source files of the library
#   primary_srcs - source files of the primary agent
#   secondary_srcs - source files of the secondary agent
#   primary_data - data files of the primary agent
#   secondary_data - data files of the secondary agent
#   env - envioronment variables of the binaries
# Generates adas_{com backend}_signalling_{signalling} library and two binaries:
#   adas_primary_{com backend}_signalling_{signalling}
#   and adas_secondary_{com backend}_signalling_{signalling}
def _adas_variants_impl(name, visibility, lib_srcs, primary_srcs, secondary_srcs, primary_data, secondary_data, env):
    for backend in COM_BACKENDS:
        for signalling in SIGNALLINGS:
            signalling_feature = "signalling_" + signalling
            crate_features = [backend, signalling_feature]
            library_name = "adas_{}_{}".format(backend, signalling)

            rust_library(
                name = library_name,
                crate_name = "adas",
                visibility = visibility,
                crate_features = crate_features,
                srcs = lib_srcs,
                deps = LIBRARY_COMMON_DEPS,
            )

            rust_binary(
                name = "adas_primary_{}_{}".format(backend, signalling),
                crate_name = "adas_primary",
                visibility = visibility,
                crate_features = crate_features,
                srcs = primary_srcs,
                data = primary_data,
                env = env,
                deps = BINARY_COMMON_DEPS + [":" + library_name],
            )

            rust_binary(
                name = "adas_secondary_{}_{}".format(backend, signalling),
                crate_name = "adas_secondary",
                visibility = visibility,
                crate_features = crate_features,
                srcs = secondary_srcs,
                data = secondary_data,
                env = env,
                deps = BINARY_COMMON_DEPS + [":" + library_name],
            )

adas_variants = macro(
    implementation = _adas_variants_impl,
    attrs = {
        "lib_srcs": attr.label_list(allow_files = True),
        "primary_srcs": attr.label_list(allow_files = True),
        "secondary_srcs": attr.label_list(allow_files = True),
        "primary_data": attr.label_list(allow_files = True, default = []),
        "secondary_data": attr.label_list(allow_files = True, default = []),
        "env": attr.string_dict(default = {}),
    },
)
