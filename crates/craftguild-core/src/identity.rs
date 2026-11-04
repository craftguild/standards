//! Structured application build facts shared by preparation and build scripts.

use serde::{Deserialize, Serialize};

/// A failed observation must not be represented as a clean source tree.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Dirty {
    Clean,
    Dirty,
    Unknown,
}

/// Facts about the calling application, never about this common library.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildInfo {
    pub command: String,
    pub version: String,
    pub commit: String,
    pub dirty: Dirty,
    pub compiler: String,
    pub target: String,
    pub profile: String,
}

impl BuildInfo {
    /// Reject control sequences and malformed revisions before embedding data.
    pub fn validate(&self) -> crate::Result<()> {
        for value in [
            &self.command,
            &self.version,
            &self.commit,
            &self.compiler,
            &self.target,
            &self.profile,
        ] {
            if value.is_empty()
                || value.len() > 512
                || !value.bytes().all(|b| (32..=126).contains(&b))
            {
                return Err("Build facts must be nonempty printable ASCII".into());
            }
        }
        if self.commit != "unknown"
            && (!matches!(self.commit.len(), 40 | 64)
                || !self.commit.bytes().all(|b| b.is_ascii_hexdigit())
                || self.commit.bytes().all(|b| b == b'0'))
        {
            return Err("Source revision must be a full commit hash or unknown".into());
        }
        Ok(())
    }
}
