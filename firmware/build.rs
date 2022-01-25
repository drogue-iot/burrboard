use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

const MEM_UF2: &[u8] = include_bytes!("memory-uf2.x");
const MEM_BM: &[u8] = include_bytes!("memory-bm.x");
fn main() {
    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let mem = if env::var("CARGO_FEATURE_SENSE").is_ok() {
        MEM_UF2
    } else {
        MEM_UF2
    };
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(mem)
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory.x`
    // here, we ensure the build script is only re-run when
    // `memory.x` is changed.
    println!("cargo:rerun-if-changed=memory-uf2.x");
    println!("cargo:rerun-if-changed=memory-bm.x");

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}
