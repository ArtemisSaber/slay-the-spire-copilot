use super::PendingCase;

const TERMINAL_CASE_RESERVE: usize = 32;

pub(super) fn retain_bounded_cases(cases: Vec<PendingCase>, maximum: usize) -> Vec<PendingCase> {
    if cases.len() <= maximum {
        return cases;
    }
    if maximum == 0 {
        return vec![];
    }
    let terminal_count = maximum.min(TERMINAL_CASE_RESERVE);
    let earlier_end = cases.len() - terminal_count;
    let earlier_slots = maximum - terminal_count;
    let mut keep = vec![false; cases.len()];
    for slot in 0..earlier_slots {
        keep[slot * earlier_end / earlier_slots] = true;
    }
    for retained in keep.iter_mut().skip(earlier_end) {
        *retained = true;
    }
    cases
        .into_iter()
        .zip(keep)
        .filter_map(|(case, keep)| keep.then_some(case))
        .collect()
}
