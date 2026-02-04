//! Storage abstraction and implementations for DevMan.
//!
//! This crate provides a trait-based storage interface with:
//! - JSON storage (default, file-based)
//! - SQLite storage (high-performance, recommended for production)
//!
//! # Examples
//!
//! ## Using SQLite storage
//!
//! ```ignore
//! use devman_storage::SqliteStorage;
//!
//! let storage = SqliteStorage::new(".devman/devman.db").await.unwrap();
//! ```

#![warn(missing_docs)]

#[cfg(feature = "json")]
pub mod trait_;
#[cfg(feature = "json")]
pub mod json_storage;

#[cfg(feature = "sqlite")]
pub mod sqlite_storage;

pub use trait_::{Storage, StorageError, Result, Transaction};

#[cfg(feature = "json")]
pub use json_storage::JsonStorage;

#[cfg(feature = "sqlite")]
pub use sqlite_storage::SqliteStorage;
