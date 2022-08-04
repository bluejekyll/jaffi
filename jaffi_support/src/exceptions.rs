// Copyright 2022 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::{any::Any, borrow::Cow, fmt, panic::UnwindSafe};

use jni::{
    objects::{JObject, JThrowable},
    strings::JNIString,
    sys::jarray,
    JNIEnv,
};

use crate::NullObject;

/// Catches and potential panics, and then converts them to a RuntimeException in Java.
///
/// * `R` - must implement `Default` in order to allow the (unused) default return value in the case of an exception
pub fn catch_panic_and_throw<F: FnOnce() -> R + UnwindSafe, R: NullObject>(
    env: JNIEnv<'_>,
    f: F,
) -> R {
    let result = std::panic::catch_unwind(f);

    match result {
        Ok(r) => r,
        Err(e) => {
            let msg: Cow<_> = match e {
                _ if e.is::<&'static str>() => {
                    let msg: &'static str = e.downcast_ref::<&str>().expect("failed to downcast");
                    msg.into()
                }
                _ if e.is::<String>() => {
                    let msg: &str = e.downcast_ref::<String>().expect("failed to downcast");
                    msg.into()
                }
                _ => format!("unknown panic: {:?}", e.type_id()).into(),
            };

            let msg = format!("panic: {msg}");
            env.throw_new("java/lang/RuntimeException", msg)
                .expect("failed to throw exception");
            R::null()
        }
    }
}

pub trait Throwable: Sized {
    /// Throw a new exception.
    #[track_caller]
    fn throw<S: Into<JNIString>>(&self, env: JNIEnv<'_>, msg: S) -> Result<(), jni::errors::Error>;

    /// Tests the exception against this type to see if it's a correct exception
    fn catch<'j>(_env: JNIEnv<'j>, exception: JThrowable<'j>) -> Result<Self, JThrowable<'j>>;
}

pub struct AnyThrowable;

impl Throwable for AnyThrowable {
    /// Throw a new exception.
    #[track_caller]
    fn throw<S: Into<JNIString>>(&self, env: JNIEnv<'_>, msg: S) -> Result<(), jni::errors::Error> {
        env.throw_new("java/lang/RuntimeException", msg)
    }

    /// Tests the exception against this type to see if it's a correct exception
    fn catch<'j>(_env: JNIEnv<'j>, _exception: JThrowable<'j>) -> Result<Self, JThrowable<'j>> {
        Ok(Self)
    }
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
    pub fn throw(&self, env: JNIEnv<'_>) -> Result<(), jni::errors::Error> {
        <E as Throwable>::throw(&self.kind, env, &self.msg)
    }
}

/// A type that represents a known Exception type from Java.
pub struct Exception<'j, T: Throwable> {
    env: JNIEnv<'j>,
    exception: JThrowable<'j>,
    throwable: T,
}

impl<'j, T: Throwable + Copy> Exception<'j, T> {
    pub fn exception(&self) -> JThrowable<'j> {
        self.exception
    }

    pub fn throwable(&self) -> T {
        self.throwable
    }
}

impl<'j, T: Throwable> Exception<'j, T> {
    /// Throw a new exception.
    #[track_caller]
    pub fn throw<S: Into<JNIString>>(
        &self,
        env: JNIEnv<'_>,
        msg: S,
    ) -> Result<(), jni::errors::Error> {
        self.throwable.throw(env, msg)
    }

    /// Tests the exception against this type to see if it's a correct exception
    pub fn catch(env: JNIEnv<'j>, exception: JThrowable<'j>) -> Result<Self, JThrowable<'j>> {
        let throwable = T::catch(env, exception)?;

        Ok(Self {
            env,
            exception,
            throwable,
        })
    }
}

impl<'j, T: Throwable> fmt::Display for Exception<'j, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if self.exception.is_null() {
            write!(f, "null exception thrown")?;
            return Ok(());
        }

        let mut exception = self.exception;

        // loop through all causes
        for i in 0usize.. {
            let ex_or_cause = if i == 0 { "exception" } else { "cause" };

            let clazz = crate::get_class_name(self.env, JObject::from(exception).into())
                .map_err(|_| fmt::Error)?;

            let message = crate::call_string_method(&self.env, exception.into(), "getMessage")
                .map_err(|_| fmt::Error)?;

            if let Some(message) = message {
                writeln!(f, "{ex_or_cause}: {clazz}: {}", Cow::from(&message))?;
            } else {
                writeln!(f, "{ex_or_cause}: {clazz}")?;
            };

            let trace = self
                .env
                .call_method(
                    JObject::from(exception),
                    "getStackTrace",
                    "()[Ljava/lang/StackTraceElement;",
                    &[],
                )
                .map_err(|_| fmt::Error)?
                .l()
                .map_err(|_| fmt::Error)?;

            if !trace.is_null() {
                let trace = *trace as jarray;
                let len = self.env.get_array_length(trace).map_err(|_| fmt::Error)?;

                for i in 0..len as usize {
                    let stack_element = self
                        .env
                        .get_object_array_element(trace, i as i32)
                        .map_err(|_| fmt::Error)?;

                    let stack_str = crate::call_string_method(&self.env, stack_element, "toString")
                        .map_err(|_| fmt::Error)?;

                    if let Some(stack_str) = stack_str {
                        writeln!(f, "\t{}", Cow::from(&stack_str))?;
                    }
                }
            }

            // continue the going through the causes
            let cause = self
                .env
                .call_method(
                    JObject::from(exception),
                    "getCause",
                    "()Ljava/lang/Throwable;",
                    &[],
                )
                .map_err(|_| fmt::Error)?;

            exception = cause.l().map(Into::into).map_err(|_| fmt::Error)?;
        }

        Ok(())
    }
}

impl<'j, T: Throwable> fmt::Debug for Exception<'j, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        <Self as fmt::Display>::fmt(self, f)
    }
}
