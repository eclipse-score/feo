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

use feo::recording::recorder::{DataDescriptionRecord, Record};
use mini_adas::activities::messages;
use score_log::{info, LevelFilter};
use serde::Deserialize;
use std::io::Read;
use stdout_logger::StdoutLoggerBuilder;

fn main() {
    StdoutLoggerBuilder::new()
        .context("adas-deserializer")
        .show_module(true)
        .show_file(true)
        .show_line(true)
        .log_level(LevelFilter::Trace)
        .set_as_default_logger();

    let mut serialized_data = Vec::new();
    std::fs::File::open("rec.bin")
        .expect("failed to open recording")
        // Reading to end for now, just for this simple tool
        .read_to_end(&mut serialized_data)
        .expect("failed to read recording");

    let serialized_data_len = serialized_data.len();
    info!("Read file with {} bytes", serialized_data_len);
    let mut remaining_bytes = serialized_data.as_slice();
    while !remaining_bytes.is_empty() {
        let (record, remaining) = postcard::take_from_bytes(remaining_bytes).expect("deserializing failed");
        remaining_bytes = remaining;

        println!("{record:#?}");
        if let Record::DataDescription(data_record) = record {
            if let Some((image, remaining)) =
                try_deserialization_as_a::<messages::CameraImage>(data_record, remaining_bytes)
            {
                remaining_bytes = remaining;
                println!("{:#?}", image);
            } else if let Some((radar, remaining)) =
                try_deserialization_as_a::<messages::RadarScan>(data_record, remaining_bytes)
            {
                remaining_bytes = remaining;
                println!("{:#?}", radar);
            } else if let Some((scene, remaining)) =
                try_deserialization_as_a::<messages::Scene>(data_record, remaining_bytes)
            {
                remaining_bytes = remaining;
                println!("{:#?}", scene);
            } else if let Some((brake, remaining)) =
                try_deserialization_as_a::<messages::BrakeInstruction>(data_record, remaining_bytes)
            {
                remaining_bytes = remaining;
                println!("{:#?}", brake);
            } else if let Some((steering, remaining)) =
                try_deserialization_as_a::<messages::Steering>(data_record, remaining_bytes)
            {
                remaining_bytes = remaining;
                println!("{:#?}", steering);
            } else {
                // Skip data record
                info!("Skipping deserialization of {}", data_record.type_name);
                remaining_bytes = &remaining_bytes[data_record.data_size..];
            }
        }
    }
}

fn try_deserialization_as_a<'a, T: Deserialize<'a>>(
    header: DataDescriptionRecord,
    bytes: &'a [u8],
) -> Option<(T, &'a [u8])> {
    if header.type_name == std::any::type_name::<T>() {
        Some(postcard::take_from_bytes(bytes).expect("failed to deserialize CameraImage"))
    } else {
        None
    }
}
