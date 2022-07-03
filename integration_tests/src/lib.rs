use jaffi_support::jni::{objects::JString, JNIEnv};
use net_bluejekyll::net_bluejekyll_NativeStrings;

use crate::net_bluejekyll::{
    net_bluejekyll_NativePrimitives, net_bluejekyll_NativePrimitivesClass,
    Static_net_bluejekyll_NativePrimitives,
};

mod net_bluejekyll {
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

    fn eatString(&self, _this: net_bluejekyll_NativeStrings<'j>, arg0: String) -> () {
        println!("eatString ate: {arg0}");
    }
}
