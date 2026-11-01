use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=ZNN_VERSION");
    let version = std::env::var("ZNN_VERSION")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
    println!("cargo:rustc-env=ZNN_VERSION={version}");

    android_page_alignment();
    bpf_object();
    mazoku_key();
}

fn mazoku_key() {
    const SIGNATURE_LEN: usize = 64;
    const BLOB_LEN: usize = SIGNATURE_LEN + 32;

    let source = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"))
        .join("../module/src/mazoku");
    let destination =
        PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set by cargo")).join("mazoku.rs");

    println!("cargo:rerun-if-changed={}", source.display());

    let blob = fs::read(&source)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", source.display()));
    assert!(
        blob.len() == BLOB_LEN,
        "{} must be a {BLOB_LEN}-byte signed blob, found {} bytes",
        source.display(),
        blob.len()
    );

    let mut generated = String::from("pub const ORG_PUBLIC_KEY: [u8; 32] = [\n");
    for chunk in blob[SIGNATURE_LEN..].chunks(8) {
        generated.push_str("    ");
        for (index, byte) in chunk.iter().enumerate() {
            if index > 0 {
                generated.push(' ');
            }
            generated.push_str(&byte.to_string());
            generated.push(',');
        }
        generated.push('\n');
    }
    generated.push_str("];\n");

    fs::write(&destination, generated)
        .unwrap_or_else(|error| panic!("cannot write {}: {error}", destination.display()));
}

fn android_page_alignment() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("android") {
        return;
    }
    println!("cargo:rustc-link-arg-bins=-Wl,-z,max-page-size=16384");
    println!("cargo:rustc-link-arg-bins=-Wl,-z,common-page-size=16384");
}

fn bpf_object() {
    let source = PathBuf::from("bpf/exec.bpf.c");
    let committed = PathBuf::from("bpf/exec.bpf.o");
    let destination = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set by cargo"))
        .join("exec.bpf.o");

    println!("cargo:rerun-if-changed={}", source.display());
    println!("cargo:rerun-if-changed={}", committed.display());
    println!("cargo:rerun-if-env-changed=BPF_CLANG");

    let fall_back = |reason: &str| {
        println!(
            "cargo:warning={reason}: embedding the committed {}",
            committed.display()
        );
        fs::copy(&committed, &destination)
            .unwrap_or_else(|error| panic!("cannot use {}: {error}", committed.display()));
    };

    let Some(clang) = find_bpf_clang() else {
        fall_back("no clang with a BPF backend found");
        return;
    };

    let compiled = Command::new(&clang)
        .args(["-O2", "-g", "-target", "bpf", "-c"])
        .arg(&source)
        .arg("-o")
        .arg(&destination)
        .status();
    match compiled {
        Ok(status) if status.success() => {}
        other => fall_back(&format!(
            "{clang} failed to compile the BPF source ({other:?})"
        )),
    }
}

fn find_bpf_clang() -> Option<String> {
    if let Ok(explicit) = std::env::var("BPF_CLANG")
        && !explicit.trim().is_empty()
    {
        return Some(explicit);
    }
    for candidate in [
        "clang", "clang-21", "clang-20", "clang-19", "clang-18", "clang-17", "clang-16",
    ] {
        let Ok(targets) = Command::new(candidate).args(["-print-targets"]).output() else {
            continue;
        };
        if !targets.status.success() {
            continue;
        }
        if String::from_utf8_lossy(&targets.stdout).contains("bpf") {
            return Some(candidate.to_owned());
        }
    }
    None
}
