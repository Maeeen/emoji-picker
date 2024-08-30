use rustc_version::{version, Version};

fn main() {
    // It is MSVC only…
    println!("cargo:rustc-link-arg=/section:.shared,RWS");

    // If compiling for Windows 7, we need to link against msvcrt.dll
    if let Ok(version) = version() {
        if version < Version::parse("1.76.0").unwrap() {
            println!("cargo:rustc-cfg=rust_version_lt_1_76");
        }
    }
}