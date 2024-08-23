use std::env;
use std::path::Path;
use std::path::PathBuf;

fn main() {
    slint_build::compile("ui/appwindow.slint").unwrap();

    // find msvc compiler
    let msvc = cc::windows_registry::find("x86_64-pc-windows-msvc", "cl.exe").expect("Could not find cl.exe.")
        .current_dir(env::current_dir().unwrap())
        .arg("key-hooker/emoji-key-hooker.c")
        .arg(format!("/Fe{}/emoji-key-hooker.dll", env::var("OUT_DIR").unwrap()))
        .arg("/LD")
        .output().expect("Could not compile emoji-key-hooker.dll.");
    if !msvc.status.success() {
        panic!("Could not compile emoji-key-hooker.dll. {:?}", msvc);
    }
    // Specify where to find the .lib and .dll files
    println!("cargo:rustc-link-search=native={}", env::var("OUT_DIR").unwrap());

    // Link with the necessary libraries (replace `your_lib` with the actual lib name)
    println!("cargo:rustc-link-lib=dylib=emoji-key-hooker");

    println!("cargo:rerun-if-changed=key-hooker\\emoji-key-hooker.c");
    println!("cargo:rerun-if-changed=build.rs");
}
