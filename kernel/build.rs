use std::{env, fs, path::Path};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let linker_script = Path::new(&out_dir).join("linker.ld");
    fs::copy("linker.ld", &linker_script).unwrap();
    println!("cargo:rustc-link-arg=-T");
    println!("cargo:rustc-link-arg={}", linker_script.display());
    println!("cargo:rerun-if-changed=linker.ld");
}
