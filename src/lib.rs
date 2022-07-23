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
use template::{
    Arg, ClassFfi, Function, JniAbi, JniType, Object, ObjectType, Return, RustFfi, RustTypeName,
};

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use cafebabe::{ClassFile, MethodAccessFlags, MethodInfo, ParseOptions};
use typed_builder::TypedBuilder;

use crate::template::{BaseJniTy, FuncAbi, JavaDesc};

pub use jaffi_support;

/// A utility for generating Rust FFI implementations from Java class files that contain `native` functions.
#[derive(TypedBuilder)]
pub struct Jaffi<'a> {
    /// Used like ClassPath in Java, defaults to `.` if empty
    classpath: Vec<Cow<'a, Path>>,
    /// generated source target path for the Rust, probably something in `target/`, defaults to `.`
    ///
    /// Implementation files will be the java class name converted to a rust module name with `_` replacing the `.`
    output_dir: Option<Cow<'a, Path>>,
    /// List of classes with native methods (specified as java class names, i.e. `java.lang.Object`) to generate bindings for
    native_classes: Vec<Cow<'a, str>>,
    /// List of classes that wrappers will be generated for
    #[builder(default=Vec::new())]
    classes_to_wrap: Vec<Cow<'a, str>>,
}

impl<'a> Jaffi<'a> {
    /// Generate the rust FFI files based on the configured inputs
    pub fn generate(&self) -> Result<(), Error> {
        // shared buffer for classes that are read into memory
        let mut class_ffis = Vec::<ClassFfi>::new();
        let mut argument_types = HashSet::<JavaDesc>::new();
        argument_types.extend(
            self.classes_to_wrap
                .iter()
                .map(|s| JavaDesc::from(s as &str)),
        );

        // create all the classes
        let native_classes = self
            .native_classes
            .iter()
            .map(|s| JavaDesc::from(s as &str))
            .collect::<Vec<_>>();
        let classes = self.search_classpath(&native_classes)?;

        let mut class_buf = Vec::<u8>::new();
        for class in classes {
            let class_file = self.read_class(&class, &mut class_buf)?;

            let (class_ffi, objects) = self.generate_native_impls(class_file)?;
            class_ffis.extend(class_ffi);
            argument_types.extend(objects);
        }

        // create the wrapper types
        let objects = self.generate_support_types(argument_types)?;

        // render the file
        let output_dir = &Cow::Borrowed(Path::new("."));
        let output_dir = if let Some(ref dir) = self.output_dir {
            dir
        } else {
            output_dir
        };

        // we always generate to the same file name
        let rust_file = PathBuf::from(output_dir.as_ref())
            .join("generated_jaffi")
            .with_extension("rs");

        let ffi_tokens = template::generate_java_ffi(objects, class_ffis);
        let rendered = ffi_tokens.to_string();

        let mut rust_file = File::create(rust_file)?;
        rust_file.write_all(rendered.as_bytes())?;

        Ok(())
    }

