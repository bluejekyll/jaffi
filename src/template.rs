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
use jaffi_support::{
    jni::{
        objects::{JByteBuffer, JClass, JObject, JString, JThrowable},
        sys,
    },
    JavaBoolean, JavaByte, JavaChar, JavaDouble, JavaFloat, JavaInt, JavaLong, JavaShort, JavaVoid,
};
use serde::Serialize;

use tinytemplate::TinyTemplate;

use crate::Error;

pub(crate) static RUST_FFI: &str = "RUST_FFI";
pub(crate) static JAVA_FUNCTION_CALL: &str = "JAVA_FUNCTION_CALL";

/// Template for the generated rust files.
///
/// This generates the trait for each of the FFI functions.
static RUST_FFI_TEMPLATE: &str = r#"
use std::ops::Deref;

use jaffi_support::\{
    FromJavaToRust,
    jni::\{
        JNIEnv,
        objects::\{JByteBuffer, JClass, JObject, JValue},
        self,
        sys::jlong,
    }
};

{# ** The Support Types, Java Object and Class wrappers, etc ** #}
{{ for obj in objects }}
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct { obj.class_name -}(JClass<'j>);

impl<'j> { obj.static_trait_name } for { obj.class_name } \{}

impl<'j> { obj.class_name -} \{
    fn java_class_desc() -> &'static str \{
        "{- obj.java_name -}"
    }
}

impl<'j> std::ops::Deref for { obj.class_name -} \{
    type Target = JClass<'j>;

    fn deref(&self) -> &Self::Target \{
        &self.0
    }
}

impl<'j> FromJavaToRust for { obj.class_name } \{
    type Rust = { obj.class_name };

    fn java_to_rust(self) -> Self::Rust \{
        self
    }

    fn rust_to_java(rust: Self::Rust) -> Self \{
        rust
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct { obj.obj_name -}(JObject<'j>);

impl<'j> { obj.static_trait_name } for { obj.obj_name } \{}

impl<'j> { obj.obj_name -} \{
    /// Returns the type name in java, e.g. `Object` is `"java/lang/Object"`
    pub fn java_class_desc() -> &'static str \{
        "{- obj.java_name -}"
    }

    {{ for function in obj.methods }}
    {{ if not function.is_static }}
    {{ call JAVA_FUNCTION_CALL with function }}
    {{ endif }}
    {{ endfor}}
}

pub trait { obj.static_trait_name } \{
    {{ for function in obj.methods }}
    {{ if function.is_static }}
    {{ call JAVA_FUNCTION_CALL with function }}
    {{ endif }}
    {{ endfor}}
}

impl<'j> std::ops::Deref for { obj.obj_name -} \{
    type Target = JObject<'j>;

    fn deref(&self) -> &Self::Target \{
        &self.0
    }
}

impl<'j> From<{ obj.obj_name -}> for JObject<'j> \{
    fn from(obj: { obj.obj_name -}) -> Self \{
        obj.0
    }
}

impl<'j> FromJavaToRust for { obj.obj_name } \{
    type Rust = { obj.obj_name };

    fn java_to_rust(self) -> Self::Rust \{
        self
    }

    fn rust_to_java(rust: Self::Rust) -> Self \{
        rust
    }
}
{{ endfor }}

