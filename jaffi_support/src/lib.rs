// Copyright 2022 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::{borrow::Cow, ops::Deref};

pub mod arrays;

pub use jni;
use jni::{
    objects::{JObject, JString, JValue},
    strings::JNIString,
    JNIEnv,
};

pub trait JavaPrimitive {}

impl<'j, T> JavaPrimitive for T where T: Deref<Target = JObject<'j>> {}

pub trait FromJavaToRust<'j, J: 'j> {
    fn java_to_rust(java: J, _env: JNIEnv<'j>) -> Self;
}

pub trait FromRustToJava<'j, R> {
    fn rust_to_java(rust: R, _env: JNIEnv<'j>) -> Self;
}

/// Byte
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaByte(pub jni::sys::jbyte);

impl FromJavaToRust<'_, JavaByte> for u8 {
    fn java_to_rust(java: JavaByte, _env: JNIEnv<'_>) -> Self {
        java.0 as u8
    }
}

impl FromRustToJava<'_, u8> for JavaByte {
    fn rust_to_java(rust: u8, _env: JNIEnv<'_>) -> Self {
        JavaByte(rust as jni::sys::jbyte)
    }
}

/// Char
///
/// Chars are generally going to be bad from Rust to Java, always best to just use Strings.
/// jchar is just a u16, which can't encode the same space as Rust...
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaChar(pub jni::sys::jchar);

impl FromJavaToRust<'_, JavaChar> for char {
    fn java_to_rust(java: JavaChar, _env: JNIEnv<'_>) -> Self {
        let ch = java.0 as u32;
        unsafe { char::from_u32_unchecked(ch) }
    }
}

impl FromRustToJava<'_, char> for JavaChar {
    fn rust_to_java(rust: char, _env: JNIEnv<'_>) -> Self {
        JavaChar(rust as u32 as u16)
    }
}

/// Double
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaDouble(pub jni::sys::jdouble);

impl FromJavaToRust<'_, JavaDouble> for f64 {
    fn java_to_rust(java: JavaDouble, _env: JNIEnv<'_>) -> Self {
        java.0
    }
}

impl FromRustToJava<'_, f64> for JavaDouble {
    fn rust_to_java(rust: f64, _env: JNIEnv<'_>) -> Self {
        JavaDouble(rust)
    }
}

/// Float
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaFloat(pub jni::sys::jfloat);

impl FromJavaToRust<'_, JavaFloat> for f32 {
    fn java_to_rust(java: JavaFloat, _env: JNIEnv<'_>) -> Self {
        java.0
    }
}

impl FromRustToJava<'_, f32> for JavaFloat {
    fn rust_to_java(rust: f32, _env: JNIEnv<'_>) -> Self {
        JavaFloat(rust)
    }
}

/// Int
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaInt(pub jni::sys::jint);

impl FromJavaToRust<'_, JavaInt> for i32 {
    fn java_to_rust(java: JavaInt, _env: JNIEnv<'_>) -> Self {
        java.0
    }
}

impl FromRustToJava<'_, i32> for JavaInt {
    fn rust_to_java(rust: i32, _env: JNIEnv<'_>) -> Self {
        JavaInt(rust)
    }
}

/// Long
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaLong(pub jni::sys::jlong);

impl FromJavaToRust<'_, JavaLong> for i64 {
    fn java_to_rust(java: JavaLong, _env: JNIEnv<'_>) -> Self {
        java.0
    }
}

impl FromRustToJava<'_, i64> for JavaLong {
    fn rust_to_java(rust: i64, _env: JNIEnv<'_>) -> Self {
        JavaLong(rust)
    }
}

/// Short
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaShort(pub jni::sys::jshort);

impl FromJavaToRust<'_, JavaShort> for i16 {
    fn java_to_rust(java: JavaShort, _env: JNIEnv<'_>) -> Self {
        java.0
    }
}

impl FromRustToJava<'_, i16> for JavaShort {
    fn rust_to_java(rust: i16, _env: JNIEnv<'_>) -> Self {
        JavaShort(rust)
    }
}

/// Boolean
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaBoolean(pub jni::sys::jboolean);

impl FromJavaToRust<'_, JavaBoolean> for bool {
    fn java_to_rust(java: JavaBoolean, _env: JNIEnv<'_>) -> Self {
        java.0 == jni::sys::JNI_TRUE
    }
}

