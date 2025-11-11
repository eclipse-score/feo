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

use crate::signalling::relayed::connectors::relays::{SecondaryReceiveRelay, SecondarySendRelay};
use crate::signalling::relayed::interface::IsChannel;
use crate::signalling::relayed::ConnectSecondary;
use feo_log::debug;

pub(crate) struct SecondaryConnector<Inter: IsChannel, Intra: IsChannel> {
    local_to_ipc_relay: SecondarySendRelay<Inter, Intra>,
    ipc_to_local_relay: SecondaryReceiveRelay<Inter, Intra>,
}

impl<Inter: IsChannel, Intra: IsChannel> SecondaryConnector<Inter, Intra> {
    pub fn with_fields(
        local_to_ipc_relay: SecondarySendRelay<Inter, Intra>,
        ipc_to_local_relay: SecondaryReceiveRelay<Inter, Intra>,
    ) -> Self {
        Self {
            local_to_ipc_relay,
            ipc_to_local_relay,
        }
    }
    pub fn run_and_connect(&mut self) {
        debug!("Starting MixedSecondaryConnector");
        self.ipc_to_local_relay.run_and_connect();
        self.local_to_ipc_relay
            .connect()
            .expect("failed to connect relay");
        if let Err(e) = self.local_to_ipc_relay.run() {
            debug!("[SecondaryConnector] Send relay exited with: {:?}", e);
        }
    }
}

impl<Inter: IsChannel, Intra: IsChannel> ConnectSecondary for SecondaryConnector<Inter, Intra> {
    fn run_and_connect(&mut self) {
        self.run_and_connect();
    }
}
