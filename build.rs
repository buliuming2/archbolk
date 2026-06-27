fn main() {
    cc::Build::new()
        .file("src/arch/x86_64/boot.S")
        .compile("boot");
}