impl FromRustToJava<'_, bool> for JavaBoolean {
    fn rust_to_java(rust: bool, _env: JNIEnv<'_>) -> Self {
        if rust {
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

impl FromJavaToRust<'_, JavaVoid> for () {
    fn java_to_rust(_java: JavaVoid, _env: JNIEnv<'_>) -> Self {}
}

impl FromRustToJava<'_, ()> for JavaVoid {
    fn rust_to_java(rust: (), _env: JNIEnv<'_>) -> Self {
        JavaVoid(rust)
    }
}

/// Strings
impl<'j, J> FromJavaToRust<'j, J> for String
where
    J: 'j + Deref<Target = JObject<'j>>,
{
    // TODO: there's probably a somewhat cheaper option to reduce all the allocations here.
    fn java_to_rust(java: J, env: JNIEnv<'j>) -> Self {
        // We're going to have Java properly return utf-8 bytes from a String rather than the BS that is the "reduced utf-8" in JNI
        let utf8_arg = env
            .new_string("UTF-8")
            .expect("Java couldn't allocate a simple string");

        // TODO: cache the method_id...
        let byte_array = env
            .call_method(
                *java,
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

impl<'j, S> FromRustToJava<'j, S> for JString<'j>
where
    S: KnownString,
{
    fn rust_to_java(rust: S, env: JNIEnv<'j>) -> Self {
        // There's basically no "cheap" way to do this
        env.new_string(rust).expect("bad string sent to Java")
    }
}

/// Convert from a JValue (return type in Java) into the Rust type
///
/// This is infallible because the generated code using it should "know" that the type is already correct
pub trait FromJavaValue<'j, J>: Sized {
    fn from_jvalue(env: JNIEnv<'j>, jvalue: JValue<'j>) -> Self;
}

impl<'j, T, J> FromJavaValue<'j, J> for T
where
    T: FromJavaToRust<'j, J>,
    J: 'j,
    J: From<JObject<'j>>,
{
    fn from_jvalue(env: JNIEnv<'j>, jvalue: JValue<'j>) -> Self {
        let object = jvalue.l().expect("wrong type conversion");
        Self::java_to_rust(object.into(), env)
    }
}

macro_rules! from_java_value {
    ($jtype: ident, $rtype:ty, $jval_func: ident) => {
        impl<'j> FromJavaValue<'j, $jtype> for $rtype {
            fn from_jvalue(env: JNIEnv<'j>, jvalue: JValue<'j>) -> Self {
                let t = $jtype(jvalue.$jval_func().expect("wrong type conversion"));
                Self::java_to_rust(t, env)
            }
        }
    };
}

from_java_value!(JavaByte, u8, b);
from_java_value!(JavaChar, char, c);
from_java_value!(JavaDouble, f64, d);
from_java_value!(JavaFloat, f32, f);
from_java_value!(JavaInt, i32, i);
from_java_value!(JavaLong, i64, j);
from_java_value!(JavaShort, i16, s);
from_java_value!(JavaVoid, (), v);

/// Convert from Rust type into JValue
pub trait IntoJavaValue<'j, J: 'j> {
    fn into_java_value(self, env: JNIEnv<'j>) -> JValue<'j>;
}

impl<'j, J, R> IntoJavaValue<'j, J> for R
where
    J: 'j,
    R: 'j,
    J: FromRustToJava<'j, R>,
    J: Deref<Target = JObject<'j>>,
{
    fn into_java_value(self, env: JNIEnv<'j>) -> JValue<'j> {
        let java = J::rust_to_java(self, env);
        JValue::Object(*java)
    }
}

macro_rules! into_java_value {
    ($jtype: ident, $rtype:ty) => {
        impl IntoJavaValue<'_, $jtype> for $rtype {
            fn into_java_value(self, env: JNIEnv<'_>) -> JValue<'_> {
                let jval = $jtype::rust_to_java(self, env);
                JValue::from(jval.0)
            }
        }
    };
}

into_java_value!(JavaByte, u8);
into_java_value!(JavaChar, char);
into_java_value!(JavaDouble, f64);
into_java_value!(JavaFloat, f32);
into_java_value!(JavaInt, i32);
into_java_value!(JavaLong, i64);
into_java_value!(JavaShort, i16);
into_java_value!(JavaVoid, ());

macro_rules! java_primitive {
    ($jtype: ty) => {
        impl JavaPrimitive for $jtype {}
    };
}

java_primitive!(JavaByte);
java_primitive!(JavaChar);
java_primitive!(JavaDouble);
java_primitive!(JavaFloat);
java_primitive!(JavaInt);
java_primitive!(JavaLong);
java_primitive!(JavaShort);
java_primitive!(JavaVoid);
