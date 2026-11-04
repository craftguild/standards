//! Shared build facts and distribution seals, independent of CLI grammar.

#![forbid(unsafe_code)]

pub mod identity;
pub mod paths;
pub mod sbom;
pub mod seal;

pub use identity::{BuildInfo, Dirty};

/// A validation failure with context suitable for a build diagnostic.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
