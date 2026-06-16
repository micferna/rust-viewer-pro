//! Rust Viewer Pro — library crate.
//!
//! The GUI binary (`src/main.rs`) is a thin wrapper around these modules, which
//! are also exercised by benchmarks and tests.
#![forbid(unsafe_code)]

pub mod app;
pub mod decode;
pub mod image_loader;
pub mod input;
pub mod update;
