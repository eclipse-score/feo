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

#include <logger.h>

extern "C" void __init(int level_filter, bool console, bool logd);

namespace feo {
namespace logger {

void init(feo::log::LevelFilter level_filter, bool console, bool logd) {
    __init(level_filter, console, logd);
}

}  // namespace logger
}  // namespace feo