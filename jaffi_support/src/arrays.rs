// Copyright 2022 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use jni::objects::{AutoArray, JByteBuffer};

use super::*;

/// Arrays
///
/// If greater than 1 dimension of
///
/// # Type Parameters
///
/// * `N` - The number of dimensions in the array
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct UnsupportedArray<'j>(pub JObject<'j>);

/// Arrays
///
/// If greater than 1 dimension of
///
/// # Type Parameters
///
/// * `N` - The number of dimensions in the array
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaByteArray<'j>(JObject<'j>);

impl<'j> JavaByteArray<'j> {
    /// Creates a new array from containing the data from `from`
    pub fn new(env: JNIEnv<'j>, from: &[u8]) -> Result<Self, jni::errors::Error> {
        env.byte_array_from_slice(from)
            .map(|jarray| Self(jarray.into()))
    }

    /// A read-only wrapper around the java array
    pub fn as_slice<'s>(
        &'s self,
        env: &'s JNIEnv<'j>,
    ) -> Result<JavaByteArrayRef<'s, 'j>, jni::errors::Error> {
        env.get_byte_array_elements(*self.0, jni::objects::ReleaseMode::NoCopyBack)
            .map(JavaByteArrayRef)
    }
}

/// Rather than implementing any conversions, the ByteArrays allow present low level options to make the best decision for performance
impl<'j> FromJavaToRust<'j, Self> for JavaByteArray<'j> {
    fn java_to_rust(java: Self, _env: JNIEnv<'j>) -> Self {
        java
    }
}

/// Rather than implementing any conversions, the ByteArrays allow present low level options to make the best decision for performance
impl<'j> FromRustToJava<'j, Self> for JavaByteArray<'j> {
    fn rust_to_java(rust: Self, _env: JNIEnv<'j>) -> Self {
        rust
    }
}

impl<'j> From<JObject<'j>> for JavaByteArray<'j> {
    fn from(jobject: JObject<'j>) -> Self {
        Self(jobject)
    }
}

impl<'j> From<JavaByteArray<'j>> for JObject<'j> {
    fn from(jarray: JavaByteArray<'j>) -> Self {
        jarray.0
    }
}

impl<'j> Deref for JavaByteArray<'j> {
    type Target = JObject<'j>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct JavaByteArrayRef<'s: 'j, 'j>(AutoArray<'s, 'j, jni::sys::jbyte>);

impl<'s: 'j, 'j> Deref for JavaByteArrayRef<'s, 'j> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        let len = self.0.size().expect("len not available on array") as usize;
        let data = self.0.as_ptr() as *const u8;

        unsafe { std::slice::from_raw_parts(data, len) }
    }
}

// ByteBuffer support

/// Rather than implementing any conversions, the ByteArrays allow present low level options to make the best decision for performance
impl<'j> FromJavaToRust<'j, Self> for JByteBuffer<'j> {
    fn java_to_rust(java: Self, _env: JNIEnv<'j>) -> Self {
        java
    }
}

/// Rather than implementing any conversions, the ByteArrays allow present low level options to make the best decision for performance
impl<'j> FromRustToJava<'j, Self> for JByteBuffer<'j> {
    fn rust_to_java(rust: Self, _env: JNIEnv<'j>) -> Self {
        rust
    }
}
