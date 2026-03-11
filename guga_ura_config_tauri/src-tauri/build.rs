fn main() {
    println!("cargo:rerun-if-changed=tauri.conf.json");
    println!("cargo:rerun-if-changed=capabilities");
    println!("cargo:rerun-if-changed=app.manifest");
    println!("cargo:rerun-if-changed=icons/icon.ico");

    #[cfg(target_os = "windows")]
    {
        let manifest_path = std::fs::canonicalize("app.manifest").expect("app.manifest not found");
        let icon_path = std::fs::canonicalize("icons/icon.ico").expect("icons/icon.ico not found");
        let mut res = winresource::WindowsResource::new();
        res.set_manifest_file(manifest_path.to_string_lossy().as_ref());
        res.set_icon(icon_path.to_string_lossy().as_ref());
        res.compile().expect("编译 Windows 资源失败");
    }
}
