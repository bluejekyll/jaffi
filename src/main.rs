use std::{borrow::Cow, error::Error, fs::File, io::Read, path::Path};

use jaffi::Jaffi;

fn main() -> Result<(), Box<dyn Error>> {
    let classpath = Path::new("/Users/benjaminfry/Development/rust/wasmtime-java/target/classes");
    let class = "net.bluejekyll.wasmtime.WasmEngine";
    let output_dir = Path::new("target");

    let jaffi = Jaffi::builder()
        .classes(vec![Cow::from(class)])
        .classpath(vec![Cow::from(classpath)])
        .output_dir(Some(Cow::from(output_dir)))
        .build();

    jaffi.generate()?;

    Ok(())
}
