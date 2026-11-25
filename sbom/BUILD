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

load("@score_sbom//:defs.bzl", "sbom")

sbom(
    name = "sbom",
    component_license = "Apache-2.0",
    component_name = "score_feo",
    generation_context = "build",
    module_lockfiles = ["//:MODULE.bazel.lock"],
    output_formats = [
        "spdx",
        "cyclonedx",
    ],
    tags = ["manual"],
    targets = ["//src/feo:libfeo_rust"],
    visibility = ["//:__pkg__"],
)
