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

#ifndef FEO_LOGGER_H

#include <log.h>

namespace feo {
namespace logger {

/// Initialize the logger.
/// Declare the minimum log level, whether to log to the console, and whether to log to the system log.
void init(feo::log::LevelFilter level_filter, bool console, bool logd);

}  // namespace logger
}  // namespace feo

#endif
