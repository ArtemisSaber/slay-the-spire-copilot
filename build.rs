use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const DEFAULT_MAX_LOGICAL_LOC: usize = 250;
const FILE_SIZE_ALLOWLIST: &[(&str, usize)] = &[
    ("src/autoplay/action.rs", 611),
    ("src/autoplay/planner.rs", 604),
    ("src/combat/damage.rs", 385),
    ("src/combat/effects.rs", 1253),
    ("src/combat/kill_scan.rs", 521),
    ("src/llm.rs", 785),
    ("src/main.rs", 778),
    ("src/postmortem.rs", 591),
    ("src/prompt/builder.rs", 1288),
    ("src/setup_wizard.rs", 1344),
    ("src/startup.rs", 594),
    ("src/state.rs", 1261),
];

fn main() {
    check_file_size_ratchet();
    copy_ranker_rules();
}

fn copy_ranker_rules() {
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

fn check_file_size_ratchet() {
    let root = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by Cargo"),
    );
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=tests");

    let allowlist: BTreeMap<&'static str, usize> = FILE_SIZE_ALLOWLIST.iter().copied().collect();
    let mut seen = BTreeSet::new();
    let mut failures = Vec::new();

    for path in rust_files(&root) {
        let rel = path
            .strip_prefix(&root)
            .expect("scanned file should be under repository root")
            .to_string_lossy()
            .replace('\\', "/");
        let loc = logical_loc(&path);

        match allowlist.get(rel.as_str()).copied() {
            Some(allowed) => {
                seen.insert(rel.clone());
                if loc > allowed {
                    failures.push(format!(
                        "{rel}: {loc} logical LOC exceeds ratchet {allowed}; split the file"
                    ));
                } else if loc < allowed {
                    failures.push(format!(
                        "{rel}: {loc} logical LOC is below ratchet {allowed}; \
                         lower or remove its build.rs allowlist entry"
                    ));
                }
            }
            None if loc > DEFAULT_MAX_LOGICAL_LOC => failures.push(format!(
                "{rel}: {loc} logical LOC exceeds default cap {DEFAULT_MAX_LOGICAL_LOC}; \
                 split the file or add an explicit ratchet entry"
            )),
            None => {}
        }
    }

    for allowed_path in allowlist.keys() {
        if !seen.contains(*allowed_path) {
            failures.push(format!(
                "{allowed_path}: allowlist entry points to a missing Rust file"
            ));
        }
    }

    if !failures.is_empty() {
        let details = failures
            .into_iter()
            .map(|failure| format!("  - {failure}"))
            .collect::<Vec<_>>()
            .join("\n");
        panic!("file size ratchet failed:\n{details}");
    }
}

fn rust_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for dirname in ["src", "tests"] {
        let dir = root.join(dirname);
        if dir.exists() {
            collect_rust_files(&dir, &mut files);
        }
    }
    files.sort();
    files
}

fn collect_rust_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()))
    {
        let path = entry
            .unwrap_or_else(|err| panic!("failed to read entry in {}: {err}", dir.display()))
            .path();
        if path.is_dir() {
            collect_rust_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}

fn logical_loc(path: &Path) -> usize {
    let contents = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    contents
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with("//")
        })
        .count()
}
