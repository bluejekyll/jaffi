// Copyright 2022 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::{
    borrow::Cow,
    fmt::{self, Write},
};

use cafebabe::descriptor::{BaseType, FieldType, ReturnDescriptor, Ty};
use jaffi_support::jni::{
    objects::{JByteBuffer, JClass, JObject, JString, JThrowable},
    sys,
};
use serde::Serialize;

use tinytemplate::TinyTemplate;

use crate::Error;

pub(crate) static RUST_FFI: &str = "RUST_FFI";
pub(crate) static RUST_FFI_OBJ: &str = "RUST_FFI_OBJ";

/// Template for the generated rust files.

static RUST_FFI_TEMPLATE: &str = r#"
use jaffi_support::jni::\{
    objects::\{JByteBuffer, JClass},
    sys::jlong,
    JNIEnv
};

{{ for function in functions }}
/// JNI method signature { function.signature }
#[no_mangle]
pub extern "system" fn { function.name -}<'j>(
    env: JNIEnv<'j>,
    this: { function.class_or_this -},
    {{ for arg in function.arguments }}
    { arg.name }: { arg.ty },
    {{ endfor }}
) -> { function.result } \{

}
{{ endfor }}
"#;

static RUST_FFI_OBJ_TEMPLATE: &str = r#"
use std::ops::Deref;

use jaffi_support::jni::objects::\{JClass, JObject};

{{ for obj in objects }}
#[repr(transparent)]
pub struct { obj.class_name -}(JClass<'j>);

impl<'j> std::ops::Deref for { obj.class_name -} \{
    type Target = JClass<'j>;

    fn deref(&self) -> &Self::Target \{
        &self.0
    }
}

#[repr(transparent)]
pub struct { obj.obj_name -}(JObject);

impl<'j> std::ops::Deref for { obj.obj_name -} \{
    type Target = JObject<'j>;

    fn deref(&self) -> &Self::Target \{
        &self.0
    }
}
{{ endfor }}
"#;

pub(crate) fn new_engine() -> Result<TinyTemplate<'static>, Error> {
    let mut tt = TinyTemplate::new();
    tt.add_template(RUST_FFI, RUST_FFI_TEMPLATE)?;
    tt.add_template(RUST_FFI_OBJ, RUST_FFI_OBJ_TEMPLATE)?;
    tt.set_default_formatter(&tinytemplate::format_unescaped);
    Ok(tt)
}

#[derive(Serialize)]
pub(crate) struct RustFfi<'a> {
    pub(crate) class_name: Cow<'a, str>,
    pub(crate) functions: Vec<Function>,
}

#[derive(Serialize)]
pub(crate) struct Function {
    pub(crate) name: String,
    pub(crate) signature: String,
    pub(crate) class_or_this: String,
    pub(crate) arguments: Vec<Arg>,
    pub(crate) result: String,
}

#[derive(Serialize)]
pub(crate) struct Arg {
    pub(crate) name: String,
    pub(crate) ty: String,
}

#[derive(Serialize)]
pub(crate) struct RustFfiObjects {
    pub(crate) objects: Vec<Object>,
}

#[derive(Serialize)]
pub(crate) struct Object {
    pub(crate) name: String,
    pub(crate) class_name: String,
    pub(crate) obj_name: String,
}

impl From<ObjectType> for Object {
    fn from(ty: ObjectType) -> Self {
        let name = ty.as_descriptor().to_string();
        let class_name = ty.to_jni_class_name();
        let obj_name = ty.to_jni_type_name().to_string();

        Object {
            name,
            class_name,
            obj_name,
        }
    }
}

#[derive(Serialize)]
pub(crate) enum Return {
    Void,
    Val(JniType),
}

impl Return {
    pub(crate) fn from_java(field_type: &ReturnDescriptor) -> Self {
        match field_type {
            ReturnDescriptor::Void => Self::Void,
            ReturnDescriptor::Return(val) => Self::Val(JniType::from_java(val)),
        }
    }

