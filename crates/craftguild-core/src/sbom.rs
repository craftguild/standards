//! Offline validation against the pinned official CycloneDX 1.5 JSON schemas.

use jsonschema::{Draft, Resource, Validator};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::sync::OnceLock;

const BOM: &str = include_str!("../../../schemas/bom-1.5.schema.json");
const SPDX: &str = include_str!("../../../schemas/spdx.schema.json");
const JSF: &str = include_str!("../../../schemas/jsf-0.82.schema.json");

fn validator() -> &'static Validator {
    static VALIDATOR: OnceLock<Validator> = OnceLock::new();
    VALIDATOR.get_or_init(|| {
        let resource =
            |data: &str| Resource::from_contents(serde_json::from_str(data).unwrap()).unwrap();
        jsonschema::options()
            .with_draft(Draft::Draft7)
            .with_resource(
                "http://cyclonedx.org/schema/spdx.schema.json",
                resource(SPDX),
            )
            .with_resource(
                "http://cyclonedx.org/schema/jsf-0.82.schema.json",
                resource(JSF),
            )
            .build(&serde_json::from_str::<Value>(BOM).unwrap())
            .expect("Vendored schemas must compile without external retrieval")
    })
}

/// Validate the standard schema, fixed version and unambiguous component graph.
pub fn validate(data: &[u8]) -> crate::Result<Value> {
    let bom: Value = serde_json::from_slice(data)?;
    validator().validate(&bom).map_err(|e| e.to_string())?;
    if bom["bomFormat"] != "CycloneDX" || bom["specVersion"] != "1.5" {
        return Err("Expected CycloneDX 1.5 JSON".into());
    }
    let root = bom["metadata"]["component"]["bom-ref"]
        .as_str()
        .ok_or("Missing artifact component identity")?;
    let mut refs = BTreeSet::from([root]);
    for component in bom["components"].as_array().ok_or("Missing components")? {
        let id = component["bom-ref"]
            .as_str()
            .ok_or("Missing component identity")?;
        if !refs.insert(id) {
            return Err("Duplicate component identity".into());
        }
    }
    let mut seen = BTreeSet::new();
    for dep in bom["dependencies"]
        .as_array()
        .ok_or("Missing dependency graph")?
    {
        let id = dep["ref"].as_str().ok_or("Missing dependency identity")?;
        if !refs.contains(id) || !seen.insert(id) {
            return Err("Unknown or duplicate dependency identity".into());
        }
        for target in dep["dependsOn"]
            .as_array()
            .ok_or("Missing dependency edges")?
        {
            if !refs.contains(target.as_str().ok_or("Invalid dependency edge")?) {
                return Err("Dependency edge points outside the SBOM".into());
            }
        }
    }
    if seen != refs {
        return Err("Dependency graph does not cover every component".into());
    }
    Ok(bom)
}

/// Legacy schema 1 properties, retained for reading existing sealed artifacts.
/// New producers use `identity_properties` and keep the source ledger local.
pub fn properties(
    info: &crate::BuildInfo,
    features: &[String],
    seal: &str,
) -> crate::Result<Value> {
    let mut properties = identity_properties(info, features)?;
    properties
        .as_array_mut()
        .expect("Identity properties are an array")
        .push(json!({"name": "craftguild:inputs-sha256", "value": seal}));
    Ok(properties)
}

/// Consumer-facing identity excludes the developer's local source input ledger.
pub fn identity_properties(info: &crate::BuildInfo, features: &[String]) -> crate::Result<Value> {
    Ok(json!([
        {"name": "craftguild:build", "value": serde_json::to_string(info)?},
        {"name": "craftguild:features", "value": serde_json::to_string(features)?}
    ]))
}
