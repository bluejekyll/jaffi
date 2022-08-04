use jaffi_support::{
    jni::{objects::JObject, JNIEnv},
    Error,
};
use net_bluejekyll::NetBluejekyllNativeStrings;

use crate::net_bluejekyll::*;

mod net_bluejekyll {
    #![allow(dead_code)]

    include!(concat!(env!("OUT_DIR"), "/generated_jaffi.rs"));
}

struct NativePrimitivesRsImpl<'j> {
    env: JNIEnv<'j>,
}

impl<'j> net_bluejekyll::NativePrimitivesRs<'j> for NativePrimitivesRsImpl<'j> {
    /// Costruct this type from the Java object
    ///
    /// Implementations should consider storing both values as types on the implementation object
    fn from_env(env: JNIEnv<'j>) -> Self {
        Self { env }
    }

    fn void_void(&self, _this: NetBluejekyllNativePrimitivesClass<'j>) {
        println!("void_1void: do nothing");
    }

    fn void_long_j(&self, _this: NetBluejekyllNativePrimitivesClass<'j>, arg0: i64) {
        println!("void_1long__J: got {arg0}");
    }

    fn void_long_ji(&self, _this: NetBluejekyllNativePrimitives<'j>, arg0: i64, arg1: i32) -> i64 {
        let ret = arg0 + arg1 as i64;
        println!("void_1long__JI: {arg0} + {arg1} = {ret}");
        ret
    }

    fn long_int_int(&self, _this: NetBluejekyllNativePrimitives<'j>, arg0: i32, arg1: i32) -> i64 {
        let ret = arg0 as i64 + arg1 as i64;
        println!("void_1long__JI: {arg0} + {arg1} = {ret}");
        ret
    }

    fn add_values_native(
        &self,
        this: NetBluejekyllNativePrimitives<'j>,
        arg0: i32,
        arg1: i32,
    ) -> i64 {
        println!("add_values_native: calling java with: {arg0}, {arg1}");
        let ret = this.add_values(self.env, arg0, arg1);
        println!("add_1values_1native: got result from java: {ret}");
        ret
    }

    fn print_hello_native(&self, this: NetBluejekyllNativePrimitives<'j>) {
        println!("print_hello_native: calling print_hello");
        this.print_hello(self.env)
    }

    fn print_hello_native_static(&self, this: NetBluejekyllNativePrimitivesClass<'j>) {
        println!("print_hello_native_static: calling print_hello, statically");
        this.print_hello(self.env)
    }

    fn call_dad_native(
        &self,
        this: net_bluejekyll::NetBluejekyllNativePrimitives<'j>,
        arg0: i32,
    ) -> i32 {
        println!("call_dad_native with {arg0}");

        let parent = this.as_net_bluejekyll_parent_class();
        parent.call_1dad(self.env, arg0)
    }

    fn unsupported(
        &self,
        _this: NetBluejekyllNativePrimitives<'j>,
        arg0: net_bluejekyll::JavaIoFile<'j>,
    ) -> net_bluejekyll::JavaIoFile<'j> {
        arg0
    }

    fn unsupported_return_native(
        &self,
        _this: NetBluejekyllNativePrimitives<'j>,
    ) -> NetBluejekyllUnsupported2<'j> {
        panic!("this is just a compilation test")
    }
}

struct NativeStringsRsImpl<'j> {
    env: JNIEnv<'j>,
}

impl<'j> net_bluejekyll::NativeStringsRs<'j> for NativeStringsRsImpl<'j> {
    /// Costruct this type from the Java object
    ///
    /// Implementations should consider storing both values as types on the implementation object
    fn from_env(env: JNIEnv<'j>) -> Self {
        Self { env }
    }

    fn ctor(
        &self,
        _class: NetBluejekyllNativeStringsClass<'j>,
        arg0: String,
    ) -> NetBluejekyllNativeStrings<'j> {
        println!("ctor: {arg0}");
        NetBluejekyllNativeStrings::new_1net_bluejekyll_native_strings_ljava_lang_string_2(
            self.env, arg0,
        )
    }

    fn eat_string(&self, _this: NetBluejekyllNativeStrings<'j>, arg0: String) {
        println!("eatString ate: {arg0}");
    }

    fn tie_off_string(&self, _this: NetBluejekyllNativeStrings<'j>, arg0: String) -> String {
        println!("tieOffString got: {arg0}");
        arg0
    }

    fn return_string_native(&self, this: NetBluejekyllNativeStrings<'j>, append: String) -> String {
        let ret = this.return_string(self.env, append);
        println!("returnStringNative got: {ret}");

        ret
    }
}

