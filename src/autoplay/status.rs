use std::path::Path;

use serde::Serialize;

use crate::advice::OverlayMetadata;

#[derive(Debug, Clone, Default, Serialize)]
pub struct AutoPlayState {
    pub mode: String,
    pub status: String,
}

pub(crate) fn write_overlay_autoplay(
    overlay_path: &Path,
    metadata: &OverlayMetadata,
    autoplay: &AutoPlayState,
) {
    let mut output = std::fs::read_to_string(overlay_path)
        .ok()
        .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
        .unwrap_or_else(|| {
            serde_json::json!({
                "schema_version": 1,
                "status": "ok",
                "overlay_visibility": false,
                "advice": {
                    "recommendation": "",
                    "reason": "",
                    "risk": "",
                    "commentary": ""
                }
            })
        });

    output["autoplay"] = serde_json::to_value(autoplay).unwrap_or(serde_json::Value::Null);
    output["screen_type"] = metadata
        .screen_type
        .clone()
        .map(serde_json::Value::String)
        .unwrap_or(serde_json::Value::Null);
    output["scenario"] = serde_json::Value::String(metadata.scenario.clone());
    output["in_combat"] = serde_json::Value::Bool(metadata.in_combat);
    output["state_hash"] = serde_json::Value::String(metadata.state_hash.clone());
    output["floor"] = metadata
        .floor
        .map(serde_json::Value::from)
        .unwrap_or(serde_json::Value::Null);
    output["character"] = metadata
        .character
        .clone()
        .map(serde_json::Value::String)
        .unwrap_or(serde_json::Value::Null);
    output["timestamp_ms"] = serde_json::Value::from(crate::advice::timestamp_ms() as u64);

    if let Ok(json) = serde_json::to_string_pretty(&output) {
        crate::advice::atomic_write_json(overlay_path, &json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata() -> OverlayMetadata {
        OverlayMetadata {
            screen_type: Some("NONE".into()),
            scenario: "combat".into(),
            in_combat: true,
            state_hash: "abc123".into(),
            floor: Some(7),
            character: Some("IRONCLAD".into()),
        }
    }

    #[test]
    fn creates_overlay_when_file_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("overlay.json");
        let autoplay = AutoPlayState {
            mode: "auto".into(),
            status: "planning".into(),
        };

        write_overlay_autoplay(&path, &metadata(), &autoplay);

        let output: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(output["autoplay"]["mode"], "auto");
        assert_eq!(output["autoplay"]["status"], "planning");
        assert_eq!(output["screen_type"], "NONE");
        assert_eq!(output["scenario"], "combat");
        assert_eq!(output["in_combat"], true);
        assert_eq!(output["floor"], 7);
        assert_eq!(output["character"], "IRONCLAD");
        assert_eq!(output["state_hash"], "abc123");
    }

    #[test]
    fn patches_existing_overlay_preserving_advice() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("overlay.json");
        std::fs::write(
            &path,
            r#"{"schema_version":1,"status":"ok","overlay_visibility":true,"advice":{"recommendation":"武装","reason":"好","risk":"卡手","commentary":"还行"},"screen_type":"CARD_REWARD","scenario":"card_reward","in_combat":false,"state_hash":"def456","floor":3,"character":"IRONCLAD","timestamp_ms":1000}"#,
        )
        .unwrap();

        let autoplay = AutoPlayState {
            mode: "auto".into(),
            status: "executing".into(),
        };

        write_overlay_autoplay(&path, &metadata(), &autoplay);

        let output: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(output["status"], "ok");
        assert_eq!(output["advice"]["recommendation"], "武装");
        assert_eq!(output["advice"]["reason"], "好");
        assert_eq!(output["autoplay"]["mode"], "auto");
        assert_eq!(output["autoplay"]["status"], "executing");
        assert_eq!(output["screen_type"], "NONE");
        assert_eq!(output["scenario"], "combat");
        assert_eq!(output["floor"], 7);
        assert_eq!(output["state_hash"], "abc123");
    }
}
