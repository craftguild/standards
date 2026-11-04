//! Configurable dependency-layer and SeaORM policies shared across repositories.

use crate::{command, config::Policy, sea_orm};
use craftguild_core::{paths, Result};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

pub fn check(
    root: &Path,
    policies: &[Policy],
    sources: &BTreeSet<String>,
) -> Result<Vec<(String, String)>> {
    let mut errors = vec![];
    for policy in policies {
        let manifest = match policy {
            Policy::Dependencies { manifest, .. } | Policy::SeaOrm { manifest, .. } => manifest,
        };
        let meta: Value = serde_json::from_slice(&command::capture(
            root,
            &[
                "cargo".into(),
                "metadata".into(),
                "--locked".into(),
                "--no-deps".into(),
                "--format-version".into(),
                "1".into(),
                "--manifest-path".into(),
                manifest.clone(),
            ],
        )?)?;
        let packages = meta["packages"]
            .as_array()
            .ok_or("Missing dependency metadata")?;
        match policy {
            Policy::Dependencies {
                layers, restricted, ..
            } => {
                let names: BTreeSet<_> =
                    packages.iter().filter_map(|p| p["name"].as_str()).collect();
                for name in layers.keys() {
                    if !names.contains(name.as_str()) {
                        return Err(format!("Policy names absent package: {name}").into());
                    }
                }
                for p in packages {
                    let name = p["name"].as_str().ok_or("Package has no name")?;
                    if !layers.contains_key(name) {
                        return Err(
                            format!("Dependency policy has no layer for package: {name}").into(),
                        );
                    }
                    for d in p["dependencies"]
                        .as_array()
                        .ok_or("Missing package dependencies")?
                    {
                        let dependency = d["name"].as_str().ok_or("Dependency has no name")?;
                        if (layers.contains_key(dependency)
                            && !layers[name].iter().any(|s| s == dependency))
                            || restricted
                                .get(dependency)
                                .is_some_and(|allowed| !allowed.iter().any(|s| s == name))
                        {
                            errors.push((
                                "CG-DEP-001".into(),
                                format!("Forbidden dependency: {name} -> {dependency}"),
                            ));
                        }
                    }
                }
            }
            Policy::SeaOrm {
                sources: patterns,
                allowed_packages,
                ..
            } => {
                let mut drivers: BTreeSet<String> =
                    ["sqlx", "rusqlite", "libsqlite3_sys", "sea_query"]
                        .map(str::to_owned)
                        .into();
                for p in packages {
                    let name = p["name"].as_str().ok_or("Package has no name")?;
                    for d in p["dependencies"]
                        .as_array()
                        .ok_or("Missing package dependencies")?
                    {
                        let dependency = d["name"].as_str().ok_or("Dependency has no name")?;
                        if matches!(
                            dependency,
                            "sqlx" | "rusqlite" | "libsqlite3-sys" | "sea-query"
                        ) {
                            drivers.insert(
                                d["rename"].as_str().unwrap_or(dependency).replace('-', "_"),
                            );
                            errors.push(("CG-ORM-001".into(),format!("Direct database driver/query dependency: {name} -> {dependency}")));
                        }
                        if dependency.starts_with("sea-orm")
                            && !allowed_packages.iter().any(|s| s == name)
                        {
                            errors.push((
                                "CG-ORM-002".into(),
                                format!("ORM dependency outside permitted packages: {name}"),
                            ));
                        }
                        if dependency == "sea-orm"
                            && d["features"]
                                .as_array()
                                .is_some_and(|v| v.iter().any(|f| f == "schema-sync"))
                        {
                            errors.push((
                                "CG-ORM-003".into(),
                                format!("Automatic model sync opt-in: {name}"),
                            ));
                        }
                    }
                }
                let patterns = patterns
                    .iter()
                    .map(|p| Ok(crate::config::glob(p)?.compile_matcher()))
                    .collect::<Result<Vec<_>>>()?;
                let selected: Vec<_> = sources
                    .iter()
                    .filter(|name| patterns.iter().any(|p| p.is_match(name)))
                    .collect();
                if selected.is_empty() {
                    return Err("SeaORM source policy matched no files".into());
                }
                for name in selected {
                    if name.ends_with(".sql") {
                        errors.push(("CG-ORM-004".into(), format!("Raw SQL source: {name}")));
                    } else if name.ends_with(".rs") {
                        for violation in sea_orm::check(
                            &std::fs::read_to_string(paths::owned(root, name)?)?,
                            &drivers,
                        )? {
                            errors.push((
                                "CG-ORM-004".into(),
                                format!("{name}: prohibited database API {violation}"),
                            ));
                        }
                    }
                }
            }
        }
    }
    Ok(errors)
}
