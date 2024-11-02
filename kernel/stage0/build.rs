//!
//! Chopin Kernel Stage0 build file
//!
//! This script just includes the chopin boot assembly
//!

use std::path::PathBuf;
use std::env;

fn main() {
    cc::Build::new()
        .file("src/boot.S")
        .target("riscv64imac-unknown-none-elf")
        .flag("-march=rv64imac_zicsr") // Specify RISC-V ISA
        .flag("-mabi=lp64") // Specify ABI
        .flag("-O2") // Optimization level
        .compiler("riscv64-elf-gcc")
        .compile("chopin-boot");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    
    // Tell cargo where to find the library
    println!("cargo:rustc-link-search={}", manifest_dir.display());
    
    // Link the static file (assuming it's named libmyfile.a)
    // Remove 'lib' prefix and '.a' suffix from filename
    println!("cargo:rustc-link-lib=static=chopin-boot");

    println!("cargo:rerun-if-changed=./src/boot.S");
}
