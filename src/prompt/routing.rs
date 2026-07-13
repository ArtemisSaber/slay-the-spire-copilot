mod models;
mod paths;
mod scoring;

pub use models::ShopTiming;
#[allow(
    unused_imports,
    reason = "preserves routing's existing public surface for sibling modules and tests"
)]
pub use models::{PathCounts, PathDescription, PathMetrics};
#[allow(
    unused_imports,
    reason = "preserves routing's existing public surface for sibling modules and tests"
)]
pub use paths::{RootPaths, describe_path};
pub use paths::{enumerate_paths, enumerate_paths_from_roots, summarize_path};
pub(crate) use scoring::plural;
pub use scoring::{PathEvaluation, evaluate_path};
