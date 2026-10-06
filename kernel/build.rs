use std::{env, fs, path::Path};

fn main() {
    let target = env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target != "none" {
        return;
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);
    let linker_script = out_dir.join("linker.ld");

    let profile_dir = out_dir
        .ancestors()
        .find(|path| path.file_name().is_some_and(|name| name == "build"))
        .and_then(Path::parent)
        .expect("OUT_DIR should be inside Cargo's build directory");

    let map_file = profile_dir.join("output.map");

    fs::copy("linker.ld", &linker_script).unwrap();
    println!("cargo:rustc-link-arg=-T");
    println!("cargo:rustc-link-arg={}", linker_script.display());
    println!("cargo:rustc-link-arg=-Map={}", map_file.display());
    println!("cargo:rerun-if-changed=linker.ld");
}
