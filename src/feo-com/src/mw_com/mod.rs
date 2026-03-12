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

use crate::interface::{
    ActivityInput, ActivityOutput, ActivityOutputDefault, DebugWrapper, Error, FeoComData, FeoComDefault, InputGuard,
    OutputGuard, OutputUninitGuard,
};
use com_api::{
    LolaRuntimeImpl, PlacementDefault, Publisher, Runtime, SampleContainer, SampleMaybeUninit, SampleMut, Subscriber,
    Subscription,
};
use core::cell::RefCell;
use core::cell::UnsafeCell;
use core::fmt::Debug;
use core::mem::MaybeUninit;
use core::ops::Deref;
use core::ops::DerefMut;

type MwComSubscription<T> =
    <<LolaRuntimeImpl as Runtime>::Subscriber<T> as Subscriber<T, LolaRuntimeImpl>>::Subscription;

type MwComSample<'a, T> = <MwComSubscription<T> as Subscription<T, LolaRuntimeImpl>>::Sample<'a>;

type MwComPublisher<T> = <LolaRuntimeImpl as Runtime>::Publisher<T>;

type MwComSampleMaybeUninit<'a, T> = <MwComPublisher<T> as Publisher<T, LolaRuntimeImpl>>::SampleMaybeUninit<'a>;

type MwComSampleMut<'a, T> = <MwComSampleMaybeUninit<'a, T> as SampleMaybeUninit<T>>::SampleMut;

pub struct MwComInputGuard<'a, T: FeoComData>(MwComSample<'a, T>);

impl<T: FeoComData> Deref for MwComInputGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        Deref::deref(&self.0)
    }
}

pub struct MwComOutputGuard<'a, T: FeoComData>(MwComSampleMut<'a, T>);

impl<'a, T> MwComOutputGuard<'a, T>
where
    T: FeoComData,
{
    pub(crate) fn send(self) -> Result<(), Error> {
        self.0.send()?;
        Ok(())
    }
}

impl<T: FeoComData> Deref for MwComOutputGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: FeoComData> DerefMut for MwComOutputGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

pub struct MwComOutputUninitGuard<'a, T: FeoComData>(
    UnsafeCell<<MwComPublisher<T> as Publisher<T, LolaRuntimeImpl>>::SampleMaybeUninit<'a>>,
);

impl<'a, T> MwComOutputUninitGuard<'a, T>
where
    T: FeoComData,
{
    // Value is initialized when allocated
    pub(crate) unsafe fn assume_init(self) -> MwComOutputGuard<'a, T> {
        MwComOutputGuard(self.0.into_inner().assume_init())
    }

    // Overwrites with given value
    pub(crate) fn write_payload(self, value: T) -> MwComOutputGuard<'a, T> {
        MwComOutputGuard(self.0.into_inner().write(value))
    }
}

impl<'a, T> MwComOutputUninitGuard<'a, T>
where
    T: FeoComData + PlacementDefault + Default + 'a,
{
    // Overwrites with [Default::default]
    pub(crate) fn init(self) -> MwComOutputGuard<'a, T> {
        MwComOutputGuard(self.0.into_inner().write_default())
    }
}

impl<T: FeoComData> Deref for MwComOutputUninitGuard<'_, T> {
    type Target = MaybeUninit<T>;

    fn deref(&self) -> &Self::Target {
        // SAFETY: not safe, needs support from MW COM API to be fixed
        AsMut::as_mut(unsafe { &mut *self.0.get() })
    }
}

impl<T: FeoComData> DerefMut for MwComOutputUninitGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.get_mut().as_mut()
    }
}

#[derive(Debug)]
pub struct MwComInput<T: FeoComData> {
    subscription: MwComSubscription<T>,
    sample_container: DebugWrapper<RefCell<SampleContainer<MwComSample<'static, T>>>>,
}

impl<T: FeoComData + 'static> MwComInput<T> {
    pub fn new(
        subscription: MwComSubscription<T>,
        sample_container: DebugWrapper<RefCell<SampleContainer<MwComSample<'static, T>>>>,
    ) -> Self {
        Self {
            subscription,
            sample_container,
        }
    }
}

impl<T> ActivityInput<T> for MwComInput<T>
where
    T: FeoComData + 'static,
{
    fn read(&self) -> Result<InputGuard<'_, T>, Error> {
        assert_eq!(
            1,
            self.subscription
                .try_receive(&mut self.sample_container.borrow_mut(), 1,)
                .expect("receive failed"),
            "mw com read failed"
        );
        let result = self
            .sample_container
            .borrow_mut()
            .pop_front()
            .expect("pop_front failed");
        Ok(InputGuard::MwCom(MwComInputGuard(result)))
    }
}

#[derive(Debug)]
pub struct MwComOutput<T: FeoComData>(DebugWrapper<MwComPublisher<T>>);

impl<T: FeoComData + 'static> MwComOutput<T> {
    pub fn new(publisher: DebugWrapper<MwComPublisher<T>>) -> Self {
        Self(publisher)
    }
}

impl<T> ActivityOutput<T> for MwComOutput<T>
where
    T: FeoComData + 'static,
{
    // Initialized when allocated
    fn write_uninit(&mut self) -> Result<OutputUninitGuard<'_, T>, Error> {
        Ok(OutputUninitGuard::MwCom(MwComOutputUninitGuard(UnsafeCell::new(
            self.0.allocate()?,
        ))))
    }
}

impl<T> ActivityOutputDefault<T> for MwComOutput<T>
where
    T: FeoComData + FeoComDefault + 'static,
{
    // Overwrites with [Default::default]
    fn write_init(&mut self) -> Result<OutputGuard<'_, T>, Error> {
        Ok(OutputGuard::MwCom(MwComOutputGuard(self.0.allocate()?.write_default())))
    }
}
