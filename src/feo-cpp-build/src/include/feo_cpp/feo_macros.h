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

#ifndef FEO_MACROS_H
#define FEO_MACROS_H

#define MAKE_ACTIVITY(class_name, prefix) extern "C" {  \
\
    void* prefix##_create(const uint64_t activity_id) { \
        class_name* activity = new class_name(activity_id); \
    	return (void*)activity; \
    }\
\
    void prefix##_startup(void* const activity_p) { \
        class_name* const activity = (class_name*)activity_p; \
    	activity->startup(); \
    }\
\
    void prefix##_step(void* const activity_p) { \
        class_name* const activity = (class_name*)activity_p; \
    	activity->step(); \
    }\
\
    void prefix##_shutdown(void* const activity_p) { \
        class_name* const activity = (class_name*)activity_p; \
    	activity->shutdown(); \
    }\
\
    void prefix##_free(void* const activity_p) { \
        class_name* const activity = (class_name*)activity_p; \
    	delete activity; \
    }\
}


#endif //FEO_MACROS_H
