use crate::learning::bundle::KnowledgeBundle;
use crate::learning::case::DecisionCase;
use crate::learning::lesson::Lesson;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeSnapshot {
    pub schema_version: u32,
    pub descriptor_version: u32,
    pub retrieval_algorithm_version: u32,
    pub snapshot_id: String,
    pub bundle_ids: Vec<String>,
    pub cases: Vec<DecisionCase>,
    #[serde(default)]
    pub lessons: Vec<Lesson>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotError {
    InvalidBundle,
    InvalidCase,
    ConflictingCase,
    InvalidLesson,
    ConflictingLesson,
    MissingLessonSource,
    Serialization,
}

impl KnowledgeSnapshot {
    pub fn build(
        local_cases: Vec<DecisionCase>,
        bundles: &[KnowledgeBundle],
    ) -> Result<Self, SnapshotError> {
        Self::build_with_lessons(local_cases, vec![], bundles)
    }

    pub fn build_with_lessons(
        local_cases: Vec<DecisionCase>,
        local_lessons: Vec<Lesson>,
        bundles: &[KnowledgeBundle],
    ) -> Result<Self, SnapshotError> {
        if bundles.iter().any(|bundle| !bundle.verify()) {
            return Err(SnapshotError::InvalidBundle);
        }
        let mut cases = BTreeMap::new();
        for case in local_cases.into_iter().chain(
            bundles
                .iter()
                .flat_map(|bundle| bundle.cases.iter().cloned()),
        ) {
            if !case.verify_id() {
                return Err(SnapshotError::InvalidCase);
            }
            if let Some(existing) = cases.get(&case.case_id)
                && existing != &case
            {
                return Err(SnapshotError::ConflictingCase);
            }
            cases.insert(case.case_id.clone(), case);
        }
        let mut bundle_ids: Vec<_> = bundles
            .iter()
            .map(|bundle| bundle.bundle_id.clone())
            .collect();
        bundle_ids.sort();
        bundle_ids.dedup();
        let all_cases: Vec<_> = cases.into_values().collect();
        let mut lessons = BTreeMap::new();
        for lesson in bundles
            .iter()
            .flat_map(|bundle| bundle.lessons.iter().cloned())
        {
            if !lesson.verify_identity() {
                return Err(SnapshotError::InvalidLesson);
            }
            if let Some(existing) = lessons.get(&lesson.lesson_id)
                && existing != &lesson
            {
                return Err(SnapshotError::ConflictingLesson);
            }
            lessons.insert(lesson.lesson_id.clone(), lesson);
        }
        for lesson in local_lessons {
            if !lesson.verify_identity() {
                return Err(SnapshotError::InvalidLesson);
            }
            lessons.insert(lesson.lesson_id.clone(), lesson);
        }
        if lessons.values().any(|lesson| {
            lesson.source_case_ids.iter().any(|source_id| {
                all_cases
                    .binary_search_by(|case| case.case_id.as_str().cmp(source_id))
                    .is_err()
            })
        }) {
            return Err(SnapshotError::MissingLessonSource);
        }
        if lessons
            .values()
            .any(|lesson| !lesson.sources_are_valid(&all_cases))
        {
            return Err(SnapshotError::InvalidLesson);
        }
        for lesson in lessons.values_mut() {
            lesson.recalculate_support(&all_cases);
        }
        let mut snapshot = Self {
            schema_version: 1,
            descriptor_version: 1,
            retrieval_algorithm_version: 1,
            snapshot_id: String::new(),
            bundle_ids,
            cases: all_cases,
            lessons: lessons.into_values().collect(),
        };
        snapshot.snapshot_id = snapshot.compute_id()?;
        Ok(snapshot)
    }

    pub fn verify(&self) -> bool {
        self.schema_version == 1
            && self.descriptor_version == 1
            && self.retrieval_algorithm_version == 1
            && self.cases.iter().all(DecisionCase::verify_id)
            && self.lessons.iter().all(Lesson::verify_identity)
            && self
                .cases
                .windows(2)
                .all(|pair| pair[0].case_id < pair[1].case_id)
            && self
                .lessons
                .windows(2)
                .all(|pair| pair[0].lesson_id < pair[1].lesson_id)
            && self.compute_id().is_ok_and(|id| id == self.snapshot_id)
    }

    pub fn file_name(&self) -> String {
        format!(
            "snapshot-{}.json",
            self.snapshot_id
                .strip_prefix("sha256:")
                .unwrap_or(&self.snapshot_id)
        )
    }

    fn compute_id(&self) -> Result<String, SnapshotError> {
        let identity = SnapshotIdentity {
            schema_version: self.schema_version,
            descriptor_version: self.descriptor_version,
            retrieval_algorithm_version: self.retrieval_algorithm_version,
            bundle_ids: &self.bundle_ids,
            cases: &self.cases,
            lessons: &self.lessons,
        };
        let bytes = serde_json::to_vec(&identity).map_err(|_| SnapshotError::Serialization)?;
        Ok(format!("sha256:{}", hex::encode(Sha256::digest(bytes))))
    }
}

#[derive(Serialize)]
struct SnapshotIdentity<'a> {
    schema_version: u32,
    descriptor_version: u32,
    retrieval_algorithm_version: u32,
    bundle_ids: &'a [String],
    cases: &'a [DecisionCase],
    lessons: &'a [Lesson],
}
