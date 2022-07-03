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
    FromRustToJava,
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

impl<'j> FromJavaToRust<'j> for { obj.class_name } \{
    type Rust = { obj.class_name };

    fn java_to_rust(self, _env: JNIEnv<'j>) -> Self::Rust \{
        self
    }
}

impl<'j> FromRustToJava<'j> for { obj.class_name } \{
    type Java = { obj.class_name };

    fn rust_to_java(self, _env: JNIEnv<'j>) -> Self \{
        self
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

    {{ for interface in obj.interfaces -}}
    pub fn as_{- interface }(&self) -> { interface -}<'j> \{
        { interface }(self.0)
    }
    {{ endfor }}

    {{ for function in obj.methods -}}
    {{ if not function.is_static -}}
    {{ call JAVA_FUNCTION_CALL with function }}
    {{- endif }}
    {{- endfor }}
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

impl<'j> FromJavaToRust<'j> for { obj.obj_name } \{
    type Rust = { obj.obj_name };

    fn java_to_rust(self, _env: JNIEnv<'j>) -> Self::Rust \{
        self
    }
}

impl<'j> FromRustToJava<'j> for { obj.obj_name } \{
    type Java = { obj.obj_name };

    fn rust_to_java(self, _env: JNIEnv<'j>) -> Self \{
        self
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
    let { arg.name } = { arg.name }.java_to_rust(env);
    {{- endfor }}
    
    let result = myself.{ function.fn_ffi_name } (
        this,
        {{- for arg in function.arguments }}
        { arg.name },
        {{- endfor }}
    );

    result.rust_to_java(env)
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
    pub(crate) type_name: RustTypeName,
    pub(crate) trait_name: String,
    pub(crate) trait_impl: String,
    pub(crate) functions: Vec<Function>,
}

#[derive(Serialize)]
pub(crate) struct Function {
    pub(crate) name: String,
    pub(crate) object_java_desc: JavaDesc,
    pub(crate) fn_export_ffi_name: ClassAndFuncAbi,
    pub(crate) class_ffi_name: RustTypeName,
    pub(crate) object_ffi_name: RustTypeName,
    pub(crate) fn_ffi_name: FuncAbi,
    pub(crate) signature: JavaDesc,
    pub(crate) is_static: bool,
    pub(crate) arguments: Vec<Arg>,
    pub(crate) result: RustTypeName,
    pub(crate) rs_result: RustTypeName,
}

#[derive(Serialize)]
pub(crate) struct Arg {
    pub(crate) name: String,
    pub(crate) ty: RustTypeName,
    pub(crate) rs_ty: RustTypeName,
}

#[derive(Serialize)]
pub(crate) struct Object {
    pub(crate) java_name: JavaDesc,
    pub(crate) class_name: RustTypeName,
    pub(crate) obj_name: RustTypeName,
    pub(crate) static_trait_name: RustTypeName,
    pub(crate) methods: Vec<Function>,
    pub(crate) interfaces: Vec<RustTypeName>,
}

impl From<ObjectType> for Object {
    fn from(ty: ObjectType) -> Self {
        let java_name = ty.as_descriptor();
        let class_name = ty.to_jni_class_name();
        let obj_name = ty.to_jni_type_name();
        let static_trait_name = ty.to_rs_type_name().prepend("Static_");

        Object {
            java_name,
            class_name,
            obj_name,
            static_trait_name,
            methods: Vec::new(),
            interfaces: Vec::new(),
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

    pub(crate) fn to_jni_type_name(&self) -> RustTypeName {
        match self {
            Self::Void => std::any::type_name::<JavaVoid>().into(),
            Self::Val(ty) => ty.to_jni_type_name(),
        }
    }

    pub(crate) fn to_rs_type_name(&self) -> RustTypeName {
        match self {
            Self::Void => "()".into(),
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
    pub(crate) fn to_jni_type_name(&self) -> RustTypeName {
        match self {
            Self::Ty(BaseJniTy::Jbyte) => std::any::type_name::<JavaByte>().into(),
            Self::Ty(BaseJniTy::Jchar) => std::any::type_name::<JavaChar>().into(),
            Self::Ty(BaseJniTy::Jdouble) => std::any::type_name::<JavaDouble>().into(),
            Self::Ty(BaseJniTy::Jfloat) => std::any::type_name::<JavaFloat>().into(),
            Self::Ty(BaseJniTy::Jint) => std::any::type_name::<JavaInt>().into(),
            Self::Ty(BaseJniTy::Jlong) => std::any::type_name::<JavaLong>().into(),
            Self::Ty(BaseJniTy::Jshort) => std::any::type_name::<JavaShort>().into(),
            Self::Ty(BaseJniTy::Jboolean) => std::any::type_name::<JavaBoolean>().into(),
            Self::Ty(BaseJniTy::Jobject(obj)) => obj.to_type_name_base(),
            // in JNI the array is always jarray
            Self::Jarray { .. } => std::any::type_name::<sys::jarray>().into(),
        }
    }

    pub(crate) fn to_rs_type_name(&self) -> RustTypeName {
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
                Ty::Object(obj) => {
                    BaseJniTy::Jobject(ObjectType::from(JavaDesc::from(obj.to_string())))
                }
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
    Object(JavaDesc),
}

impl ObjectType {
    pub(crate) fn as_descriptor(&self) -> JavaDesc {
        match self {
            Self::JClass => "java/lang/Class".into(),
            Self::JByteBuffer => "java/nio/ByteBuffer".into(),
            Self::JObject => "java/lang/Object".into(),
            Self::JString => "java/lang/String".into(),
            Self::JThrowable => "java/lang/Throwable".into(),
            Self::Object(desc) => desc.clone(),
        }
    }

    fn to_type_name_base(&self) -> RustTypeName {
        match *self {
            Self::JClass => "jni::objects::JClass<'j>".into(),
            Self::JByteBuffer => "jni::objects::JByteBuffer<'j>".into(),
            Self::JObject => "jni::objects::JObject<'j>".into(),
            Self::JString => "jni::objects::JString<'j>".into(),
            Self::JThrowable => "jni::objects::JThrowable<'j>".into(),
            Self::Object(ref obj) => obj.0.replace('/', "_").into(),
        }
    }

    /// Returns the typename with a lifetime
    pub(crate) fn to_jni_type_name(&self) -> RustTypeName {
        // add the lifetime
        self.to_type_name_base().append("<'j>")
    }

    /// Returns the typename plus "Class" with a lifetime
    pub(crate) fn to_jni_class_name(&self) -> RustTypeName {
        // add the lifetime
        self.to_type_name_base().append("Class<'j>")
    }

    /// Returns the typename without a lifetime
    pub(crate) fn to_rs_type_name(&self) -> RustTypeName {
        match *self {
            Self::JClass => "jni::objects::JClass".into(),
            Self::JByteBuffer => "jni::objects::JByteBuffer".into(),
            Self::JObject => "jni::objects::JObject".into(),
            Self::JString => "String".into(),
            Self::JThrowable => "jni::objects::JThrowable".into(),
            Self::Object(ref obj) => obj.0.replace('/', "_").into(),
        }
    }
}

// impl<'a, S: AsRef<str> + 'a> From<S> for ObjectType {
//     fn from(path_name: S) -> Self {
//         let path_name = path_name.as_ref();
//         match dbg!(path_name) {
//             _ if &*path_name == "java/lang/Class" => Self::JClass,
//             _ if &*path_name == "java/nio/ByteBuffer" => Self::JByteBuffer,
//             _ if &*path_name == "java/lang/Object" => Self::JObject,
//             _ if &*path_name == "java/lang/String" => Self::JString,
//             _ if &*path_name == "java/lang/Throwable" => Self::JThrowable,
//             path_name => Self::Object(path_name.to_string().into()),
//         }
//     }
// }

impl From<JavaDesc> for ObjectType {
    fn from(java_desc: JavaDesc) -> Self {
        let path_name = java_desc.as_str();
        match dbg!(path_name) {
            _ if &*path_name == "java/lang/Class" => Self::JClass,
            _ if &*path_name == "java/nio/ByteBuffer" => Self::JByteBuffer,
            _ if &*path_name == "java/lang/Object" => Self::JObject,
            _ if &*path_name == "java/lang/String" => Self::JString,
            _ if &*path_name == "java/lang/Throwable" => Self::JThrowable,
            path_name => Self::Object(path_name.to_string().into()),
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub(crate) struct FuncAbi(JniAbi);

impl From<JniAbi> for FuncAbi {
    fn from(abi: JniAbi) -> Self {
        FuncAbi(abi)
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub(crate) struct ClassAndFuncAbi(JniAbi);

/// An escaped String for the Java JNI ABI
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub(crate) struct JniAbi(String);

impl FuncAbi {
    pub(crate) fn with_class(&self, class: &RustTypeName) -> ClassAndFuncAbi {
        let ffi_name = class
            .clone()
            .prepend("Java_")
            .append("_")
            .append(&self.0 .0)
            .to_string();
        ClassAndFuncAbi(JniAbi(ffi_name))
    }

    pub(crate) fn with_descriptor(self, descriptor: &JavaDesc) -> Self {
        // strip the '(', ')', and return from the descriptor
        let descriptor = descriptor.0.strip_prefix('(').unwrap_or(&descriptor.0);
        let descriptor = if let Some(pos) = descriptor.find(')') {
            &descriptor[..pos]
        } else {
            descriptor
        };

        let abi_descriptor = JniAbi::from(descriptor);

        Self(JniAbi(format!("{self}__{abi_descriptor}")))
    }
}

/// Converts the method info into the native ABI name, see [resolving native method names](https://docs.oracle.com/en/java/javase/18/docs/specs/jni/design.html#resolving-native-method-names)
///
/// ```text
///
/// The JNI defines a 1:1 mapping from the name of a native method declared in Java to the name of a native method residing in a native library. The VM uses this mapping to dynamically link a Java invocation of a native method to the corresponding implementation in the native library.
///
/// The mapping produces a native method name by concatenating the following components derived from a native method declaration:
///
///     the prefix Java_
///     given the binary name, in internal form, of the class which declares the native method: the result of escaping the name.
///     an underscore ("_")
///     the escaped method name
///     if the native method declaration is overloaded: two underscores ("__") followed by the escaped parameter descriptor (JVMS 4.3.3) of the method declaration.
///
/// Escaping leaves every alphanumeric ASCII character (A-Za-z0-9) unchanged, and replaces each UTF-16 code unit in the table below with the corresponding escape sequence. If the name to be escaped contains a surrogate pair, then the high-surrogate code unit and the low-surrogate code unit are escaped separately. The result of escaping is a string consisting only of the ASCII characters A-Za-z0-9 and underscore.
/// | UTF-16 code unit                | Escape sequence |
/// | Forward slash (/, U+002F)       | _               |
/// | Underscore (_, U+005F)          | _1              |
/// | Semicolon (;, U+003B)           | _2              |
/// | Left square bracket ([, U+005B) | _3              |
/// | Any UTF-16 code unit \uWXYZ that does not represent alphanumeric ASCII (A-Za-z0-9), forward slash, underscore, semicolon, or left square bracket | _0wxyz where w, x, y, and z are the lower-case forms of the hexadecimal digits W, X, Y, and Z. (For example, U+ABCD becomes _0abcd.)|
///
/// Escaping is necessary for two reasons. First, to ensure that class and method names in Java source code, which may include Unicode characters, translate into valid function names in C source code. Second, to ensure that the parameter descriptor of a native method, which uses ";" and "[" characters to encode parameter types, can be encoded in a C function name.
///
/// When a Java program invokes a native method, the VM searches the native library by looking first for the short version of the native method name, that is, the name without the escaped argument signature. If a native method with the short name is not found, then the VM looks for the long version of the native method name, that is, the name including the escaped argument signature.
///
/// Looking for the short name first makes it easier to declare implementations in the native library. For example, given this native method in Java:
///
/// package p.q.r;
/// class A {
///     native double f(int i, String s);
/// }
///
/// The corresponding C function can be named Java_p_q_r_A_f, rather than Java_p_q_r_A_f__ILjava_lang_String_2.
///
/// Declaring implementations with long names in the native library is only necessary when two or more native methods in a class have the same name. For example, given these native methods in Java:
///
/// package p.q.r;
/// class A {
///     native double f(int i, String s);
///     native double f(int i, Object s);
/// }
///
/// The corresponding C functions must be named Java_p_q_r_A_f__ILjava_lang_String_2 and Java_p_q_r_A_f__ILjava_lang_Object_2, because the native methods are overloaded.
///
/// Long names in the native library are not necessary if a native method in Java is overloaded by non-native methods only. In the following example, the native method g does not have to be linked using the long name because the other method g is not native and thus does not reside in the native library.
///
/// package p.q.r;
/// class B {
///     int g(int i);
///     native int g(double d);
/// }
///
/// Note that escape sequences can safely begin _0, _1, etc, because class and method names in Java source code never begin with a number. However, that is not the case in class files that were not generated from Java source code. To preserve the 1:1 mapping to a native method name, the VM checks the resulting name as follows. If the process of escaping any precursor string from the native method declaration (class or method name, or argument type) causes a "0", "1", "2", or "3" character from the precursor string to appear unchanged in the result either immediately after an underscore or at the beginning of the escaped string (where it will follow an underscore in the fully assembled name), then the escaping process is said to have "failed". In such cases, no native library search is performed, and the attempt to link the native method invocation will throw UnsatisfiedLinkError. It would be possible to extend the present simple mapping scheme to cover such cases, but the complexity costs would outweigh any benefit.
///
/// Both the native methods and the interface APIs follow the standard library-calling convention on a given platform. For example, UNIX systems use the C calling convention, while Win32 systems use __stdcall.
///
/// Native methods can also be explicitly linked using the RegisterNatives function. Be aware that RegisterNatives can change the documented behavior of the JVM (including cryptographic algorithms, correctness, security, type safety), by changing the native code to be executed for a given native Java method. Therefore use applications that have native libraries utilizing the RegisterNatives function with caution.
/// ```
impl<S: AsRef<str>> From<S> for JniAbi {
    fn from(name: S) -> Self {
        let name = name.as_ref();
        let mut abi_name = String::with_capacity(name.len());

        for ch in name.chars() {
            match ch {
                '.' | '/' => abi_name.push('_'),
                '_' => abi_name.push_str("_1"),
                ';' => abi_name.push_str("_2"),
                '[' => abi_name.push_str("_3"),
                _ if ch.is_ascii_alphanumeric() => abi_name.push(ch),
                _ => {
                    abi_name.push_str("_0");

                    for c in ch.escape_unicode().skip(3).filter(|c| *c != '}') {
                        abi_name.push(c);
                    }
                }
            }
        }

        JniAbi(abi_name)
    }
}

impl fmt::Display for JniAbi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(&self.0)
    }
}

impl fmt::Display for FuncAbi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(&self.0 .0)
    }
}

impl fmt::Display for ClassAndFuncAbi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(&self.0 .0)
    }
}

/// Descriptor in java, like `java.lang.String` or `(Ljava.lang.String;)J`
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub(crate) struct JavaDesc(String);

impl JavaDesc {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for JavaDesc {
    fn from(s: String) -> Self {
        JavaDesc(s.replace('.', "/"))
    }
}

impl From<&str> for JavaDesc {
    fn from(s: &str) -> Self {
        JavaDesc::from(s.to_string())
    }
}

/// Descriptor in java, like `java.lang.String` or `(Ljava.lang.String;)J`
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub(crate) struct RustTypeName(Cow<'static, str>);

impl RustTypeName {
    pub(crate) fn append(&self, s: &str) -> Self {
        let no_lifetime = self.0.trim_end_matches("<'j>").to_string();
        RustTypeName(Cow::Owned(no_lifetime + s))
    }

    pub(crate) fn prepend(&self, s: &str) -> Self {
        RustTypeName(Cow::Owned(s.to_string() + &self.0))
    }
}

impl From<JavaDesc> for RustTypeName {
    fn from(d: JavaDesc) -> Self {
        let abi_name = JniAbi::from(d.0);
        Self(abi_name.to_string().into())
    }
}

impl From<String> for RustTypeName {
    fn from(s: String) -> Self {
        RustTypeName(Cow::Owned(s))
    }
}

impl From<&'static str> for RustTypeName {
    fn from(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }
}

impl fmt::Display for RustTypeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(&self.0)
    }
}
