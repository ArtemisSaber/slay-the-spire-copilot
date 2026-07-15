use super::capture::facts;
use super::{FinalizeSummary, LearningSession, RunCapture};
use crate::learning::bundle::embedded_bundle;
use crate::learning::case::CaseProvenance;
use crate::learning::eligibility::evaluate;

impl LearningSession {
    pub fn finalize(&mut self, reason: &str) -> anyhow::Result<FinalizeSummary> {
        let result = self.commit_finalized_run(reason);
        self.capture = RunCapture::default();
        result
    }

    fn commit_finalized_run(&mut self, reason: &str) -> anyhow::Result<FinalizeSummary> {
        let run_facts = facts(&self.capture, &self.config, &self.provenance, reason);
        let eligibility = evaluate(&run_facts);
        self.last_eligibility = Some(eligibility.clone());
        let mut appended_cases = 0;
        let mut skipped_cases = 0;
        if eligibility.knowledge_eligible {
            let provenance = CaseProvenance {
                app_version: env!("CARGO_PKG_VERSION").into(),
                prompt_schema_version: 1,
                rules_sha256: self.provenance.rules_sha256.clone(),
                model_profile_sha256: self.provenance.model_profile_sha256.clone(),
                mod_profile_sha256: self.config.mod_profile_sha256.clone().unwrap_or_default(),
                knowledge_snapshot_id: Some(self.snapshot.snapshot_id.clone()),
            };
            let cases = self.capture.finalize_cases(
                run_facts.terminal_outcome,
                provenance,
                self.config.max_cases_per_run,
            )?;
            let summary = self.store.append_cases(&cases)?;
            appended_cases = summary.appended;
            skipped_cases = summary.skipped;
            let bundle = embedded_bundle()
                .map_err(|error| anyhow::anyhow!("embedded bundle invalid: {error:?}"))?;
            self.snapshot = self.store.rebuild(&[bundle])?;
        }
        self.store.write_status(
            &self.snapshot,
            self.config.mode,
            self.last_eligibility.as_ref(),
        )?;
        Ok(FinalizeSummary {
            eligibility,
            appended_cases,
            skipped_cases,
            snapshot_id: self.snapshot.snapshot_id.clone(),
        })
    }
}
