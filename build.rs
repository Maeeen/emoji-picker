use std::{fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    slint_build::compile("ui/emoji-picker.slint").unwrap();

    if cfg!(target_os = "windows") {
        // Another workaround to make it link dynamically
        let lib_path: PathBuf = ["target", std::env::var("PROFILE").unwrap().as_ref(), "deps"].iter().collect();
        let dll_name = "emoji_picker_hooker.dll";

        println!("cargo:rustc-link-search=native={}", lib_path.display());
        println!("cargo:rustc-link-lib={dll_name}");

        // Copy the DLL to the output directory so that it can be found at runtime

        let src = lib_path.join(dll_name);
        let dst: PathBuf = ["target", std::env::var("PROFILE").unwrap().as_ref(), dll_name].iter().collect();
        fs::copy(src, dst).expect("Failed to copy DLL");

        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/ico.ico");
        res.set_icon_with_id("assets/ico.ico", "tray-icon");
        if let Err(e) = res.compile() {
            println!("cargo:warning=Failed to compile Windows resources: {}", e);
        }
    }
}
