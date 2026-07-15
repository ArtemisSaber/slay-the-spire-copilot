use sha2::{Digest, Sha256};
use std::path::Path;

use super::{LearningSession, SessionProvenance};
use crate::config::Config;
use crate::learning::bundle::embedded_bundle;
use crate::learning::compatibility::runtime_compatibility_sha256;
use crate::learning::snapshot::KnowledgeSnapshot;
use crate::learning::store::KnowledgeStore;

pub fn bootstrap_session(
    project_root: &Path,
    config: &Config,
    locale: &str,
    synthetic_input: bool,
) -> anyhow::Result<LearningSession> {
    let bundle = embedded_bundle()
        .map_err(|error| anyhow::anyhow!("embedded knowledge bundle invalid: {error:?}"))?;
    let store = KnowledgeStore::new(project_root.join("learning").join("knowledge"));
    let snapshot = if config.memory.captures() {
        match store.rebuild(std::slice::from_ref(&bundle)) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                tracing::error!(
                    "local knowledge rebuild failed; using repository bundle only: {error:#}"
                );
                KnowledgeSnapshot::build(vec![], std::slice::from_ref(&bundle))
                    .map_err(|error| anyhow::anyhow!("knowledge fallback failed: {error:?}"))?
            }
        }
    } else {
        KnowledgeSnapshot::build(vec![], &[bundle])
            .map_err(|error| anyhow::anyhow!("knowledge snapshot build failed: {error:?}"))?
    };
    let rules_sha256 = crate::ranker::active_rules_sha256(project_root);
    let provenance = SessionProvenance {
        locale: locale.to_string(),
        model_profile_sha256: model_profile_sha256(config)?,
        compatibility_sha256: runtime_compatibility_sha256(&rules_sha256),
        rules_sha256,
        synthetic_input,
    };
    if config.memory.captures() {
        store.write_status(&snapshot, config.memory.mode, None)?;
    }
    Ok(LearningSession::new(
        config.memory.clone(),
        store,
        snapshot,
        provenance,
    ))
}

fn model_profile_sha256(config: &Config) -> anyhow::Result<String> {
    let identity = serde_json::json!({
        "provider": config.provider,
        "base_url": config.base_url,
        "model_fast": config.model_fast,
        "model_medium": config.model_medium,
        "model_heavy": config.model_heavy,
        "max_tokens_fast": config.max_tokens_fast,
        "max_tokens_medium": config.max_tokens_medium,
        "max_tokens_heavy": config.max_tokens_heavy,
        "temperature_bits": config.temperature.to_bits(),
        "disable_fast_thinking": config.disable_fast_thinking,
    });
    let bytes = serde_json::to_vec(&identity)?;
    Ok(format!("sha256:{}", hex::encode(Sha256::digest(bytes))))
}
