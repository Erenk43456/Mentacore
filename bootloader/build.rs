use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());

    let asm_dir = manifest_dir.join("asm");
    let out_dir =
        PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let source = asm_dir.join("handoff.s");
    let object = out_dir.join("handoff.o");

    println!(
        "cargo:rerun-if-changed={}",
        source.display()
    );

    let status = Command::new("nasm")
        .args([
            "-f",
            "win64",
            source.to_str().unwrap(),
            "-o",
            object.to_str().unwrap(),
        ])
        .status()
        .expect("failed to execute NASM");

    assert!(
        status.success(),
        "NASM failed to assemble handoff.s"
    );

    println!(
        "cargo:rustc-link-arg={}",
        object.display()
    );
}