{# ** The Support Types, Java Object and Class wrappers, etc ** #}

{{ for class in class_ffis}}
// This is the trait developers must implement
use super::{- class.trait_impl -};

pub trait { class.trait_name }<'j> \{
    /// Costruct this type from the Java object
    /// 
    /// Implementations should consider storing both values as types on the implementation object
    fn from_env(env: JNIEnv<'j>) -> Self;
{{ for function in class.functions }}
    fn { function.fn_ffi_name }(
        &self,
        this: {{ if function.is_static }}{ function.class_ffi_name -}{{ else }}{ function.object_ffi_name -}{{ endif }},
        {{- for arg in function.arguments }}
        { arg.name }: { arg.rs_ty },
        {{- endfor }}    
    ) -> { function.rs_result -};
{{ endfor }}
}

{{ for function in class.functions }}
/// JNI method signature { function.signature }
#[no_mangle]
pub extern "system" fn {function.fn_export_ffi_name -}<'j>(
    env: JNIEnv<'j>,
    this: {{ if function.is_static }}{ function.class_ffi_name -}{{ else }}{ function.object_ffi_name -}{{ endif }},
    {{- for arg in function.arguments }}
    { arg.name }: { arg.ty },
    {{- endfor }}
) -> { function.result } \{
    let myself = { class.trait_impl }::from_env(env);
    
    {{- for arg in function.arguments }}
    let { arg.name } = { arg.name }.java_to_rust();
    {{- endfor }}
    
    let result = myself.{ function.fn_ffi_name } (
        this,
        {{- for arg in function.arguments }}
        { arg.name },
        {{- endfor }}
    );

    { function.result -}::rust_to_java(result)
}
{{ endfor }}
{{ endfor }}
"#;

/// This expects the Function type as the serialized data
static JAVA_FUNCTION_CALL_TEMPLATE: &str = r#"
    /// A wrapper for the java function { name }
    /// 
    /// # Arguments
    /// 
    /// * `env` - this should be the same JNIEnv "owning" this object
    {{ if not is_static }}pub{{ endif }} fn { fn_ffi_name }(
        &self,
        env: JNIEnv<'_>,
        {{- for arg in arguments }}
        { arg.name }: { arg.rs_ty },
        {{- endfor }}  
    ) -> { rs_result -} \{
        let args: &[JValue<'_>] = &[
            {{- for arg in arguments }}
            JValue::from({ arg.name }),
            {{- endfor }} 
        ];

        {{ if is_static }}
        let jvalue = match env.call_static_method(
            "{ object_java_desc }",
            "{ name }",
            "{ signature }",
            args
        ) \{
            Ok(jvalue) => jvalue,
            Err(e) => panic!("error calling java, \{e}"),
        };
        {{ else }}
        let jvalue = match env.call_method(
            self.0,
            "{ name }",
            "{ signature }",
            args
        ) \{
            Ok(jvalue) => jvalue,
            Err(e) => panic!("error calling java, \{e}"),
        };
        {{ endif }}

        match jvalue.try_into() \{
            Ok(ret) => ret,
            Err(e) => panic!("could not convert to rust from jvalue, \{e}"),
        }
    }
"#;

pub(crate) fn new_engine() -> Result<TinyTemplate<'static>, Error> {
    let mut tt = TinyTemplate::new();
    tt.add_template(RUST_FFI, RUST_FFI_TEMPLATE)?;
    tt.add_template(JAVA_FUNCTION_CALL, JAVA_FUNCTION_CALL_TEMPLATE)?;
    tt.set_default_formatter(&tinytemplate::format_unescaped);
    Ok(tt)
}

#[derive(Serialize)]
pub(crate) struct RustFfi {
    pub(crate) class_ffis: Vec<ClassFfi>,
    pub(crate) objects: Vec<Object>,
}

#[derive(Serialize)]
pub(crate) struct ClassFfi {
    pub(crate) class_name: String,
    pub(crate) type_name: String,
    pub(crate) trait_name: String,
    pub(crate) trait_impl: String,
    pub(crate) functions: Vec<Function>,
}

#[derive(Serialize)]
pub(crate) struct Function {
    pub(crate) name: String,
    pub(crate) object_java_desc: String,
    pub(crate) fn_export_ffi_name: String,
    pub(crate) class_ffi_name: String,
    pub(crate) object_ffi_name: String,
    pub(crate) fn_ffi_name: String,
    pub(crate) signature: String,
    pub(crate) is_static: bool,
    pub(crate) arguments: Vec<Arg>,
    pub(crate) result: String,
    pub(crate) rs_result: String,
}

#[derive(Serialize)]
pub(crate) struct Arg {
    pub(crate) name: String,
    pub(crate) ty: String,
    pub(crate) rs_ty: String,
}

#[derive(Serialize)]
pub(crate) struct Object {
    pub(crate) java_name: String,
    pub(crate) class_name: String,
    pub(crate) obj_name: String,
    pub(crate) static_trait_name: String,
    pub(crate) methods: Vec<Function>,
}

