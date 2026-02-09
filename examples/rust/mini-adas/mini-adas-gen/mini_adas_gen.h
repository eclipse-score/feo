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
#ifndef SCORE_MINI_ADAS_DATATYPE_H
#define SCORE_MINI_ADAS_DATATYPE_H

#include "score/mw/com/types.h"

namespace score::feo::com
{

struct CameraImage
{
    CameraImage() = default;
    CameraImage(CameraImage&&) = default;
    CameraImage(const CameraImage&) = default;
    CameraImage& operator=(CameraImage&&) = default;
    CameraImage& operator=(const CameraImage&) = default;
    std::size_t num_people;
    std::size_t num_cars;
    std::double_t distance_obstacle;
};

template <typename Trait>
class CameraInterface : public Trait::Base
{
  public:
    using Trait::Base::Base;
    typename Trait::template Event<CameraImage> image{*this, "image"};
};

using CameraProxy = mw::com::AsProxy<CameraInterface>;
using CameraSkeleton = mw::com::AsSkeleton<CameraInterface>;

struct RadarScan
{
    RadarScan() = default;
    RadarScan(RadarScan&&) = default;
    RadarScan(const RadarScan&) = default;
    RadarScan& operator=(RadarScan&&) = default;
    RadarScan& operator=(const RadarScan&) = default;

    std::double_t distance_obstacle;
    std::double_t error_margin;
};

template <typename Trait>
class RadarInterface : public Trait::Base
{
  public:
    using Trait::Base::Base;
    typename Trait::template Event<RadarScan> scan{*this, "scan"};
};

using RadarProxy = mw::com::AsProxy<RadarInterface>;
using RadarSkeleton = mw::com::AsSkeleton<RadarInterface>;

struct Scene
{
    Scene() = default;
    Scene(Scene&&) = default;
    Scene(const Scene&) = default;
    Scene& operator=(Scene&&) = default;
    Scene& operator=(const Scene&) = default;

    std::size_t num_people;
    std::size_t num_cars;
    std::double_t distance_obstacle;
    std::double_t distance_left_lane;
    std::double_t distance_right_lane;
};

template <typename Trait>
class NeuralNetInterface : public Trait::Base
{
  public:
    using Trait::Base::Base;
    typename Trait::template Event<Scene> scene{*this, "scene"};
};

using NeuralNetProxy = mw::com::AsProxy<NeuralNetInterface>;
using NeuralNetSkeleton = mw::com::AsSkeleton<NeuralNetInterface>;

struct BrakeInstruction
{
    BrakeInstruction() = default;
    BrakeInstruction(BrakeInstruction&&) = default;
    BrakeInstruction(const BrakeInstruction&) = default;
    BrakeInstruction& operator=(BrakeInstruction&&) = default;
    BrakeInstruction& operator=(const BrakeInstruction&) = default;

    bool active;
    std::double_t level;
};

template <typename Trait>
class BrakeControllerInterface : public Trait::Base
{
  public:
    using Trait::Base::Base;
    typename Trait::template Event<BrakeInstruction> brake_instruction{*this, "brake_instruction"};
};

using BrakeControllerProxy = mw::com::AsProxy<BrakeControllerInterface>;
using BrakeControllerSkeleton = mw::com::AsSkeleton<BrakeControllerInterface>;

struct Steering
{
    Steering() = default;
    Steering(Steering&&) = default;
    Steering(const Steering&) = default;
    Steering& operator=(Steering&&) = default;
    Steering& operator=(const Steering&) = default;

    std::double_t angle;
};

template <typename Trait>
class SteeringControllerInterface : public Trait::Base
{
  public:
    using Trait::Base::Base;
    typename Trait::template Event<Steering> steering{*this, "steering"};
};

using SteeringControllerProxy = mw::com::AsProxy<SteeringControllerInterface>;
using SteeringControllerSkeleton = mw::com::AsSkeleton<SteeringControllerInterface>;

}  // namespace score::feo::com
#endif  // SCORE_MINI_ADAS_DATATYPE_H
