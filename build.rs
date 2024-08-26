fn main() {
    slint_build::compile("ui/emoji-picker.slint").unwrap();
    if cfg!(target_os = "windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/ico.ico");
        res.set_icon_with_id("assets/ico.ico", "tray-icon"); // TODO: see if this is necessary
        if let Err(e) = res.compile() {
            println!("cargo:warning=Failed to compile Windows resources: {}", e);
        }
    }
}
