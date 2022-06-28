// Copyright 2022 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! A generator for Rust based on Java class files to help define the FFI bindings with strongly declared types.

#![warn(
    clippy::default_trait_access,
    clippy::dbg_macro,
    clippy::print_stdout,
    clippy::unimplemented,
    missing_copy_implementations,
    missing_docs,
    non_snake_case,
    non_upper_case_globals,
    rust_2018_idioms,
    unreachable_pub
)]

mod error;
mod template;

pub use error::{Error, ErrorKind};
use template::{Arg, Function, JniType, Object, ObjectType, Return};
use tinytemplate::TinyTemplate;

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use cafebabe::{MethodAccessFlags, ParseOptions};
use typed_builder::TypedBuilder;

use crate::template::BaseJniTy;

/// A utility for generating Rust FFI implementations from Java class files that contain `native` functions.
#[derive(TypedBuilder)]
pub struct Jaffi<'a> {
    /// Used like ClassPath in Java, defaults to `.` if empty
    classpath: Vec<Cow<'a, Path>>,
    /// generated source target path for the Rust, probably something in `target/`, defaults to `.`
    ///
    /// Implementation files will be the java class name converted to a rust module name with `_` replacing the `.`
    output_dir: Option<Cow<'a, Path>>,
    /// List of classes (specified as java class names, i.e. `java.lang.Object`) to generate bindings for
    classes: Vec<Cow<'a, str>>,
}

impl<'a> Jaffi<'a> {
    /// Generate the rust FFI files based on the configured inputs
    pub fn generate(&self) -> Result<(), Error> {
        let template = template::new_engine()?;

        let default_classpath = &[Cow::Borrowed(Path::new("."))] as &[_];
        let classpaths = if self.classpath.is_empty() {
            default_classpath
        } else {
            self.classpath.as_slice()
        };

        // shared buffer for classes that are read into memory
        let mut class_buf = Vec::<u8>::new();
        let mut argument_types = HashSet::<ObjectType>::new();

        // create all the classes
        for class in &self.classes {
            class_buf.clear();
            let class = class_to_path(class);

            let mut found_class = false;
            'search: for classpath in classpaths {
                if classpath.is_dir() && lookup_from_path(&*classpath, &class, &mut class_buf)? {
                    found_class = true;
                    break 'search;
                } else if classpath.is_file() && classpath.extension().unwrap_or_default() == "jar"
                {
                    unimplemented!("jar files for classpath not yet supported")
                } else {
                    continue 'search;
                };
            }

            // couldn't find the class
            if !found_class {
                return Err(
                    format!("could not find class in classpath: {}", class.display()).into(),
                );
            }

            let objects = self.generate_native_impls(&class_buf, &template)?;
            argument_types.extend(objects);
        }

        // create the wrapper types
        self.generate_support_types(argument_types, &template)?;

