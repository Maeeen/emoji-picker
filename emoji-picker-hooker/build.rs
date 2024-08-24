fn main() {
    // It is MSVC only…
    println!("cargo:rustc-link-arg=/section:.shared,RWS");
}