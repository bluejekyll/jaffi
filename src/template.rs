// Copyright 2022 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::borrow::Cow;

use serde::Serialize;

use tinytemplate::TinyTemplate;

use crate::Error;

pub(crate) static RUST_FFI: &str = "RUST_FFI";

/// Template for the generated rust files.

static RUST_FFI_TEMPLATE: &str = r#"
use jni::objects::\{JByteBuffer, JClass};
use jni::sys::jlong;
use jni::JNIEnv;

{{ for function in functions }}
/// JNI method signature { function.signature }
#[no_mangle]
pub extern "system" fn { function.name }<'j>(
    env: JNIEnv<'j>,
    {{ for arg in function.args }}
    arg.name: arg.type,
    {{ endfor }}
) \{

}
{{ endfor }}
"#;

#[derive(Serialize)]
pub(crate) struct RustFfi<'a> {
    pub(crate) class_name: Cow<'a, str>,
    pub(crate) functions: Vec<Function>,
}

#[derive(Serialize)]
pub(crate) struct Function {
    pub(crate) signature: String,
}

#[derive(Serialize)]
pub(crate) struct Arg<'a> {
    pub(crate) name: Cow<'a, str>,
    pub(crate) ty: JniType,
}

pub(crate) fn new_engine() -> Result<TinyTemplate<'static>, Error> {
    let mut tt = TinyTemplate::new();
    tt.add_template(RUST_FFI, RUST_FFI_TEMPLATE)?;
    Ok(tt)
}

#[derive(Clone, Debug)]
pub(crate) enum JniType<'a> {
    ///Byte
    Jbyte,
    /// Char
    Jchar,
    /// Double
    Jdouble,
    /// Float
    Jfloat,
    /// Int
    Jint,
    /// Long
    Jlong,
    /// Short
    Jshort,
    /// Boolean
    Jboolean,
    /// Object
    Jobject(Cow<'a, str>),
}

impl<'a> JniType<'a> {
    impl fn to_jni(&self) -> Cow<'a, str> {
        match self {
    Jbyte => "jaffi_support::jni::",
    Jchar ,
    Jdouble,
    Jfloat,
    Jint,
    Jlong,
    Jshort,
    Jboolean,
    Jobject(Cow<'a, str>),
        }
    }
}