    fn search_classpath(&self, classes: &[JavaDesc]) -> Result<Vec<PathBuf>, Error> {
        let default_classpath = &[Cow::Borrowed(Path::new("."))] as &[_];
        let classpath = if self.classpath.is_empty() {
            default_classpath
        } else {
            self.classpath.as_slice()
        };

        // create all the classes
        let mut found_classes = Vec::new();
        for class in classes {
            let class = class_to_path(class.as_str());

            let mut found_class = false;
            'search: for classpath in classpath {
                if classpath.is_dir() && lookup_from_path(&*classpath, &class) {
                    found_class = true;
                    found_classes.push(classpath.join(&class));
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
        }

        Ok(found_classes)
    }

    /// # Arguments
    /// * `path` - path to the classfile
    /// * `class_buf` - temporary buffer to use for the parsing, this will be cleared before use
    fn read_class(&self, path: &Path, class_buf: &'a mut Vec<u8>) -> Result<ClassFile<'a>, Error> {
        class_buf.clear();

        if !path.exists() {
            return Err(Error::from(format!("file not found: {}", path.display())));
        }

        let mut file = File::open(path)?;
        file.read_to_end(class_buf)?;

        let mut opts = ParseOptions::default();
        opts.parse_bytecode(false);
        cafebabe::parse_class_with_options(class_buf, &opts).map_err(Into::into)
    }

    /// Returns list of Support types needed as interfaces in the ABI interfaces
    fn generate_native_impls(
        &self,
        class_file: ClassFile<'_>,
    ) -> Result<(Option<ClassFfi>, HashSet<JavaDesc>), Error> {
        eprintln!(
            "Generating native implementations for: {}, version: {}.{}",
            class_file.this_class, class_file.major_version, class_file.minor_version
        );

        let native_methods = class_file
            .methods
            .iter()
            .filter(|method_info| method_info.access_flags.contains(MethodAccessFlags::NATIVE))
            .collect::<Vec<_>>();

        // do nothing, no native methods found...
        if native_methods.is_empty() {
            return Ok((None, HashSet::new()));
        }

        // get all the function information
        let (functions, argument_objects) =
            self.extract_function_info(&class_file, native_methods)?;

        let trait_name = Path::new(&*class_file.this_class)
            .file_name()
            .expect("no file component")
            .to_string_lossy()
            .to_string()
            + "Rs";
        let trait_impl = format!("{trait_name}Impl");

        // build up the rendering information.
        let class_ffi = template::ClassFfi {
            class_name: class_file.this_class.to_string(),
            type_name: RustTypeName::from(class_file.this_class.to_string()),
            trait_name,
            trait_impl,
            functions,
        };

        Ok((Some(class_ffi), argument_objects))
    }

    fn generate_support_types(&self, mut types: HashSet<JavaDesc>) -> Result<Vec<Object>, Error> {
        let mut search_object_types = types.iter().cloned().collect::<Vec<_>>();
        let mut objects = Vec::<Object>::with_capacity(search_object_types.len());
        let mut already_generated = HashSet::<JavaDesc>::new();
        let classes_to_wrap = self
            .classes_to_wrap
            .iter()
            .chain(self.native_classes.iter())
            .map(|s| JavaDesc::from(&**s))
            .collect::<HashSet<_>>();

        let mut class_buf = Vec::<u8>::new();
        while let Some(object_desc) = search_object_types.pop() {
            if already_generated.contains(&object_desc) {
                continue;
            } else {
                already_generated.insert(object_desc.clone());
            }

            let wrap_methods = classes_to_wrap.contains(&object_desc);
            let mut object = Object::from(ObjectType::from(&object_desc));

            if wrap_methods {
                let class = self.search_classpath(&[object_desc.clone()])?;

                for obj_path in class {
                    let class_file = self.read_class(&obj_path, &mut class_buf)?;

                    // collect public and non-native methods
                    let public_methods = class_file
                        .methods
                        .iter()
                        .filter(|method_info| {
                            !method_info.access_flags.contains(MethodAccessFlags::NATIVE)
                                && method_info.access_flags.contains(MethodAccessFlags::PUBLIC)
                        })
                        .collect::<Vec<_>>();

                    let (functions, new_types) =
                        self.extract_function_info(&class_file, public_methods)?;

                    // add any types to generate that we haven't seen before
                    for ty in new_types {
                        if !types.contains(&ty) {
                            types.insert(ty.clone());
                            search_object_types.push(ty);
                        }
                    }

                    // find all interfaces this type supports
                    for interface in class_file
                        .super_class
                        .iter()
                        .chain(class_file.interfaces.iter())
                    {
                        // we're only going to generate types that have been explicitly been asked for,
                        //   or those that appear in args, that's what's in the hash_map. So unlike above
                        //   we won't add to the types hashmap
                        let interface = JavaDesc::from(interface as &str);
                        if types.contains(&interface) {
                            search_object_types.push(interface.clone());
                            object.interfaces.push(RustTypeName::from(interface));
                        }
                    }

                    // add the function to the methods in the object
                    object.methods.extend(functions.into_iter());
                }
            }
            objects.push(object);
        }

        Ok(objects)
    }

    /// # Return
    ///
    /// On success, the discovered Functions are returned in a Vec, and a HashSet of additional types to support function calls
    fn extract_function_info(
        &self,
        class_file: &ClassFile<'_>,
        methods: Vec<&MethodInfo<'_>>,
    ) -> Result<(Vec<Function>, HashSet<JavaDesc>), Error> {
        eprintln!(
            "Extracting function information for: {}, version: {}.{}",
            class_file.this_class, class_file.major_version, class_file.minor_version
        );

        let method_names = methods.iter().fold(HashMap::new(), |mut map, method| {
            // TODO: figure out how to dedup this code...
            let method_name = if method.name == "<init>" {
                Cow::from(format!("new_{}", class_file.this_class))
            } else {
                method.name.clone()
            };

            *map.entry(method_name).or_insert(0) += 1;
            map
        });

        // All objects needed to support calls into JNI from Java
        let mut argument_objects = HashSet::<JavaDesc>::new();

        // This class will always be necessary
        let this_class_desc = JavaDesc::from(&class_file.this_class as &str);
        let this_class = ObjectType::Object(this_class_desc.clone());
        argument_objects.insert(this_class_desc.clone());

        // build up the function definitions
        let mut functions = Vec::new();
        for method in methods {
            let descriptor = JavaDesc::from(method.descriptor.to_string());

            let is_constructor = method.name == "<init>";
            let is_static = method.access_flags.contains(MethodAccessFlags::STATIC);

            let object_java_desc = this_class_desc.clone();
            let class_ffi_name = this_class.to_jni_class_name();
            let object_ffi_name = this_class.to_jni_type_name();

            let arg_types = method
                .descriptor
                .parameters
                .iter()
                .map(JniType::from_java)
                .collect::<Vec<_>>();

            let result = if !is_constructor {
                Return::from_java(&method.descriptor.result)
            } else {
                Return::Val(JniType::Ty(BaseJniTy::Jobject(ObjectType::from(
                    object_java_desc.clone(),
                ))))
            };

            // Collect the Objects that need to be supported for returns and argument lists
            for ty in arg_types.iter().chain(result.as_val().into_iter()) {
                match ty {
                    JniType::Ty(BaseJniTy::Jobject(ObjectType::Object(obj))) => {
                        argument_objects.insert(obj.clone())
                    }
                    _ => continue,
                };
            }

            let arguments = arg_types
                .into_iter()
                .enumerate()
                .map(move |(i, ty)| Arg {
                    name: format!("arg{i}"),
                    ty: ty.to_jni_type_name(),
                    rs_ty: ty.to_rs_type_name(),
                })
                .collect();

            let method_name = if is_constructor {
                Cow::from(format!("new_{}", class_file.this_class))
            } else {
                method.name.clone()
            };
            let fn_ffi_name = if *method_names
                .get(&method_name)
                .expect("should have been added above")
                > 1
            {
                // need to long abi name
                FuncAbi::from(JniAbi::from(method_name)).with_descriptor(&descriptor)
            } else {
                // short is ok (faster lookup in dynamic linking)
                FuncAbi::from(JniAbi::from(method_name))
            };
            let fn_export_ffi_name = fn_ffi_name.with_class(&this_class.to_jni_type_name());

            let function = Function {
                name: method.name.to_string(),
                object_java_desc,
                fn_export_ffi_name,
                class_ffi_name,
                object_ffi_name,
                fn_ffi_name,
                signature: descriptor,
                is_constructor,
                is_static,
                arguments,
                result: result.to_jni_type_name(),
                rs_result: result.to_rs_type_name(),
            };

            functions.push(function);
        }

        Ok((functions, argument_objects))
    }
}

fn class_to_path(name: &str) -> PathBuf {
    let name = name.replace('.', "/");
    PathBuf::from(name).with_extension("class")
}

fn lookup_from_path(classpath: &Path, class: &Path) -> bool {
    let path = classpath.join(class);

    path.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_name() {
        assert_eq!(JniAbi::from("p.q.r.A").to_string(), "p_q_r_A");
        assert_eq!(
            FuncAbi::from(JniAbi::from("f"))
                .with_descriptor(&JavaDesc::from("(ILjava.lang.String;)D"))
                .with_class(&RustTypeName::from("p.q.r.A"))
                .to_string(),
            "Java_p_q_r_A_f__ILjava_lang_String_2"
        );
    }

    #[test]
    fn test_escape_name_unicode() {
        assert_eq!(JniAbi::from("i❤'🦀").to_string(), "i_02764_027_01f980");
    }
}
