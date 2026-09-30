use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // Release uses LLVM, which already links this archive. Debug uses Cranelift,
    // which records the search path but drops `rustc-link-lib=static`.
    if std::env::var("PROFILE").ok().as_deref() != Some("debug") {
        return;
    }
    let Ok(out_dir) = std::env::var("OUT_DIR") else {
        return;
    };
    let Some(build_root) = aws_lc_build_root(Path::new(&out_dir)) else {
        println!("cargo:warning=aws-lc build directory not found; debug link may fail");
        return;
    };
    let Some(lib) = newest_crypto_lib(&build_root.join("aws-lc-sys")) else {
        println!("cargo:warning=aws-lc static library not found; debug link may fail");
        return;
    };
    println!("cargo:rerun-if-changed={}", lib.display());
    // A raw linker input is still forwarded. `rustc-link-lib=static` is not.
    println!("cargo:rustc-link-arg={}", lib.display());
}

fn aws_lc_build_root(out_dir: &Path) -> Option<&Path> {
    let mut cur = out_dir;
    for _ in 0..8 {
        cur = cur.parent()?;
        if cur.join("aws-lc-sys").is_dir() {
            return Some(cur);
        }
    }
    None
}

fn newest_crypto_lib(root: &Path) -> Option<PathBuf> {
    let mut best: Option<(SystemTime, PathBuf)> = None;
    for entry in fs::read_dir(root).ok()?.flatten() {
        let out = entry.path().join("out");
        let Some(files) = fs::read_dir(&out).ok() else {
            continue;
        };
        for file in files.flatten() {
            let path = file.path();
            let Some(fname) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !(fname.starts_with("libaws_lc_") && fname.ends_with("_crypto.a")) {
                continue;
            }
            let Ok(modified) = file.metadata().and_then(|m| m.modified()) else {
                continue;
            };
            if best.as_ref().is_none_or(|(t, _)| modified > *t) {
                best = Some((modified, path));
            }
        }
    }
    best.map(|(_, path)| path)
}
