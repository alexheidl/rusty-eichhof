// On Linux machines without the ALSA development package, the linker can't find
// `libasound.so`. Point it at a private link folder holding a copy-free symlink to
// the runtime library `libasound.so.2` when that is all the system has.
fn main() {
    let target = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target != "linux" {
        return;
    }
    let candidates = [
        "/usr/lib/x86_64-linux-gnu/libasound.so",
        "/usr/lib64/libasound.so",
        "/usr/lib/libasound.so",
        "/usr/lib/aarch64-linux-gnu/libasound.so",
    ];
    if candidates.iter().any(|p| std::path::Path::new(p).exists()) {
        return;
    }
    let runtime = [
        "/usr/lib/x86_64-linux-gnu/libasound.so.2",
        "/usr/lib64/libasound.so.2",
        "/usr/lib/libasound.so.2",
        "/usr/lib/aarch64-linux-gnu/libasound.so.2",
    ];
    if let Some(lib) = runtime.iter().find(|p| std::path::Path::new(p).exists()) {
        let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("linkshim");
        let _ = std::fs::create_dir_all(&out);
        let link = out.join("libasound.so");
        let _ = std::fs::remove_file(&link);
        #[cfg(unix)]
        {
            let _ = std::os::unix::fs::symlink(lib, &link);
        }
        println!("cargo:rustc-link-search=native={}", out.display());
    }
}