    pub(crate) fn to_jni_type_name(&self) -> String {
        match self {
            Self::Void => "()".to_string(),
            Self::Val(ty) => ty.to_jni_type_name(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) enum BaseJniTy {
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
    Jobject(ObjectType),
}

#[derive(Clone, Debug, Serialize)]
pub(crate) enum JniType {
    /// Non recursive types
    Ty(BaseJniTy),
    /// Array,
    Jarray { dimensions: usize, ty: BaseJniTy },
}

impl JniType {
    /// Outputs the form needed in jni function interfaces
    pub(crate) fn to_jni_type_name(&self) -> String {
        match self {
            Self::Ty(BaseJniTy::Jbyte) => std::any::type_name::<sys::jbyte>().into(),
            Self::Ty(BaseJniTy::Jchar) => std::any::type_name::<sys::jchar>().into(),
            Self::Ty(BaseJniTy::Jdouble) => std::any::type_name::<sys::jdouble>().into(),
            Self::Ty(BaseJniTy::Jfloat) => std::any::type_name::<sys::jfloat>().into(),
            Self::Ty(BaseJniTy::Jint) => std::any::type_name::<sys::jint>().into(),
            Self::Ty(BaseJniTy::Jlong) => std::any::type_name::<sys::jlong>().into(),
            Self::Ty(BaseJniTy::Jshort) => std::any::type_name::<sys::jshort>().into(),
            Self::Ty(BaseJniTy::Jboolean) => std::any::type_name::<sys::jboolean>().into(),
            Self::Ty(BaseJniTy::Jobject(obj)) => obj.to_jni_type_name().into(),
            // in JNI the array is always jarray
            Self::Jarray { .. } => std::any::type_name::<sys::jarray>().into(),
        }
    }

    /// Takes the types from the class file and converts to Self.
    pub(crate) fn from_java(field_type: &FieldType<'_>) -> Self {
        fn base_jni_ty_from_java(ty: &Ty<'_>) -> BaseJniTy {
            match ty {
                Ty::Base(BaseType::Byte) => BaseJniTy::Jbyte,
                Ty::Base(BaseType::Char) => BaseJniTy::Jchar,
                Ty::Base(BaseType::Double) => BaseJniTy::Jdouble,
                Ty::Base(BaseType::Float) => BaseJniTy::Jfloat,
                Ty::Base(BaseType::Int) => BaseJniTy::Jint,
                Ty::Base(BaseType::Long) => BaseJniTy::Jlong,
                Ty::Base(BaseType::Short) => BaseJniTy::Jshort,
                Ty::Base(BaseType::Boolean) => BaseJniTy::Jboolean,
                Ty::Object(obj) => BaseJniTy::Jobject(ObjectType::from(obj)),
            }
        }

        match field_type {
            FieldType::Ty(ty) => Self::Ty(base_jni_ty_from_java(ty)),
            FieldType::Array { dimensions, ty } => Self::Jarray {
                dimensions: *dimensions,
                ty: base_jni_ty_from_java(ty),
            },
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize)]
pub(crate) enum ObjectType {
    JClass,
    JByteBuffer,
    JObject,
    JString,
    JThrowable,
    Object(String),
}

impl ObjectType {
    pub(crate) fn as_descriptor(&self) -> &str {
        match self {
            Self::JClass => "java/lang/Class",
            Self::JByteBuffer => "java/nio/ByteBuffer",
            Self::JObject => "java/lang/Object",
            Self::JString => "java/lang/String",
            Self::JThrowable => "java/lang/Throwable",
            Self::Object(desc) => desc,
        }
    }

    fn to_type_name_base(&self) -> String {
        match *self {
            Self::JClass => std::any::type_name::<JClass<'_>>().into(),
            Self::JByteBuffer => std::any::type_name::<JByteBuffer<'_>>().into(),
            Self::JObject => std::any::type_name::<JObject<'_>>().into(),
            Self::JString => std::any::type_name::<JString<'_>>().into(),
            Self::JThrowable => std::any::type_name::<JThrowable<'_>>().into(),
            Self::Object(ref obj) => obj.replace('/', "_"),
        }
    }

    pub(crate) fn to_jni_type_name(&self) -> String {
        // add the lifetime
        self.to_type_name_base() + "<'j>"
    }

    pub(crate) fn to_jni_class_name(&self) -> String {
        // add the lifetime
        self.to_type_name_base() + "Class<'j>"
    }
}

impl<'a> From<&Cow<'a, str>> for ObjectType {
    fn from(path_name: &Cow<'a, str>) -> Self {
        match path_name {
            _ if &*path_name == "java/lang/Class" => Self::JClass,
            _ if &*path_name == "java/nio/ByteBuffer" => Self::JByteBuffer,
            _ if &*path_name == "java/lang/Object" => Self::JObject,
            _ if &*path_name == "java/lang/String" => Self::JString,
            _ if &*path_name == "java/lang/Throwable" => Self::JThrowable,
            path_name => Self::Object(path_name.to_string()),
        }
    }
}