pub(crate) struct NativeArraysRsImpl<'j> {
    env: JNIEnv<'j>,
}

impl<'j> net_bluejekyll::NativeArraysRs<'j> for NativeArraysRsImpl<'j> {
    fn from_env(env: jaffi_support::jni::JNIEnv<'j>) -> Self {
        Self { env }
    }

    fn send_bytes(
        &self,
        _this: net_bluejekyll::NetBluejekyllNativeArraysClass<'j>,
        arg0: jaffi_support::arrays::JavaByteArray<'_>,
    ) {
        let slice = arg0.as_slice(&self.env).expect("no data?");

        println!("sendBytes: {:x?}", &slice[..]);
    }

    fn get_bytes(
        &self,
        _this: net_bluejekyll::NetBluejekyllNativeArraysClass<'j>,
        arg0: jaffi_support::arrays::JavaByteArray<'j>,
    ) -> jaffi_support::arrays::JavaByteArray<'j> {
        println!(
            "getBytes: {:x?}",
            &arg0.as_slice(&self.env).expect("no data")[..]
        );
        arg0
    }

    fn new_bytes(
        &self,
        _this: net_bluejekyll::NetBluejekyllNativeArraysClass<'j>,
    ) -> jaffi_support::arrays::JavaByteArray<'j> {
        let bytes: [u8; 4] = [0xCA, 0xFE, 0xBA, 0xBE];

        let jarray = jaffi_support::arrays::JavaByteArray::new(self.env, &bytes)
            .expect("could not create array");

        println!(
            "newBytes: {:x?}",
            &jarray.as_slice(&self.env).expect("no data")[..]
        );

        jarray
    }

    fn new_java_bytes_native(
        &self,
        this: net_bluejekyll::NetBluejekyllNativeArrays<'j>,
    ) -> jaffi_support::arrays::JavaByteArray<'j> {
        let bytes = this.new_java_bytes(self.env);

        println!(
            "newJavaBytesNative: {:x?}",
            &bytes.as_slice(&self.env).expect("no data")[..]
        );

        bytes
    }
}

struct RustKeywordsRsImpl<'j> {
    _env: JNIEnv<'j>,
}

impl<'j> RustKeywordsRs<'j> for RustKeywordsRsImpl<'j> {
    fn from_env(env: JNIEnv<'j>) -> Self {
        Self { _env: env }
    }

    fn r#as(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r#async(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r#await(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r_crate(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r#dyn(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r#extern(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r#fn(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r#impl(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r#in(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r#let(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r#loop(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r#match(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r#mod(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r#move(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r#mut(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r#pub(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r#ref(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r_self(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn self_18(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r#struct(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r#trait(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r#type(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r#union(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r#unsafe(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r#use(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }

    fn r#where(&self, _this: NetBluejekyllRustKeywords<'j>) {
        todo!()
    }
}

struct ExceptionsRsImpl<'j> {
    env: JNIEnv<'j>,
}

impl<'j> ExceptionsRs<'j> for ExceptionsRsImpl<'j> {
    fn from_env(env: JNIEnv<'j>) -> Self {
        Self { env }
    }

    fn throws_something(
        &self,
        _this: NetBluejekyllExceptions<'j>,
    ) -> Result<(), Error<SomethingExceptionErr>> {
        Err(Error::new(
            SomethingExceptionErr::SomethingException(SomethingException),
            "Test Message",
        ))
    }

    fn throws_something_ljava_lang_string_2(
        &self,
        _this: NetBluejekyllExceptions<'j>,
        msg: String,
    ) -> Result<(), Error<SomethingExceptionErr>> {
        Err(Error::new(
            SomethingExceptionErr::SomethingException(SomethingException),
            msg,
        ))
    }

    fn catches_something(
        &self,
        this: net_bluejekyll::NetBluejekyllExceptions<'j>,
    ) -> net_bluejekyll::NetBluejekyllSomethingException<'j> {
        let ex = this
            .i_always_throw(self.env)
            .expect_err("error expected here");

        #[allow(irrefutable_let_patterns)]
        if let SomethingExceptionErr::SomethingException(SomethingException) = ex.throwable() {
            net_bluejekyll::NetBluejekyllSomethingException::from(JObject::from(ex.exception()))
        } else {
            panic!("expected SomethingException")
        }
    }

    fn panics_are_runtime_exceptions(&self, _this: NetBluejekyllExceptions<'j>) {
        panic!("{}", "Panics are safe".to_string());
    }
}
