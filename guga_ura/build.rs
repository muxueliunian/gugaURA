fn main() {
    #[cfg(target_os = "windows")]
    {
        // 🔑 关键：链接导出定义文件，确保UnityMain符号被导出
        let def_path =
            std::fs::canonicalize("src/proxy/exports.def").expect("exports.def not found");
        println!("cargo:rustc-cdylib-link-arg=/DEF:{}", def_path.display());

        // Windows资源编译
        let mut res = winresource::WindowsResource::new();
        res.set("ProductName", "GugaURA");
        res.set("FileDescription", "Uma Musume Data Capture Tool");
        res.compile().expect("Failed to compile resources");
    }
}
