fn main() {
    cc::Build::new().file("src/boot.S").compile("boot");
}
