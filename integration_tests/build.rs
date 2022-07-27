use std::{
    borrow::Cow,
    error::Error,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use jaffi::Jaffi;

fn class_path() -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set")).join("java/classes")
}

fn find_java_files() -> Vec<PathBuf> {
    let search_paths: Vec<Cow<'_, Path>> = vec![Cow::from(PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"),
    ))];

    find_files(search_paths, "java")
}

fn find_files(mut search_paths: Vec<Cow<'_, Path>>, extension: &str) -> Vec<PathBuf> {
    let mut java_files = Vec::<PathBuf>::new();

    while let Some(path) = search_paths.pop() {
        if !path.is_dir() {
            continue;
        }

        for dir_entry in path.read_dir().expect("could not read directory") {
            let dir_entry = dir_entry.expect("could not open directory");
            let path = dir_entry.path();
            match dir_entry.file_type().expect("could not read file") {
                e if e.is_dir() => {
                    search_paths.push(path.into());
                }
                e if e.is_file() => {
                    if path
                        .extension()
                        .map(|ext| ext == extension)
                        .unwrap_or(false)
                    {
                        java_files.push(path);
                    }
                }
                _ => (),
            }
        }
    }

    java_files
}

fn compile_java() {
    let java_files = find_java_files()
        .into_iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();

    // create the target dir
    let class_path = class_path().display().to_string();
    std::fs::create_dir_all(&class_path).expect("failed to create dir");

    let output = Command::new("javac")
        .arg("-version")
        .output()
        .expect("failed to execute process");
    std::io::stderr().write_all(&output.stdout).unwrap();
    std::io::stderr().write_all(&output.stderr).unwrap();

    let mut cmd = Command::new("javac");
    cmd.arg("-d")
        .arg(&class_path)
        .arg("-h")
        .arg(class_path)
        .args(java_files);

    eprintln!("javac: {cmd:?}");

    let output = cmd.output().expect("Failed to execute command");

    std::io::stderr().write_all(&output.stdout).unwrap();
    std::io::stderr().write_all(&output.stderr).unwrap();
    eprintln!("java compilations status: {}", output.status);

    if !output.status.success() {
        panic!("javac failed");
    }
    eprintln!("successfully compiled java");
}

fn main() -> Result<(), Box<dyn Error>> {
    // only need this if you need to compile the java, this is needed for the integration tests...
    compile_java();

    let class_path = class_path();
    let classes = vec![
        Cow::from("net.bluejekyll.NativePrimitives"),
        Cow::from("net.bluejekyll.NativeStrings"),
        Cow::from("net.bluejekyll.NativeArrays"),
        Cow::from("net.bluejekyll.RustKeywords"),
        Cow::from("net.bluejekyll.Exceptions"),
    ];
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
