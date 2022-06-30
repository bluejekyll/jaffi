// Copyright 2022 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

pub use jni;

pub trait FromJavaToRust {
    type Rust;

    fn java_to_rust(self) -> Self::Rust;

    fn rust_to_java(rust: Self::Rust) -> Self;
}

trait FromRustToJava {}

/// Byte
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaByte(pub jni::sys::jbyte);

impl FromJavaToRust for JavaByte {
    type Rust = u8;

    fn java_to_rust(self) -> Self::Rust {
        self.0 as u8
    }

    fn rust_to_java(rust: Self::Rust) -> Self {
        Self(rust as jni::sys::jbyte)
    }
}

/// Char
///
/// Chars are generally going to be bad from Rust to Java, always best to just use Strings.
/// jchar is just a u16, which can't encode the same space as Rust...
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaChar(pub jni::sys::jchar);

impl FromJavaToRust for JavaChar {
    type Rust = char;

    fn java_to_rust(self) -> Self::Rust {
        let ch = self.0 as u32;
        unsafe { char::from_u32_unchecked(ch) }
    }

    fn rust_to_java(rust: Self::Rust) -> Self {
        Self(rust as u32 as u16)
    }
}

/// Double
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaDouble(pub jni::sys::jdouble);

impl FromJavaToRust for JavaDouble {
    type Rust = f64;

    fn java_to_rust(self) -> Self::Rust {
        self.0
    }

    fn rust_to_java(rust: Self::Rust) -> Self {
        Self(rust)
    }
}

/// Float
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaFloat(pub jni::sys::jfloat);

impl FromJavaToRust for JavaFloat {
    type Rust = f32;

    fn java_to_rust(self) -> Self::Rust {
        self.0
    }

    fn rust_to_java(rust: Self::Rust) -> Self {
        Self(rust)
    }
}

/// Int
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaInt(pub jni::sys::jint);

impl FromJavaToRust for JavaInt {
    type Rust = i32;

    fn java_to_rust(self) -> Self::Rust {
        self.0
    }

    fn rust_to_java(rust: Self::Rust) -> Self {
        Self(rust)
    }
}

/// Long
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaLong(pub jni::sys::jlong);

impl FromJavaToRust for JavaLong {
    type Rust = i64;

    fn java_to_rust(self) -> Self::Rust {
        self.0
    }

    fn rust_to_java(rust: Self::Rust) -> Self {
        Self(rust)
    }
}

/// Short
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaShort(pub jni::sys::jshort);

impl FromJavaToRust for JavaShort {
    type Rust = i16;

    fn java_to_rust(self) -> Self::Rust {
        self.0
    }

    fn rust_to_java(rust: Self::Rust) -> Self {
        Self(rust)
    }
}

/// Boolean
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaBoolean(pub jni::sys::jboolean);

impl FromJavaToRust for JavaBoolean {
    type Rust = bool;

    fn java_to_rust(self) -> Self::Rust {
        self.0 == jni::sys::JNI_TRUE
    }

    fn rust_to_java(rust: Self::Rust) -> Self {
        if rust {
            Self(jni::sys::JNI_TRUE)
        } else {
            Self(jni::sys::JNI_FALSE)
        }
    }
}

/// Void
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct JavaVoid(());

impl FromJavaToRust for JavaVoid {
    type Rust = ();

    fn java_to_rust(self) -> Self::Rust {}

    fn rust_to_java(_rust: Self::Rust) -> Self {
        Self(())
    }
}
