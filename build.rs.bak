use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=boot.asm");
    let status = Command::new("nasm")
        .args(&["-f", "elf64", "-o", "boot.o", "boot.asm"])
        .status()
        .expect("failed to assemble boot.asm");
    assert!(status.success());

    println!("cargo:rustc-link-arg=boot.o");
    println!("cargo:rustc-link-arg=-Tlinker.ld");
    println!("cargo:rustc-link-arg=-nostdlib");
}
