//! Storage abstraction and implementations for DevMan.
//!
//! This crate provides a trait-based storage interface with a Git+JSON
//! reference implementation.

#![warn(missing_docs)]

pub mod trait_;
pub mod json_storage;

pub use trait_::{Storage, StorageError, Result, Transaction};
pub use json_storage::JsonStorage;
