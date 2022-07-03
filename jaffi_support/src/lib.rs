// Copyright 2022 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::{borrow::Cow, ffi::CStr, ops::Deref};

pub use jni;
use jni::{
    objects::{JString, JValue, ReleaseMode},
    strings::JNIString,
    JNIEnv,
};

pub trait FromJavaToRust<'j> {
    type Rust: 'j;

    fn java_to_rust(self, _env: JNIEnv<'j>) -> Self::Rust;
}

pub trait FromRustToJava<'j> {
    type Java: 'j;

    fn rust_to_java(self, _env: JNIEnv<'j>) -> Self::Java;
}

/// Byte
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaByte(pub jni::sys::jbyte);

impl FromJavaToRust<'_> for JavaByte {
    type Rust = u8;

    fn java_to_rust(self, _env: JNIEnv<'_>) -> Self::Rust {
        self.0 as u8
    }
}

impl FromRustToJava<'_> for u8 {
    type Java = JavaByte;

    fn rust_to_java(self, _env: JNIEnv<'_>) -> Self::Java {
        JavaByte(self as jni::sys::jbyte)
    }
}

/// Char
///
/// Chars are generally going to be bad from Rust to Java, always best to just use Strings.
/// jchar is just a u16, which can't encode the same space as Rust...
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaChar(pub jni::sys::jchar);

impl FromJavaToRust<'_> for JavaChar {
    type Rust = char;

    fn java_to_rust(self, _env: JNIEnv<'_>) -> Self::Rust {
        let ch = self.0 as u32;
        unsafe { char::from_u32_unchecked(ch) }
    }
}

impl FromRustToJava<'_> for char {
    type Java = JavaChar;

    fn rust_to_java(self, _env: JNIEnv<'_>) -> Self::Java {
        JavaChar(self as u32 as u16)
    }
}

/// Double
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaDouble(pub jni::sys::jdouble);

impl FromJavaToRust<'_> for JavaDouble {
    type Rust = f64;

    fn java_to_rust(self, _env: JNIEnv<'_>) -> Self::Rust {
        self.0
    }
}

impl FromRustToJava<'_> for f64 {
    type Java = JavaDouble;

    fn rust_to_java(self, _env: JNIEnv<'_>) -> Self::Java {
        JavaDouble(self)
    }
}

/// Float
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaFloat(pub jni::sys::jfloat);

impl FromJavaToRust<'_> for JavaFloat {
    type Rust = f32;

    fn java_to_rust(self, _env: JNIEnv<'_>) -> Self::Rust {
        self.0
    }
}

impl FromRustToJava<'_> for f32 {
    type Java = JavaFloat;

    fn rust_to_java(self, _env: JNIEnv<'_>) -> Self::Java {
        JavaFloat(self)
    }
}

/// Int
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaInt(pub jni::sys::jint);

impl FromJavaToRust<'_> for JavaInt {
    type Rust = i32;

    fn java_to_rust(self, _env: JNIEnv<'_>) -> Self::Rust {
        self.0
    }
}

impl FromRustToJava<'_> for i32 {
    type Java = JavaInt;

    fn rust_to_java(self, _env: JNIEnv<'_>) -> Self::Java {
        JavaInt(self)
    }
}

/// Long
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaLong(pub jni::sys::jlong);

impl FromJavaToRust<'_> for JavaLong {
    type Rust = i64;

    fn java_to_rust(self, _env: JNIEnv<'_>) -> Self::Rust {
        self.0
    }
}

impl FromRustToJava<'_> for i64 {
    type Java = JavaLong;

    fn rust_to_java(self, _env: JNIEnv<'_>) -> Self::Java {
        JavaLong(self)
    }
}

/// Short
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaShort(pub jni::sys::jshort);

impl FromJavaToRust<'_> for JavaShort {
    type Rust = i16;

    fn java_to_rust(self, _env: JNIEnv<'_>) -> Self::Rust {
        self.0
    }
}

impl FromRustToJava<'_> for i16 {
    type Java = JavaShort;

    fn rust_to_java(self, _env: JNIEnv<'_>) -> Self::Java {
        JavaShort(self)
    }
}

/// Boolean
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaBoolean(pub jni::sys::jboolean);

impl FromJavaToRust<'_> for JavaBoolean {
    type Rust = bool;

    fn java_to_rust(self, _env: JNIEnv<'_>) -> Self::Rust {
        self.0 == jni::sys::JNI_TRUE
    }
}

impl FromRustToJava<'_> for bool {
    type Java = JavaBoolean;

    fn rust_to_java(self, _env: JNIEnv<'_>) -> Self::Java {
        if self {
            JavaBoolean(jni::sys::JNI_TRUE)
        } else {
            JavaBoolean(jni::sys::JNI_FALSE)
        }
    }
}

/// Void
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaVoid(());

impl FromJavaToRust<'_> for JavaVoid {
    type Rust = ();

    fn java_to_rust(self, env: JNIEnv<'_>) -> Self::Rust {}
}

impl FromRustToJava<'_> for () {
    type Java = JavaVoid;

    fn rust_to_java(self, _env: JNIEnv<'_>) -> Self::Java {
        JavaVoid(())
    }
}

/// Strings
impl<'j> FromJavaToRust<'j> for JString<'j> {
    type Rust = String;

    // TODO: there's probably a somewhat cheaper option to reduce all the allocations here.
    fn java_to_rust(self, env: JNIEnv<'j>) -> Self::Rust {
        // We're going to have Java properly return utf-8 bytes from a String rather than the BS that is the "reduced utf-8" in JNI
        let utf8_arg = env
            .new_string("UTF-8")
            .expect("Java couldn't allocate a simple string");

        // TODO: cache the method_id...
        let byte_array = env
            .call_method(
                self,
                "getBytes",
                "(Ljava/lang/String;)[B",
                &[JValue::Object(utf8_arg.into())],
            )
            .expect("couldn't call a standard method in Java");
        let byte_array = byte_array
            .l()
            .expect("should have been a JObject of a byte array");

        let bytes = env
            .convert_byte_array(*byte_array)
            .expect("the byte_array from previous call was bad");

        // Java should really not have returned bad UTF-8
        unsafe { String::from_utf8_unchecked(bytes) }
    }
}

trait KnownString: Into<JNIString> {}

impl KnownString for String {}
impl KnownString for &'_ str {}
impl KnownString for Cow<'_, str> {}
impl KnownString for Box<str> {}

impl<'j, S> FromRustToJava<'j> for S
where
    S: KnownString,
{
    type Java = JString<'j>;

    fn rust_to_java(self, env: JNIEnv<'j>) -> Self::Java {
        // There's basically no "cheap" way to do this
        env.new_string(self).expect("bad string sent to Java")
    }
}
