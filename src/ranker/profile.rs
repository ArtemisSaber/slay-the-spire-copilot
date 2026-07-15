use sha2::{Digest, Sha256};

use super::rules::RuleSet;

pub fn active_rules_sha256(project_root: &std::path::Path) -> String {
    let embedded = include_str!("rules.json").as_bytes();
    let path = project_root.join("rules.json");
    let bytes = std::fs::read(path)
        .ok()
        .filter(|bytes| serde_json::from_slice::<RuleSet>(bytes).is_ok())
        .unwrap_or_else(|| embedded.to_vec());
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}
