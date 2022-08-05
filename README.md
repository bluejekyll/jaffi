# Introduction

This library functions similarly to bindgen for C, though Java and JNI do not have as simple a calling convention as other languages, so this generates some bridge functions to make integrating between Rust and Java simpler. The name comes from Ja(va)FFI -> Jaffi

*WARNING* this is very early days, there is not an exhaustive test suite verifying everythig functions properly at the FFI boundary.

## Building this project

Rust and Java toolchains will need to be installed.

- Install Rust: [rustup](https://rustup.rs/)
- Install Just: `cargo install just` // used for simple script execution
- Install Java: [OpenJDK](https://jdk.java.net/18/) // this was tested with Java 18

Now tests should work as expected (there may be a lot of warnings, this is under active development), if it works, you should this output after the build details:

```shell
$> just test
...
Running tests
loadLibrary succeeded for jaffi_integration_tests
running tests jaffi_integration_tests
void_1void: do nothing
void_1long__J: got 100
void_1long__JI: 100 + 10 = 110
void_1long__JI: 2147483647 + 2147483647 = 4294967294
add_1values_1native: calling java with: 2147483647, 2147483647
add_1values_1native: got result from java: 4294967294
print_1hello_1native_1static: calling print_hello, statically
hello!
print_1hello_1native: calling print_hello
hello!
call_1dad_1native with 732
All tests succeeded
```

## Getting started

The Jaffi library will scan class files based on the configuration parameters specified. There are some deficiencies, currently only unzipped classpaths are supported, i.e. if there jars in the classpath the build will fail.

To use the library, this hasn't been published to Crates.io yet, you will need to add dependencies like this to your Cargo.toml:

```toml
[build-dependencies]
jaffi = "0.2.0"

[dependencies]
jaffi_support = "0.2.0"
```

Once that is added, you will need to create a `build.rs` script for executing Jaffi, something like this (see the integration test for a working example [build.rs](https://github.com/bluejekyll/jaffi/blob/084db8c2478bbb43343c4661dafb968f9289575e/integration_tests/build.rs)):

```rust
fn main() -> Result<(), Box<dyn Error>> {
    let class_path = class_path();
    let classes = vec![Cow::from("net.bluejekyll.NativeClass")];
    let classes_to_wrap = vec![Cow::from("net.bluejekyll.ParentClass")];
    let output_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set"));

    let jaffi = Jaffi::builder()
        .native_classes(classes)
        .classes_to_wrap(classes_to_wrap)
        .classpath(vec![Cow::from(class_path)])
        .output_dir(Some(Cow::from(output_dir)))
        .build();

    jaffi.generate()?;

    Ok(())
}
```

If Jaffi runs successfully it will produce a file named `generated_jaffi.rs` in the build path, `OUT_DIR`. This Rust file has a few expectations on the way that it expects interfaces to be implemented. It looks for a type named `super::{Class}RsImpl`, i.e. it expects this to be in the super module, the one above where the generated code is included. The `generated_jaffi.rs` file can be included in a module to achieve this, see the example [NativeClassRsImpl](https://github.com/bluejekyll/jaffi/blob/084db8c2478bbb43343c4661dafb968f9289575e/integration_tests/src/lib.rs#L5-L13):

```rust
use crate::net_bluejekyll::{net_bluejekyll_NativeClass, net_bluejekyll_NativeClassClass};

mod net_bluejekyll {
    include!(concat!(env!("OUT_DIR"), "/generated_jaffi.rs"));
}

impl<'j> net_bluejekyll::NativeClassRs<'j> for NativeClassRsImpl<'j> {
    // implement methods here
}
```

The file is generated from a Java class file that has native interfaces defined, for example:

```java
public class NativeClass extends ParentClass {
    // basic test
    public static native void void_void();
}
```

## Using the generated code

### Generate docs

Discovery of all the functions available is easily done with `cargo doc --document-private-items --open`.

### the \*RsImpl implements the \*Rs trait

This is one of the primary benefits of the library. It will generate type-safe bindings from the Java and require that the `*RsImpl` type implements all of the required native functions. Additionally, it properly converts the types between the Rust calls and the Java (at the time of this writing only primitive types have been tested). The compiler will helpfully fail until all the native interfaces have been implemented.

Example from the `integration_tests`, these Java native interfaces:

```java
public class NativePrimitives extends ParentClass {
    // basic test
    public static native void voidVoid();

    // a parameter
    public static native void voidLong(long foo);

    // ...
}
```

are then generated into a trait in rust:

```rust
pub trait NativePrimitivesRs<'j> {
    fn from_env(env: JNIEnv<'j>) -> Self;
    fn void_void(&self, class: NetBluejekyllNativePrimitivesClass<'j>);
    fn void_long_j(
        &self, 
        class: NetBluejekyllNativePrimitivesClass<'j>, 
        arg0: i64
    );

    // ...
}
```

where the env should be captured in the `from_env` which constructs the Rust type. Then the bindings are called from the assocatied C FFI function (there's no need to use these functions directly):

```rust
#[no_mangle]
pub extern "system" fn Java_net_bluejekyll_NativePrimitives_voidVoid<'j>(
    env: JNIEnv<'j>, 
    class: NetBluejekyllNativePrimitivesClass<'j>
) -> JavaVoid {
    // ...
}

#[no_mangle]
pub extern "system" fn Java_net_bluejekyll_NativePrimitives_voidLong__J<'j>(
    env: JNIEnv<'j>, 
    class: NetBluejekyllNativePrimitivesClass<'j>, 
    arg0: JavaLong
) -> JavaVoid {
    // ...
}
```

All the calls into rust are properly wrapped in panic handlers and will convert Errors into Exceptions (and vice versa) as necessary. See `Exceptions, Errors, and Panics` below.

### Wrappers for specified classes, i.e. calling back to Java

In all function invocations a `this` parameter is available for calling back to any `public` methods on the class. For static methods, the `this` is bound to a `*Class` generated type. Both the static method invocations and object method invocations share a trait that is implemented for both that exposes all `public static` methods as well.

Example from the `inegration_tests`:

```rust
    /// A constructor method wrapped and then the type returned from Rust to Java
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
```

### Super class support

If specified in the `build.rs` as the `classes_to_wrap` option, any super classes will also be wrapped, in addition to those specified, any classes that appear as arguments will (and are found in the classpath) will have wrappers generated. To get access to a super class or interface and it's methods, simply call `this.as_{package}_{Class}()` on and object (won't work on `static native` methods), and then that super classes methods can be called on the object.

Example from the `integration_tests`:

```rust
    fn call_dad_native(
        &self,
        this: net_bluejekyll::NetBluejekyllNativePrimitives<'j>,
        arg0: i32,
    ) -> i32 {
        println!("call_dad_native with {arg0}");

        let parent = this.as_net_bluejekyll_parent_class();
        parent.call_1dad(self.env, arg0)
    }
```

### Exceptions, Errors, and Panics

Any panics in the Rust code will be caught via `std::panic::set_hook` and `std::panic::catch_unwind`. The panic hook will create an `RuntimeException` in Java (based on the `PanicInfo` in Rust). The `catch_unwind` will catch the panic and ensure that a proper default of null value is returned from the native method, this value is essentially useless as the Exception should shortcircuit the return in Java.

The type signature in the Java classfile will be evaluated for Exceptions. An enum type in Rust will be generated that contains the various exception types that can be either thrown in Java or can be used by jaffi to auto translate a Rust error into an Exception in Java. The interfaces generated in Rust abstract these conversations out of the method interfaces. If a method does not list Exceptions in it's `throws` section yet those exceptions need to be caught, this can be done manually via the `JNIEnv` that is available to the generated methods.

Examples from the `integration_tests`:

```rust
    fn throws_something(
        &self,
        _this: NetBluejekyllExceptions<'j>,
    ) -> Result<(), Error<SomethingExceptionErr>> {
        Err(Error::new(
            SomethingExceptionErr::SomethingException(SomethingException),
            "Test Message",
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

    /// this panic will generate an RuntimeException in Java.
    fn panics_are_runtime_exceptions(&self, _this: NetBluejekyllExceptions<'j>) {
        panic!("{}", "Panics are safe".to_string());
    }
```

## What's next?

I built this to help with a different project I've been working on where I was constantly tracking down bugs in the FFI bindings when variables changed and the signatures weren't properly updated. This should help reduce those simple errors and improve productivity when working with JNI and Rust.

## Thank you

This project makes heavy usage of these crates, thank you to everyone who's worked on them:

- `cafebabe` - a Java class file reader
- `jni` - state of the art JNI support in Rust
- `tinytemplate` - for all the Rust code generation

Thank you!

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
