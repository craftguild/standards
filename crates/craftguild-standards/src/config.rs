//! Strict versioned configuration, shared by build and quality entry points.

use craftguild_core::{paths, Result};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub schema_version: u32,
    pub standards_version: String,
    pub operations: Option<Vec<crate::operation_config::Operation>>,
    pub rust_output_layout: Option<crate::installer_config::RustOutputLayout>,
    pub installers: Option<Vec<crate::installer_config::Installer>>,
    pub artifacts: Vec<Artifact>,
    #[serde(default)]
    pub sources: Vec<Source>,
    #[serde(default)]
    pub commands: Vec<CheckCommand>,
    #[serde(default)]
    pub adapters: Vec<Adapter>,
    #[serde(default)]
    pub generated: Vec<Generated>,
    #[serde(default)]
    pub policies: Vec<Policy>,
    #[serde(default)]
    pub releases: Vec<crate::release::versions::Group>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub id: String,
    pub command: String,
    pub project: String,
    pub kind: Kind,
    pub manifest: String,
    pub lockfile: String,
    pub package: String,
    pub target: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub no_default_features: bool,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub input_dirs: Vec<String>,
    /// Bundler metadata mapped to npm package-lock v3 for shipped Web code.
    pub bundle: Option<Bundle>,
    pub supplemental: Option<String>,
    pub build: Option<crate::build_config::Build>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    Rust,
    Frontend,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub lockfile: String,
    pub metafile: String,
    pub directory: String,
    pub command: Vec<String>,
    /// Files below this directory in the bundler metadata resolve to lock entries.
    pub root: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub pattern: String,
    pub role: Role,
    pub reason: String,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Role {
    Implementation,
    Test,
    TestHelper,
    Fixture,
    Documentation,
    Vendored,
    Generated,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckCommand {
    pub id: String,
    pub purpose: String,
    pub argv: Vec<String>,
    pub phase: Phase,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Phase {
    Test,
    Check,
    Acceptance,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Adapter {
    pub language: Language,
    pub test: Vec<String>,
    pub lint: Vec<String>,
    pub format: Option<Vec<String>>,
    pub typecheck: Option<Vec<String>>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Language {
    Rust,
    Npm,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Generated {
    pub owner: String,
    pub manifest: String,
    pub output_root: String,
    pub reproduce: Vec<String>,
    #[serde(default)]
    pub rust_size_exception: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Policy {
    Publication {
        repository: String,
        manifest: Option<String>,
        #[serde(default)]
        npm_lockfiles: Vec<String>,
    },
    Dependencies {
        manifest: String,
        /// Every named first-party package has an exhaustive list of allowed edges.
        layers: std::collections::BTreeMap<String, Vec<String>>,
        #[serde(default)]
        restricted: std::collections::BTreeMap<String, Vec<String>>,
    },
    SeaOrm {
        manifest: String,
        sources: Vec<String>,
        allowed_packages: Vec<String>,
    },
}

pub fn load(root: &Path) -> Result<Config> {
    let text = std::fs::read_to_string(paths::owned(root, "craftguild.toml")?)?;
    let config: Config = toml::from_str(&text)?;
    config.validate()?;
    Ok(config)
}

impl Config {
    pub fn artifact(&self, id: &str) -> Result<&Artifact> {
        self.artifacts
            .iter()
            .find(|a| a.id == id)
            .ok_or_else(|| format!("Unknown artifact: {id}").into())
    }

    pub fn validate(&self) -> Result<()> {
        crate::operation_config::validate(self)?;
        crate::release::versions::validate(&self.releases)?;
        crate::installer_config::validate(self)?;
        if !matches!(self.schema_version, 1 | 2)
            || self.standards_version != "0.1.0"
            || self.artifacts.is_empty()
        {
            return Err("Unsupported configuration version or empty artifacts".into());
        }
        let mut ids = BTreeSet::new();
        let mut commands = BTreeSet::new();
        for a in &self.artifacts {
            if self.schema_version == 1 && a.build.is_some() {
                return Err("Artifact build settings require configuration schema 2".into());
            }
            if let Some(build) = &a.build {
                build.validate(a)?;
            }
            for value in [&a.id, &a.command, &a.project, &a.package] {
                if value.is_empty()
                    || !value
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
                {
                    return Err("Artifact names must be portable ASCII identifiers".into());
                }
            }
            if !ids.insert(&a.id) {
                return Err("Duplicate artifact id".into());
            }
            if !commands.insert(&a.command) {
                return Err("Artifact commands must uniquely identify prepared records".into());
            }
            for p in [&a.manifest, &a.lockfile]
                .into_iter()
                .chain(&a.inputs)
                .chain(&a.input_dirs)
            {
                paths::relative(p)?;
            }
            if let Some(path) = &a.supplemental {
                paths::relative(path)?;
            }
            if a.kind == Kind::Frontend && a.bundle.is_none() {
                return Err("Frontend artifacts require bundle evidence".into());
            }
            if a.kind == Kind::Frontend && (!a.features.is_empty() || a.no_default_features) {
                return Err("Cargo feature selection applies only to Rust artifacts".into());
            }
            if let Some(b) = &a.bundle {
                crate::build_config::output_path(&b.directory, &a.command)?;
                crate::build_config::output_path(&b.metafile, &a.command)?;
                if b.root != "." {
                    paths::relative(&b.root)?;
                }
                for p in [&b.directory, &b.lockfile, &b.metafile] {
                    paths::relative(p)?;
                }
                validate_argv(&b.command)?;
            }
        }
        crate::build_config::artifact_order(self, None)?;
        ids.clear();
        for c in &self.commands {
            if c.id.is_empty() || c.purpose.trim().is_empty() || !ids.insert(&c.id) {
                return Err("Repository commands require unique ids and a purpose".into());
            }
            validate_argv(&c.argv)?;
        }
        for s in &self.sources {
            if s.reason.trim().is_empty() || s.pattern.is_empty() {
                return Err("Source roles require a pattern and responsibility evidence".into());
            }
            glob(&s.pattern)?;
        }
        for adapter in &self.adapters {
            validate_argv(&adapter.test)?;
            validate_argv(&adapter.lint)?;
            if adapter.language == Language::Rust && adapter.format.is_none() {
                return Err("Rust adapters require formatting".into());
            }
            if adapter.language == Language::Npm && adapter.typecheck.is_none() {
                return Err("npm adapters require type checking".into());
            }
            for argv in [&adapter.format, &adapter.typecheck].into_iter().flatten() {
                validate_argv(argv)?;
            }
        }
        for group in &self.generated {
            paths::relative(&group.manifest)?;
            paths::relative(&group.output_root)?;
            validate_argv(&group.reproduce)?;
            if group.owner.trim().is_empty() || !group.reproduce.iter().any(|a| a == "{output}") {
                return Err("Generated ownership needs an owner and isolated {output} reproduction argument".into());
            }
        }
        for policy in &self.policies {
            match policy {
                Policy::Publication {
                    repository,
                    manifest,
                    npm_lockfiles,
                } => {
                    let parts: Vec<_> = repository.split('/').collect();
                    if parts.len() != 2
                        || parts.iter().any(|s| {
                            s.is_empty()
                                || !s
                                    .bytes()
                                    .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
                        })
                    {
                        return Err(
                            "Publication repository must be an owner/name identifier".into()
                        );
                    }
                    if manifest.is_none() && npm_lockfiles.is_empty() {
                        return Err("Empty publication source scope".into());
                    }
                    for path in manifest.iter().chain(npm_lockfiles) {
                        paths::relative(path)?;
                    }
                }
                Policy::Dependencies {
                    manifest, layers, ..
                } => {
                    paths::relative(manifest)?;
                    if layers.is_empty() {
                        return Err("Empty dependency layer policy".into());
                    }
                }
                Policy::SeaOrm {
                    manifest,
                    sources,
                    allowed_packages,
                } => {
                    paths::relative(manifest)?;
                    if sources.is_empty() || allowed_packages.is_empty() {
                        return Err("Empty SeaORM policy scope".into());
                    }
                    for pattern in sources {
                        glob(pattern)?;
                    }
                }
            }
        }
        Ok(())
    }
}

/// Repository globs use path components; `*` never crosses a directory separator.
pub fn glob(pattern: &str) -> Result<globset::Glob> {
    Ok(globset::GlobBuilder::new(pattern)
        .literal_separator(true)
        .build()?)
}

fn validate_argv(argv: &[String]) -> Result<()> {
    if argv.is_empty() || argv.iter().any(|s| s.is_empty() || s.contains('\0')) {
        return Err("Required command argv must be nonempty".into());
    }
    Ok(())
}
