use sha2::{Digest, Sha256};

const COMPATIBILITY_FORMAT: &str = "sts-copilot-learning-compatibility-v1";

pub(super) fn runtime_compatibility_sha256(rules_sha256: &str) -> String {
    let mut hasher = Sha256::new();
    for component in [
        COMPATIBILITY_FORMAT,
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        "case-schema:1",
        "descriptor-schema:1",
        "ranker-tag-schema:1",
        "prompt-schema:1",
        rules_sha256,
    ] {
        hasher.update((component.len() as u64).to_le_bytes());
        hasher.update(component.as_bytes());
    }
    format!("sha256:{}", hex::encode(hasher.finalize()))
}