        Ok(())
    }

    /// Returns list of Support types needed as interfaces in the ABI interfaces
    fn generate_native_impls<'b>(
        &self,
        class_bytes: &'b [u8],
        template: &TinyTemplate<'_>,
    ) -> Result<HashSet<ObjectType>, Error> {
        let output_dir = &Cow::Borrowed(Path::new("."));
        let output_dir = if let Some(ref dir) = self.output_dir {
            dir
        } else {
            output_dir
        };

        let mut opts = ParseOptions::default();
        opts.parse_bytecode(false);
        let class_file =
            cafebabe::parse_class_with_options(class_bytes, &opts).map_err(|e| e.to_string())?;

        let native_methods = class_file
            .methods
            .iter()
            .filter(|method_info| method_info.access_flags.contains(MethodAccessFlags::NATIVE))
            .collect::<Vec<_>>();

        let method_names = native_methods
            .iter()
            .fold(HashMap::new(), |mut map, method| {
                *map.entry(&method.name).or_insert(0) += 1;
                map
            });

        // All objects needed to support calls into JNI from Java
        let mut argument_objects = HashSet::<ObjectType>::new();

        // This class will always be necessary
        let this_class = ObjectType::Object(class_file.this_class.to_string());
        argument_objects.insert(this_class.clone());

        // build up the function definitions
        let mut functions = Vec::new();
        for method in native_methods {
            println!("{method:#?}");

            let descriptor = method.descriptor.to_string();

            let class_or_this = if method.access_flags.contains(MethodAccessFlags::STATIC) {
                this_class.to_jni_class_name()
            } else {
                this_class.to_jni_type_name().to_string()
            };

            let fn_name = if *method_names
                .get(&method.name)
                .expect("should have been added above")
                > 1
            {
                // need to long abi name
                method_to_long_abi_name(&class_file.this_class, &method.name, &descriptor)
            } else {
                // short is ok (faster lookup in dynamic linking)
                method_to_abi_name(&class_file.this_class, &method.name)
            };

            let arg_types = method
                .descriptor
                .parameters
                .iter()
                .map(JniType::from_java)
                .collect::<Vec<_>>();

            for ty in &arg_types {
                match ty {
                    JniType::Ty(BaseJniTy::Jobject(obj)) => argument_objects.insert(obj.clone()),
                    _ => continue,
                };
            }

            let arguments = arg_types
                .into_iter()
                .enumerate()
                .map(move |(i, ty)| Arg {
                    name: format!("arg{i}"),
                    ty: ty.to_jni_type_name(),
                })
                .collect();
            let result = Return::from_java(&method.descriptor.result);

            let function = Function {
                name: fn_name,
                signature: descriptor,
                class_or_this,
                arguments,
                result: result.to_jni_type_name(),
            };

            functions.push(function);
        }

        // build up the rendering information.
        let context = template::RustFfi {
            class_name: class_file.this_class.clone(),
            functions,
        };

        // the file name will be the full class name
        let mut rust_file = PathBuf::from(output_dir.as_ref())
            .join(escape_for_abi(&class_file.this_class))
            .with_extension("rs");

        let rendered = template.render(template::RUST_FFI, &context)?;

        let mut rust_file = File::create(rust_file)?;
        rust_file.write_all(rendered.as_bytes())?;

        Ok(argument_objects)
    }

    fn generate_support_types(
        &self,
        mut types: HashSet<ObjectType>,
        template: &TinyTemplate<'_>,
    ) -> Result<(), Error> {
        let output_dir = &Cow::Borrowed(Path::new("."));
        let output_dir = if let Some(ref dir) = self.output_dir {
            dir
        } else {
            output_dir
        };

        let context = template::RustFfiObjects {
            objects: types
                .drain()
                .filter_map(|obj| {
                    if let ObjectType::Object(_) = obj {
                        Some(Object::from(obj))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
        };
        let rendered = template.render(template::RUST_FFI_OBJ, &context)?;

        let rust_file = output_dir.join("support_types").with_extension("rs");
        let mut rust_file = File::create(rust_file)?;
        rust_file.write_all(rendered.as_bytes())?;

        Ok(())
    }
}

fn class_to_path(name: &str) -> PathBuf {
    let name = name.replace('.', "/");
    PathBuf::from(name).with_extension("class")
}

fn lookup_from_path(classpath: &Path, class: &Path, bytes: &mut Vec<u8>) -> Result<bool, Error> {
    let path = classpath.join(class);

    if !path.is_file() {
        return Ok(false);
    }

    let mut file = File::open(path)?;
    file.read_to_end(bytes)?;

    Ok(true)
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
fn method_to_abi_name(class_name: &str, method_name: &str) -> String {
    let abi_class_name = escape_for_abi(class_name);
    let abi_method_name = escape_for_abi(method_name);

    format!("Java_{abi_class_name}_{abi_method_name}")
}

fn method_to_long_abi_name(class_name: &str, method_name: &str, descriptor: &str) -> String {
    // strip the '(', ')', and return from the descriptor
    let descriptor = descriptor.strip_prefix('(').unwrap_or(descriptor);
    let descriptor = if let Some(pos) = descriptor.find(")") {
        &descriptor[..pos]
    } else {
        descriptor
    };

    let abi_method = method_to_abi_name(class_name, method_name);
    let abi_descriptor = escape_for_abi(descriptor);

    format!("{abi_method}__{abi_descriptor}")
}

fn escape_for_abi(name: &str) -> String {
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

    abi_name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_name() {
        assert_eq!(method_to_abi_name("p.q.r.A", "f"), "Java_p_q_r_A_f");
        assert_eq!(
            method_to_long_abi_name("p.q.r.A", "f", "(ILjava.lang.String;)D"),
            "Java_p_q_r_A_f__ILjava_lang_String_2"
        );
    }

    #[test]
    fn test_escape_name_unicode() {
        assert_eq!(
            method_to_abi_name("p.q.r.A", "i❤'🦀"),
            "Java_p_q_r_A_i_02764_027_01f980"
        );
    }
}
