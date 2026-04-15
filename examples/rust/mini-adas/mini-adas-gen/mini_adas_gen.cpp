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

#include "mini_adas_gen.h"
#include "score/mw/com/impl/rust/com-api/com-api-ffi-lola/registry_bridge_macro.h"

BEGIN_EXPORT_MW_COM_INTERFACE(CameraInterface, ::score::feo::com::CameraProxy, ::score::feo::com::CameraSkeleton)
EXPORT_MW_COM_EVENT(::score::feo::com::CameraImage, image)
END_EXPORT_MW_COM_INTERFACE()

EXPORT_MW_COM_TYPE(CameraImage, ::score::feo::com::CameraImage)

BEGIN_EXPORT_MW_COM_INTERFACE(RadarInterface, ::score::feo::com::RadarProxy, ::score::feo::com::RadarSkeleton)
EXPORT_MW_COM_EVENT(::score::feo::com::RadarScan, scan)
END_EXPORT_MW_COM_INTERFACE()
EXPORT_MW_COM_TYPE(RadarScan, ::score::feo::com::RadarScan)

BEGIN_EXPORT_MW_COM_INTERFACE(NeuralNetInterface, ::score::feo::com::NeuralNetProxy, ::score::feo::com::NeuralNetSkeleton)
EXPORT_MW_COM_EVENT(::score::feo::com::Scene, scene)
END_EXPORT_MW_COM_INTERFACE()
EXPORT_MW_COM_TYPE(Scene, ::score::feo::com::Scene)

BEGIN_EXPORT_MW_COM_INTERFACE(BrakeControllerInterface, ::score::feo::com::BrakeControllerProxy, ::score::feo::com::BrakeControllerSkeleton)
EXPORT_MW_COM_EVENT(::score::feo::com::BrakeInstruction, brake_instruction)
END_EXPORT_MW_COM_INTERFACE()
EXPORT_MW_COM_TYPE(BrakeInstruction, ::score::feo::com::BrakeInstruction)

BEGIN_EXPORT_MW_COM_INTERFACE(SteeringControllerInterface, ::score::feo::com::SteeringControllerProxy, ::score::feo::com::SteeringControllerSkeleton)
EXPORT_MW_COM_EVENT(::score::feo::com::Steering, steering)
END_EXPORT_MW_COM_INTERFACE()
EXPORT_MW_COM_TYPE(Steering, ::score::feo::com::Steering)
