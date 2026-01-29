//! Storage abstraction and implementations for DevMan.
//!
//! This crate provides a trait-based storage interface with a Git+JSON
//! reference implementation.

#![warn(missing_docs)]

pub mod trait_;
pub mod git_json;

pub use trait_::{Storage, StorageError, Result};
pub use git_json::GitJsonStorage;
