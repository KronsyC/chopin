fn main(){

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    println!("cargo:rustc-link-arg=-T{manifest_dir}/kernel.ld");
    
    // Ensure cargo rebuilds if the linker script changes
    println!("cargo:rerun-if-changed=kernel.ld");
}
