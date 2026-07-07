use std::path::Path;

fn main() {
    let src = "src/ranker/rules.json";
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set by Cargo");
    // OUT_DIR = target/<profile>/build/<package>-<hash>/out
    // Going up 3 parents yields target/<profile>/ (where the binary lives).
    let target_dir = Path::new(&out_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("OUT_DIR structure unexpected — Cargo may have changed its build directory layout");
    let dest = target_dir.join("rules.json");

    std::fs::copy(src, &dest)
        .unwrap_or_else(|e| panic!("failed to copy {src} to {}: {e}", dest.display()));

    println!("cargo:rerun-if-changed={src}");
}
