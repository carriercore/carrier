fn main() {
    cc::Build::new()
        .file("src/init.c")
        .compile("init");
}