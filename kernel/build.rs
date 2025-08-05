extern crate bindgen;

use bindgen::{Builder, RustEdition, MacroTypeVariation::Signed};
use std::path::PathBuf;

const FILEPATH: &str = "src/bindings/imports.rs";
const HEADERPATH: &str = "headers/wrapper.h";

fn main() {
    let src_base = match std::env::var("SRC_BASE") {
        Ok(s) => format!("-I{s}/sys"),
        _ => "-I/usr/src/sys".to_string(),
    };
    let bindings = Builder::default()
        .use_core()
        .ctypes_prefix("libc")
        .size_t_is_usize(true)
        .default_macro_constant_type(Signed)
        .header(HEADERPATH)
        .clang_arg("-D_KERNEL")
        .clang_arg("-DKLD_MODULE")
        .clang_arg("-I.")
        .clang_arg(src_base)
        .rust_edition(RustEdition::Edition2021)
        .rust_target(bindgen::RustTarget::stable(77, 0)
                     .unwrap_or_else(|_| panic!("Invalid Rust Target")))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(FILEPATH);
    bindings
        .write_to_file(out_path)
        .expect("Unable to write bindings");
}
