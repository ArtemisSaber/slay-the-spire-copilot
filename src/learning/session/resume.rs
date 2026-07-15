use std::path::Path;

use super::LearningSession;

impl LearningSession {
    pub fn pin_snapshot_from_journal(&mut self, journal_path: &Path) -> anyhow::Result<bool> {
        let content = std::fs::read_to_string(journal_path)?;
        let snapshot_id = content.lines().find_map(|line| {
            let event: serde_json::Value = serde_json::from_str(line).ok()?;
            if event.get("event")?.as_str()? != "autoplay_decision_proposed" {
                return None;
            }
            event
                .get("knowledge_snapshot_id")?
                .as_str()
                .map(str::to_string)
        });
        let Some(snapshot_id) = snapshot_id else {
            return Ok(false);
        };
        match self.store.load_snapshot_by_id(&snapshot_id) {
            Ok(snapshot) => self.snapshot = snapshot,
            Err(error) => {
                self.config.mode = crate::learning::config::MemoryMode::Off;
                self.capture = Default::default();
                return Err(error);
            }
        }
        Ok(true)
    }
}
