use jaffi_support::jni::JNIEnv;

use crate::net_bluejekyll::{net_bluejekyll_NativeClass, net_bluejekyll_NativeClassClass};

mod net_bluejekyll {
    include!(concat!(env!("OUT_DIR"), "/generated_jaffi.rs"));
}

struct NativeClassRsImpl {}

impl<'j> net_bluejekyll::NativeClassRs<'j> for NativeClassRsImpl {
    /// Costruct this type from the Java object
    ///
    /// Implementations should consider storing both values as types on the implementation object
    fn from_env(env: JNIEnv<'j>) -> Self {
        todo!()
    }

    fn test_1void_1void(&self, this: net_bluejekyll_NativeClassClass<'j>) -> () {
        todo!()
    }

    fn test_1void_1long__J(&self, this: net_bluejekyll_NativeClassClass<'j>, arg0: i64) -> () {
        todo!()
    }

    fn test_1void_1long__JJ(
        &self,
        this: net_bluejekyll_NativeClass<'j>,
        arg0: i64,
        arg1: i64,
    ) -> () {
        todo!()
    }

    fn test_1long_1long(&self, this: net_bluejekyll_NativeClass<'j>, arg0: i64, arg1: i64) -> i64 {
        todo!()
    }
}
