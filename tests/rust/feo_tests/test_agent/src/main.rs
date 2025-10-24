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
use clap::{Parser, Subcommand, ValueEnum};
use feo::error::Error;
use feo_log::LevelFilter;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Once;

use crate::primary::PrimaryLauncher as _;
use crate::secondary::SecondaryLauncher as _;

mod activities;
mod config;
mod monitor;
mod primary;
mod scenario;
mod secondary;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    com_config: Option<PathBuf>,
    #[arg(value_enum)]
    signalling: Signalling,
    #[arg(value_enum)]
    scenario: Scenario,
    #[command(subcommand)]
    role: Role,
}

#[derive(Subcommand)]
enum Role {
    Primary { monitor_server: String },
    Secondary { num: usize },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Signalling {
    DirectMpsc,
    DirectTcp,
    DirectUnix,
    RelayedTcp,
    RelayedUnix,
}

#[derive(ValueEnum, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Scenario {
    SingleAgent,
    MultipleAgents,
}

fn init(_com_config: Option<&Path>) {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        feo_logger::init(LevelFilter::Debug, true, true);
        feo_tracing::init(feo_tracing::LevelFilter::TRACE);
    });
}

pub fn main() {
    let cli = Cli::parse();
    init(cli.com_config.as_deref());
    run_scenario(cli).unwrap()
}

fn run_scenario(cli: Cli) -> Result<(), Error> {
    match cli.role {
        Role::Primary { monitor_server } => {
            cli.signalling.launch_primary(cli.scenario, monitor_server)
        }
        Role::Secondary { num } => cli.signalling.launch_secondary(cli.scenario, num),
    }
}
