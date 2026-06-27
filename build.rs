use std::path::Path;

fn main() {
    let src = "src/ranker/rules.json";
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("rules.json");

    std::fs::copy(src, &dest)
        .unwrap_or_else(|e| panic!("failed to copy {src} to {}: {e}", dest.display()));

    println!("cargo:rerun-if-changed={src}");
}
