// *******************************************************************************
// Copyright (c) 2026 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// <https://www.apache.org/licenses/LICENSE-2.0>
//
// SPDX-License-Identifier: Apache-2.0
// *******************************************************************************

use alloc::collections::BTreeSet;
use score_log::fmt::ScoreDebug;
use score_log::fmt::{DebugSet, FormatSpec, ScoreWrite};
use std::collections::HashSet;
use std::io::Write;

pub struct ScoreDebugHashSet<'a, T: ScoreDebug>(pub &'a HashSet<T>);

impl<'a, T: ScoreDebug> ScoreDebug for ScoreDebugHashSet<'a, T> {
    fn fmt(&self, f: &mut dyn ScoreWrite, spec: &FormatSpec) -> Result<(), score_log::fmt::Error> {
        DebugSet::new(f, spec).entries(self.0.iter()).finish()
    }
}

pub struct ScoreDebugBTreeSet<'a, T: ScoreDebug>(pub &'a BTreeSet<T>);

impl<'a, T: ScoreDebug> ScoreDebug for ScoreDebugBTreeSet<'a, T> {
    fn fmt(&self, f: &mut dyn ScoreWrite, spec: &FormatSpec) -> Result<(), score_log::fmt::Error> {
        DebugSet::new(f, spec).entries(self.0.iter()).finish()
    }
}

pub struct ScoreDebugDebug<'a, T: core::fmt::Debug, const MAX_LENGTH: usize>(pub &'a T);

impl<'a, T: core::fmt::Debug, const MAX_LENGTH: usize> ScoreDebug for ScoreDebugDebug<'a, T, MAX_LENGTH> {
    fn fmt(&self, f: &mut dyn ScoreWrite, spec: &FormatSpec) -> Result<(), score_log::fmt::Error> {
        let buf = &mut [0u8; { MAX_LENGTH }];
        let debug_str = {
            let mut writer = std::io::Cursor::new(&mut buf[..]);
            write!(&mut writer, "{:?}", self.0).expect("failed to write Debug");
            let len = writer.position() as usize;
            &buf[0..len]
        };
        f.write_str(core::str::from_utf8(debug_str).expect("not a valid UTF-8 string"), spec)
    }
}
