use jaffi_support::jni::JNIEnv;
use net_bluejekyll::net_bluejekyll_NativeStrings;

use crate::net_bluejekyll::{
    net_bluejekyll_NativePrimitives, net_bluejekyll_NativePrimitivesClass,
    Static_net_bluejekyll_NativePrimitives,
};

mod net_bluejekyll {
    #![allow(non_camel_case_types, dead_code, non_snake_case)]

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

    fn void_1void(&self, _this: net_bluejekyll_NativePrimitivesClass<'j>) -> () {
        println!("void_1void: do nothing");
    }

    fn void_1long__J(&self, _this: net_bluejekyll_NativePrimitivesClass<'j>, arg0: i64) -> () {
        println!("void_1long__J: got {arg0}");
    }

    fn void_1long__JI(
        &self,
        _this: net_bluejekyll_NativePrimitives<'j>,
        arg0: i64,
        arg1: i32,
    ) -> i64 {
        let ret = arg0 + arg1 as i64;
        println!("void_1long__JI: {arg0} + {arg1} = {ret}");
        ret
    }

    fn long_1int_1int(
        &self,
        _this: net_bluejekyll_NativePrimitives<'j>,
        arg0: i32,
        arg1: i32,
    ) -> i64 {
        let ret = arg0 as i64 + arg1 as i64;
        println!("void_1long__JI: {arg0} + {arg1} = {ret}");
        ret
    }

    fn add_1values_1native(
        &self,
        this: net_bluejekyll_NativePrimitives<'j>,
        arg0: i32,
        arg1: i32,
    ) -> i64 {
        println!("add_1values_1native: calling java with: {arg0}, {arg1}");
        let ret = this.add_1values(self.env, arg0, arg1);
        println!("add_1values_1native: got result from java: {ret}");
        ret
    }

    fn print_1hello_1native(&self, this: net_bluejekyll_NativePrimitives<'j>) -> () {
        println!("print_1hello_1native: calling print_hello");
        this.print_1hello(self.env)
    }

    fn print_1hello_1native_1static(&self, this: net_bluejekyll_NativePrimitivesClass<'j>) -> () {
        println!("print_1hello_1native_1static: calling print_hello, statically");
        this.print_1hello(self.env)
    }

    fn call_1dad_1native(
        &self,
        this: net_bluejekyll::net_bluejekyll_NativePrimitives<'j>,
        arg0: i32,
    ) -> i32 {
        println!("call_1dad_1native with {arg0}");

        let parent = this.as_net_bluejekyll_ParentClass();
        parent.call_1dad(self.env, arg0)
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

    fn eatString(&self, _this: net_bluejekyll_NativeStrings<'j>, arg0: String) {
        println!("eatString ate: {arg0}");
    }

    fn tieOffString(&self, _this: net_bluejekyll_NativeStrings<'j>, arg0: String) -> String {
        println!("tieOffString got: {arg0}");
        arg0
    }

    fn returnStringNative(&self, this: net_bluejekyll_NativeStrings<'j>, append: String) -> String {
        let ret = this.returnString(self.env, append);
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

    fn sendBytes(
        &self,
        this: net_bluejekyll::net_bluejekyll_NativeArraysClass<'j>,
        arg0: jaffi_support::arrays::JavaByteArray<'_>,
    ) {
        let slice = arg0.as_slice(&self.env).expect("no data?");

        println!("sendBytes: {:x?}", &slice[..]);
    }

    fn getBytes(
        &self,
        this: net_bluejekyll::net_bluejekyll_NativeArraysClass<'j>,
        arg0: jaffi_support::arrays::JavaByteArray<'j>,
    ) -> jaffi_support::arrays::JavaByteArray<'j> {
        println!(
            "getBytes: {:x?}",
            &arg0.as_slice(&self.env).expect("no data")[..]
        );
        arg0
    }

    fn newBytes(
        &self,
        this: net_bluejekyll::net_bluejekyll_NativeArraysClass<'j>,
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
}
