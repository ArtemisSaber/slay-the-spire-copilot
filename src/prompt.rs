pub mod builder;
pub mod routing;

pub use builder::build_prompt;
pub use routing::{enumerate_paths, enumerate_paths_from_roots, summarize_path};

#[cfg(test)]
#[path = "tests/prompt_tests.rs"]
mod tests;
