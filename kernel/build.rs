use std::env;
use std::path::PathBuf;
use std::process::Command;

fn assemble(source: &PathBuf, object: &PathBuf) {
    let status = Command::new("nasm")
        .args([
            "-f",
            "elf64",
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

    let sources = [
        ("interrupts.s", "interrupts.o"),
        ("cpu.s", "cpu.o"),
    ];

    for (source_name, object_name) in sources {
        let source = asm_dir.join(source_name);
        let object = out_dir.join(object_name);

        println!(
            "cargo:rerun-if-changed={}",
            source.display()
        );

        assemble(&source, &object);

        println!(
            "cargo:rustc-link-arg={}",
            object.display()
        );
    }
}
