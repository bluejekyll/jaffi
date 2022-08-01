// Copyright 2022 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::{borrow::Cow, marker::PhantomData};

use jni::{strings::JNIString, JNIEnv};

pub trait Throwable: 'static {
    #[track_caller]
    fn throw<'j, S: Into<JNIString>>(
        &self,
        env: JNIEnv<'j>,
        msg: S,
    ) -> Result<(), jni::errors::Error>;
}

pub struct Error<E: Throwable> {
    kind: E,
    msg: Cow<'static, str>,
}

impl<E: Throwable> Error<E> {
    pub fn new<S: Into<Cow<'static, str>>>(kind: E, msg: S) -> Self {
        let msg = msg.into();
        Self { kind, msg }
    }

    #[track_caller]
    pub fn throw<'j>(&self, env: JNIEnv<'j>) -> Result<(), jni::errors::Error> {
        <E as Throwable>::throw(&self.kind, env, &self.msg)
    }
}
