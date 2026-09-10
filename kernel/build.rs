use std::env;
use std::path::PathBuf;
use std::process::Command;

fn assemble(source: &PathBuf, object: &PathBuf, asm_dir: &PathBuf) {
    let status = Command::new("nasm")
        .args([
            "-f",
            "elf64",
            "-I",
            &format!("{}\\", asm_dir.display()),
            source.to_str().unwrap(),
            "-o",
            object.to_str().unwrap(),
        ])
        .status()
        .expect("failed to execute NASM");

    assert!(
        status.success(),
        "NASM failed to assemble {}",
        source.display()
    );
}

fn main() {
    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());

    let asm_dir = manifest_dir.join("asm");
    let out_dir =
        PathBuf::from(env::var_os("OUT_DIR").unwrap());

    println!(
        "cargo:rerun-if-changed={}",
        asm_dir.join("common.inc").display()
    );

    let sources = [
        ("exceptions.s", "exceptions.o"),
        ("timer.s", "timer.o"),
        ("cpu.s", "cpu.o"),
    ];

    for (source_name, object_name) in sources {
        let source = asm_dir.join(source_name);
        let object = out_dir.join(object_name);

        println!(
            "cargo:rerun-if-changed={}",
            source.display()
        );

        assemble(&source, &object, &asm_dir);

        println!(
            "cargo:rustc-link-arg={}",
            object.display()
        );
    }
}