impl From<ObjectType> for Object {
    fn from(ty: ObjectType) -> Self {
        let java_name = ty.as_descriptor().to_string();
        let class_name = ty.to_jni_class_name();
        let obj_name = ty.to_jni_type_name();
        let static_trait_name = format!("Static_{}", ty.to_rs_type_name());

        Object {
            java_name,
            class_name,
            obj_name,
            static_trait_name,
            methods: Vec::new(),
        }
    }
}

#[derive(Serialize)]
pub(crate) enum Return {
    Void,
    Val(JniType),
}

impl Return {
    pub(crate) fn from_java(field_type: &ReturnDescriptor<'_>) -> Self {
        match field_type {
            ReturnDescriptor::Void => Self::Void,
            ReturnDescriptor::Return(val) => Self::Val(JniType::from_java(val)),
        }
    }

    pub(crate) fn to_jni_type_name(&self) -> String {
        match self {
            Self::Void => std::any::type_name::<JavaVoid>().to_string(),
            Self::Val(ty) => ty.to_jni_type_name(),
        }
    }

    pub(crate) fn to_rs_type_name(&self) -> String {
        match self {
            Self::Void => std::any::type_name::<()>().to_string(),
            Self::Val(ty) => ty.to_rs_type_name(),
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
    ///
    /// These must all be marked `#[repr(transparent)]` in order to be used at the FFI boundary
    pub(crate) fn to_jni_type_name(&self) -> String {
        match self {
            Self::Ty(BaseJniTy::Jbyte) => std::any::type_name::<JavaByte>().into(),
            Self::Ty(BaseJniTy::Jchar) => std::any::type_name::<JavaChar>().into(),
            Self::Ty(BaseJniTy::Jdouble) => std::any::type_name::<JavaDouble>().into(),
            Self::Ty(BaseJniTy::Jfloat) => std::any::type_name::<JavaFloat>().into(),
            Self::Ty(BaseJniTy::Jint) => std::any::type_name::<JavaInt>().into(),
            Self::Ty(BaseJniTy::Jlong) => std::any::type_name::<JavaLong>().into(),
            Self::Ty(BaseJniTy::Jshort) => std::any::type_name::<JavaShort>().into(),
            Self::Ty(BaseJniTy::Jboolean) => std::any::type_name::<JavaBoolean>().into(),
            Self::Ty(BaseJniTy::Jobject(obj)) => obj.to_jni_type_name(),
            // in JNI the array is always jarray
            Self::Jarray { .. } => std::any::type_name::<sys::jarray>().into(),
        }
    }

    pub(crate) fn to_rs_type_name(&self) -> String {
        match self {
            Self::Ty(BaseJniTy::Jbyte) => std::any::type_name::<i8>().into(),
            Self::Ty(BaseJniTy::Jchar) => std::any::type_name::<char>().into(),
            Self::Ty(BaseJniTy::Jdouble) => std::any::type_name::<f64>().into(),
            Self::Ty(BaseJniTy::Jfloat) => std::any::type_name::<f32>().into(),
            Self::Ty(BaseJniTy::Jint) => std::any::type_name::<i32>().into(),
            Self::Ty(BaseJniTy::Jlong) => std::any::type_name::<i64>().into(),
            Self::Ty(BaseJniTy::Jshort) => std::any::type_name::<i16>().into(),
            Self::Ty(BaseJniTy::Jboolean) => std::any::type_name::<bool>().into(),
            Self::Ty(BaseJniTy::Jobject(obj)) => obj.to_rs_type_name(),
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

    /// Returns the typename with a lifetime
    pub(crate) fn to_jni_type_name(&self) -> String {
        // add the lifetime
        self.to_type_name_base() + "<'j>"
    }

    /// Returns the typename plus "Class" with a lifetime
    pub(crate) fn to_jni_class_name(&self) -> String {
        // add the lifetime
        self.to_type_name_base() + "Class<'j>"
    }

    /// Returns the typename without a lifetime
    pub(crate) fn to_rs_type_name(&self) -> String {
        self.to_type_name_base()
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
