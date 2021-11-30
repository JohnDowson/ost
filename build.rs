fn main() {
    cc::Build::new()
        .file("src/thread.S")
        .flag("-c")
        .compile("thread.o")
}
