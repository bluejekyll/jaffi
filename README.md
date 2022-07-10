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
jaffi = { version = "0.1.0", git = "git@github.com:bluejekyll/jaffi.git", branch = "main" }

[dependencies]
jaffi_support = { version = "0.1.0", git = "git@github.com:bluejekyll/jaffi.git", branch = "main" }
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

### Wrappers for specified classes, i.e. calling back to Java

In all function invocations a `this` parameter is available for calling back to any `public` methods on the class. For static methods, the `this` is bound to a `*Class` generated type. Both the static method invocations and object method invocations share a trait that is implemented for both that exposes all `public static` methods as well.

### Super class support

If specified in the `build.rs` as the `classes_to_wrap` option, any super classes will also be wrapped, in addition to those specified, any classes that appear as arguments will (and are found in the classpath) will have wrappers generated. To get access to a super class or interface and it's methods, simply call `this.as_{package}_{Class}()` on and object (won't work on `static native` methods), and then that super classes methods can be called on the object.

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
