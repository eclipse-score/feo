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

#include <feo_time.h>
#include <gtest/gtest.h>

namespace time_test {
TEST(time, timespec) {
    struct feo_timespec ts;
    feo_clock_gettime(&ts);
    EXPECT_GT(ts.tv_sec, 0);
    EXPECT_GT(ts.tv_nsec, 0);
}
}  // namespace time_test
