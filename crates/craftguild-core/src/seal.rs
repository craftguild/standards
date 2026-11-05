//! Prepared artifact records bind the SBOM to exact compiler and distribution inputs.

use crate::{paths, sbom, BuildInfo};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// One atomic prepared record; its paths are relative to the declared source root.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prepared {
    pub schema_version: u32,
    pub build: BuildInfo,
    /// Effective root features, not all workspace features or just requested aliases.
    pub features: Vec<String>,
    /// Required inputs include lockfiles, manifests, sources and embedded assets.
    pub inputs: BTreeMap<String, String>,
    /// Complete directory inventories detect additions as well as changed hashes.
    pub inventories: BTreeMap<String, BTreeSet<String>>,
    pub sbom: String,
    pub sbom_sha256: String,
}

impl Prepared {
    /// Digest only inputs and inventories, avoiding an SBOM/seal hash cycle.
    pub fn input_digest(&self) -> crate::Result<String> {
        Ok(paths::digest(&serde_json::to_vec(&(
            &self.inputs,
            &self.inventories,
        ))?))
    }

    /// Schema 2 separates consumer SBOM identity from the local source input ledger.
    pub fn properties(&self) -> crate::Result<serde_json::Value> {
        match self.schema_version {
            2 => return sbom::identity_properties(&self.build, &self.features),
            1 => {}
            _ => return Err("Unsupported prepared seal schema".into()),
        }
        // Preserve validation of already-published schema 1 artifacts.
        let mut properties = sbom::properties(&self.build, &self.features, &self.input_digest()?)?;
        let values = properties.as_array_mut().expect("Properties are an array");
        values.push(serde_json::json!({"name":"craftguild:inputs", "value":serde_json::to_string(&self.inputs)?}));
        values.push(serde_json::json!({"name":"craftguild:inventories", "value":serde_json::to_string(&self.inventories)?}));
        Ok(properties)
    }

    /// Refuse stale or incomplete records, then return exactly the prepared bytes.
    pub fn verify(
        &self,
        root: &Path,
        expected: &BuildInfo,
        features: &[String],
    ) -> crate::Result<Vec<u8>> {
        self.build.validate()?;
        if !matches!(self.schema_version, 1 | 2) || self.inputs.is_empty() {
            return Err("Invalid or empty prepared input record".into());
        }
        if &self.build != expected || self.features != features {
            return Err("Prepared artifact identity does not match this build".into());
        }
        for (directory, expected) in &self.inventories {
            if paths::inventory(root, directory)? != *expected {
                return Err(format!("Changed input inventory: {directory}").into());
            }
            if !expected.iter().all(|p| self.inputs.contains_key(p)) {
                return Err("Inventory contains unsealed files".into());
            }
        }
        for (name, hash) in &self.inputs {
            if paths::hash_file(root, name)? != *hash {
                return Err(format!("Stale prepared input: {name}").into());
            }
        }
        let bytes = std::fs::read(paths::owned(root, &self.sbom)?)?;
        if paths::digest(&bytes) != self.sbom_sha256 {
            return Err("SBOM bytes do not match prepared digest".into());
        }
        let bom = sbom::validate(&bytes)?;
        if bom["metadata"]["component"]["name"] != self.build.command
            || bom["metadata"]["component"]["version"] != self.build.version
            || bom["metadata"]["properties"] != self.properties()?
        {
            return Err("SBOM does not describe the prepared application/inputs".into());
        }
        Ok(bytes)
    }
}
