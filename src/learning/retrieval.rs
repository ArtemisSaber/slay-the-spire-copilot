use crate::learning::case::DecisionCase;
use crate::learning::config::MemoryConfig;
use crate::learning::descriptor::SituationDescriptor;
use crate::learning::lesson::{Lesson, LessonStatus, lesson_guidance_is_coherent};
use crate::learning::snapshot::KnowledgeSnapshot;
use std::collections::HashSet;

mod similarity;
pub use similarity::situation_similarity;

const MIN_PROPOSED_LESSON_CONFIDENCE: u16 = 600;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalQuery {
    pub situation: SituationDescriptor,
    pub seed_hash: String,
    pub compatibility_sha256: String,
    pub language: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalResult {
    pub snapshot_id: String,
    pub items: Vec<RetrievalItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetrievalItem {
    Case {
        case: Box<DecisionCase>,
        similarity: u16,
        rank: i32,
    },
    Lesson {
        lesson: Box<Lesson>,
        evidence_case_id: String,
        similarity: u16,
        rank: i32,
    },
}

impl RetrievalItem {
    pub fn id(&self) -> String {
        match self {
            Self::Case { case, .. } => case.case_id.clone(),
            Self::Lesson { lesson, .. } => lesson.lesson_id.clone(),
        }
    }

    pub fn similarity(&self) -> u16 {
        match self {
            Self::Case { similarity, .. } | Self::Lesson { similarity, .. } => *similarity,
        }
    }

    fn rank(&self) -> i32 {
        match self {
            Self::Case { rank, .. } | Self::Lesson { rank, .. } => *rank,
        }
    }

    fn status_priority(&self) -> u8 {
        match self {
            Self::Lesson { lesson, .. } => match lesson.status {
                LessonStatus::Validated => 3,
                LessonStatus::Supported => 2,
                LessonStatus::Proposed => 1,
                LessonStatus::Contested | LessonStatus::Retired => 0,
            },
            Self::Case { .. } => 0,
        }
    }
}

pub fn retrieve(
    snapshot: &KnowledgeSnapshot,
    query: &RetrievalQuery,
    config: &MemoryConfig,
) -> RetrievalResult {
    let mut candidates = raw_candidates(snapshot, query, config);
    candidates.extend(lesson_candidates(snapshot, query, config));
    candidates.sort_by(|left, right| {
        right
            .rank()
            .cmp(&left.rank())
            .then_with(|| right.similarity().cmp(&left.similarity()))
            .then_with(|| right.status_priority().cmp(&left.status_priority()))
            .then_with(|| left.id().cmp(&right.id()))
    });
    RetrievalResult {
        snapshot_id: snapshot.snapshot_id.clone(),
        items: diversify(candidates, config.max_items),
    }
}

fn raw_candidates(
    snapshot: &KnowledgeSnapshot,
    query: &RetrievalQuery,
    config: &MemoryConfig,
) -> Vec<RetrievalItem> {
    snapshot
        .cases
        .iter()
        .filter(|case| compatible_case(case, query))
        .filter_map(|case| {
            let similarity = situation_similarity(&query.situation, &case.situation);
            (similarity >= config.case_min_similarity).then(|| RetrievalItem::Case {
                case: Box::new(case.clone()),
                similarity,
                rank: i32::from(similarity),
            })
        })
        .collect()
}

fn lesson_candidates(
    snapshot: &KnowledgeSnapshot,
    query: &RetrievalQuery,
    config: &MemoryConfig,
) -> Vec<RetrievalItem> {
    snapshot
        .lessons
        .iter()
        .filter(|lesson| lesson.language == query.language)
        .filter(|lesson| lesson.matches_situation(&query.situation))
        .filter(|lesson| {
            !matches!(
                lesson.status,
                LessonStatus::Contested | LessonStatus::Retired
            )
        })
        .filter(|lesson| {
            lesson.status == LessonStatus::Validated || lesson_guidance_is_coherent(lesson)
        })
        .filter(|lesson| {
            lesson.status != LessonStatus::Proposed
                || lesson.critic.confidence_millis >= MIN_PROPOSED_LESSON_CONFIDENCE
        })
        .filter_map(|lesson| best_lesson_item(lesson, snapshot, query, config))
        .collect()
}

fn best_lesson_item(
    lesson: &Lesson,
    snapshot: &KnowledgeSnapshot,
    query: &RetrievalQuery,
    config: &MemoryConfig,
) -> Option<RetrievalItem> {
    let allow_cited = lesson.status == LessonStatus::Proposed;
    let (evidence, similarity) = snapshot
        .cases
        .iter()
        .filter(|case| compatible_case(case, query))
        .filter(|case| {
            lesson.is_independent_support(case)
                || (allow_cited && lesson.source_case_ids.contains(&case.case_id))
        })
        .map(|case| {
            (
                case,
                situation_similarity(&query.situation, &case.situation),
            )
        })
        .max_by(|(left_case, left), (right_case, right)| {
            left.cmp(right)
                .then_with(|| right_case.case_id.cmp(&left_case.case_id))
        })?;
    let threshold = match lesson.status {
        LessonStatus::Proposed => config.proposed_lesson_min_similarity,
        LessonStatus::Supported | LessonStatus::Validated => config.lesson_min_similarity,
        LessonStatus::Contested | LessonStatus::Retired => return None,
    };
    if similarity < threshold {
        return None;
    }
    let status_bonus = match lesson.status {
        LessonStatus::Validated => 150,
        LessonStatus::Supported => 75,
        _ => 0,
    };
    let support_bonus = 50.min(10 * lesson.support.distinct_independent_seeds as i32);
    let contradiction_penalty = 100.min(20 * lesson.support.contradicting_cases as i32);
    Some(RetrievalItem::Lesson {
        lesson: Box::new(lesson.clone()),
        evidence_case_id: evidence.case_id.clone(),
        similarity,
        rank: i32::from(similarity) + status_bonus + support_bonus - contradiction_penalty,
    })
}

fn compatible_case(case: &DecisionCase, query: &RetrievalQuery) -> bool {
    let left = &case.situation;
    let right = &query.situation;
    case.seed_hash != query.seed_hash
        && case.provenance.compatibility_sha256 == query.compatibility_sha256
        && left.descriptor_version == right.descriptor_version
        && left.ranker_tag_schema_version == right.ranker_tag_schema_version
        && left.character == right.character
        && left.objective == right.objective
        && left.ascension_band == right.ascension_band
        && left.encounter_ids == right.encounter_ids
}

fn diversify(candidates: Vec<RetrievalItem>, maximum: usize) -> Vec<RetrievalItem> {
    let mut selected = Vec::new();
    let mut families = HashSet::new();
    let mut lessons = 0;
    let mut cases = 0;
    for candidate in candidates {
        let keep = match &candidate {
            RetrievalItem::Lesson { lesson, .. } => {
                lessons < 2 && families.insert(lesson.family_key.clone())
            }
            RetrievalItem::Case { .. } => cases < 1,
        };
        if !keep {
            continue;
        }
        match &candidate {
            RetrievalItem::Lesson { .. } => lessons += 1,
            RetrievalItem::Case { .. } => cases += 1,
        }
        selected.push(candidate);
        if selected.len() >= maximum {
            break;
        }
    }
    selected
}
