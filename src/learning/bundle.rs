use crate::learning::case::DecisionCase;
use crate::learning::lesson::Lesson;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const EMBEDDED_BUNDLE: &str = include_str!("../../knowledge/bundled-v1.json");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeBundle {
    pub schema_version: u32,
    pub bundle_id: String,
    pub descriptor_versions: Vec<u32>,
    pub cases: Vec<DecisionCase>,
    #[serde(default)]
    pub lessons: Vec<Lesson>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundleError {
    Json,
    UnsupportedSchema,
    InvalidCase,
    InvalidIdentity,
    NonCanonical,
    InvalidLesson,
    MissingLessonSource,
}

impl KnowledgeBundle {
    #[cfg(test)]
    pub fn from_cases(cases: Vec<DecisionCase>) -> Result<Self, BundleError> {
        Self::from_knowledge(cases, vec![])
    }

    pub fn from_knowledge(
        mut cases: Vec<DecisionCase>,
        mut lessons: Vec<Lesson>,
    ) -> Result<Self, BundleError> {
        if cases.iter().any(|case| !case.verify_id()) {
            return Err(BundleError::InvalidCase);
        }
        if lessons.iter().any(|lesson| !lesson.verify_identity()) {
            return Err(BundleError::InvalidLesson);
        }
        cases.sort_by(|left, right| left.case_id.cmp(&right.case_id));
        cases.dedup_by(|left, right| left.case_id == right.case_id);
        lessons.sort_by(|left, right| left.lesson_id.cmp(&right.lesson_id));
        lessons.dedup_by(|left, right| left.lesson_id == right.lesson_id);
        if !lesson_sources_present(&cases, &lessons) {
            return Err(BundleError::MissingLessonSource);
        }
        let mut bundle = Self {
            schema_version: 1,
            bundle_id: String::new(),
            descriptor_versions: vec![1],
            cases,
            lessons,
        };
        bundle.bundle_id = bundle.compute_id()?;
        Ok(bundle)
    }

    pub fn from_json(json: &str) -> Result<Self, BundleError> {
        let bundle: Self = serde_json::from_str(json).map_err(|_| BundleError::Json)?;
        if bundle.schema_version != 1 || bundle.descriptor_versions != [1] {
            return Err(BundleError::UnsupportedSchema);
        }
        if !bundle.is_canonical() {
            return Err(BundleError::NonCanonical);
        }
        if bundle.cases.iter().any(|case| !case.verify_id()) {
            return Err(BundleError::InvalidCase);
        }
        if bundle
            .lessons
            .iter()
            .any(|lesson| !lesson.verify_identity())
        {
            return Err(BundleError::InvalidLesson);
        }
        if !lesson_sources_present(&bundle.cases, &bundle.lessons) {
            return Err(BundleError::MissingLessonSource);
        }
        if !bundle.verify() {
            return Err(BundleError::InvalidIdentity);
        }
        Ok(bundle)
    }

    pub fn verify(&self) -> bool {
        self.schema_version == 1
            && self.descriptor_versions == [1]
            && self.is_canonical()
            && self.cases.iter().all(DecisionCase::verify_id)
            && self.lessons.iter().all(Lesson::verify_identity)
            && lesson_sources_present(&self.cases, &self.lessons)
            && self.compute_id().is_ok_and(|id| id == self.bundle_id)
    }

    fn is_canonical(&self) -> bool {
        self.cases
            .windows(2)
            .all(|pair| pair[0].case_id < pair[1].case_id)
            && self
                .lessons
                .windows(2)
                .all(|pair| pair[0].lesson_id < pair[1].lesson_id)
    }

    fn compute_id(&self) -> Result<String, BundleError> {
        let identity = BundleIdentity {
            schema_version: self.schema_version,
            descriptor_versions: &self.descriptor_versions,
            cases: &self.cases,
            lessons: &self.lessons,
        };
        let bytes = serde_json::to_vec(&identity).map_err(|_| BundleError::Json)?;
        Ok(format!("sha256:{}", hex::encode(Sha256::digest(bytes))))
    }
}

#[derive(Serialize)]
struct BundleIdentity<'a> {
    schema_version: u32,
    descriptor_versions: &'a [u32],
    cases: &'a [DecisionCase],
    lessons: &'a [Lesson],
}

pub fn embedded_bundle() -> Result<KnowledgeBundle, BundleError> {
    KnowledgeBundle::from_json(EMBEDDED_BUNDLE)
}

fn lesson_sources_present(cases: &[DecisionCase], lessons: &[Lesson]) -> bool {
    lessons.iter().all(|lesson| {
        lesson.source_case_ids.iter().all(|source_id| {
            cases
                .binary_search_by(|case| case.case_id.as_str().cmp(source_id))
                .is_ok()
        })
    })
}
