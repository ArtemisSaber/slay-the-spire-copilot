mod assignments;
mod env_file;
mod features;
mod flow;
mod prompts;
mod types;

#[allow(
    unused_imports,
    reason = "Retained as part of the setup wizard module's public API."
)]
pub use flow::env_path;
pub use flow::{maybe_run_setup, run_setup};

#[cfg(test)]
#[allow(
    unused_imports,
    reason = "Re-exported only to preserve the existing setup wizard test namespace."
)]
use assignments::*;
#[cfg(test)]
#[allow(
    unused_imports,
    reason = "Re-exported only to preserve the existing setup wizard test namespace."
)]
use env_file::*;
#[cfg(test)]
#[allow(
    unused_imports,
    reason = "Re-exported only to preserve the existing setup wizard test namespace."
)]
use features::*;
#[cfg(test)]
#[allow(
    unused_imports,
    reason = "Re-exported only to preserve the existing setup wizard test namespace."
)]
use flow::run_setup_with_io;
#[cfg(test)]
#[allow(
    unused_imports,
    reason = "Re-exported only to preserve the existing setup wizard test namespace."
)]
use std::{collections::HashMap, fs};
#[cfg(test)]
#[allow(
    unused_imports,
    reason = "Re-exported only to preserve the existing setup wizard test namespace."
)]
use types::*;

#[cfg(test)]
#[path = "tests/setup_wizard_tests.rs"]
mod tests;
