use std::{env, fs, path::Path};

fn main() {
    cc::Build::new().file("src/boot.S").compile("boot");

    {
        let out_dir = env::var("OUT_DIR").unwrap();
        let dest = Path::new(&out_dir).join("linker.ld");
        fs::copy("linker.ld", &dest).unwrap();
        println!("cargo:rerun-if-changed=linker.ld");
    }
}
