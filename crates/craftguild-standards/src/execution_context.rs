//! Resolve selectors once; installation reuses recorded facts without tool probes.

use crate::{
    build_config::{identifier, Profile},
    config::{Artifact, Kind},
    execution_process::Runner,
};
use craftguild_core::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

pub const PROTOCOL: &str = "craftguild-execution-1";

#[derive(Clone, Debug, Default)]
pub struct Options {
    pub values: BTreeMap<String, String>,
    pub fallback_target: Option<String>,
    pub known_host: Option<String>,
    pub known_compiler: Option<String>,
    pub offline: bool,
    pub environment: BTreeMap<String, String>,
}

impl Options {
    pub fn parse(args: &[String]) -> Result<Self> {
        let mut options = Self::default();
        let mut args = args.iter();
        while let Some(flag) = args.next() {
            let (name, value) = match flag.as_str() {
                "--no-default-features" => ("no-default-features", "true".into()),
                "--default-features" => ("no-default-features", "false".into()),
                "--variant" | "--artifact" | "--target" | "--toolchain" | "--profile"
                | "--features" | "--target-dir" | "--prefix" | "--bindir" | "--destdir"
                | "--cancel-file" | "--timeout" => (
                    &flag[2..],
                    args.next().ok_or("Missing selector value")?.clone(),
                ),
                _ => return Err(format!("Unknown execution option: {flag}").into()),
            };
            if options.values.insert(name.into(), value).is_some() {
                return Err("Duplicate selector".into());
            }
        }
        Ok(options)
    }

    pub fn get(&self, name: &str) -> Option<String> {
        self.values.get(name).cloned().or_else(|| {
            let env = if name == "target-dir" {
                "CARGO_TARGET_DIR".into()
            } else {
                name.replace('-', "_").to_ascii_uppercase()
            };
            std::env::var(env).ok().filter(|s| !s.is_empty())
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Context {
    pub artifact: String,
    pub command: String,
    pub variant: Option<String>,
    pub explicit_target: bool,
    pub compiler: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub toolchain: Option<String>,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub compiler_version: String,
    pub target: String,
    pub profile: Profile,
    pub features: Vec<String>,
    pub no_default_features: bool,
    pub target_dir: PathBuf,
    pub output: PathBuf,
    pub outputs: Vec<PathBuf>,
    pub file: String,
}

impl Context {
    pub fn resolve(root: &Path, artifact: &Artifact, options: &Options) -> Result<Self> {
        let build = artifact.build.clone().unwrap_or_default();
        let variant = options.get("variant");
        let settings = build.settings(variant.as_deref())?;
        let requested_target = options
            .get("target")
            .or(settings.target)
            .or_else(|| artifact.target.clone());
        let explicit_target = requested_target.is_some();
        let toolchain = options.get("toolchain").or_else(|| build.toolchain.clone());
        if toolchain.as_deref().is_some_and(|s| !identifier(s)) {
            return Err("Invalid named toolchain".into());
        }
        let (host, compiler_version) = tooling(root, artifact.kind, toolchain.clone(), options)?;
        let target = match artifact.kind {
            Kind::Rust => requested_target
                .or_else(|| options.fallback_target.clone())
                .or_else(|| host.clone())
                .ok_or("No recorded target; build with an installed compiler first")?,
            Kind::Frontend => artifact.target.clone().unwrap_or_else(|| "browser".into()),
        };
        let settings = build.settings_for(variant.as_deref(), Some(&target), host.as_deref())?;
        if !identifier(&target) {
            return Err("Use a named target; custom target files are unsupported".into());
        }
        let profile = options
            .get("profile")
            .map(|s| Profile::parse(&s))
            .transpose()?
            .or(build.profile)
            .unwrap_or_default();
        let mut features = options
            .get("features")
            .map(|s| {
                s.split(',')
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_else(|| artifact.features.clone());
        features.sort();
        features.dedup();
        let no_default_features = match options.get("no-default-features").as_deref() {
            None => artifact.no_default_features,
            Some("true" | "1") => true,
            Some("false" | "0") => false,
            _ => return Err("NO_DEFAULT_FEATURES must be true or false".into()),
        };
        if artifact.kind == Kind::Frontend
            && (toolchain.is_some() || !features.is_empty() || no_default_features)
        {
            return Err("Rust selectors cannot be used for a frontend artifact".into());
        }
        let target_dir = options
            .get("target-dir")
            .or(build.target_dir)
            .unwrap_or_else(|| "target".into());
        let target_dir = absolute(root, &target_dir)?;
        let files = build
            .library
            .as_ref()
            .map(|library| library.filenames(&target))
            .unwrap_or_else(|| {
                vec![format!(
                    "{}{}",
                    artifact.command,
                    if target.contains("windows") {
                        ".exe"
                    } else {
                        ""
                    }
                )]
            });
        let mut context = Self {
            artifact: artifact.id.clone(),
            command: artifact.command.clone(),
            variant,
            explicit_target,
            compiler: settings.compiler,
            environment: BTreeMap::new(),
            toolchain,
            host,
            compiler_version,
            target,
            profile,
            features,
            no_default_features,
            target_dir,
            output: PathBuf::new(),
            outputs: vec![],
            file: files[0].clone(),
        };
        context.environment = settings
            .environment
            .into_iter()
            .map(|(name, value)| Ok((name, context.render(&value)?)))
            .collect::<Result<_>>()?;
        context.outputs = match artifact.kind {
            Kind::Rust => files
                .into_iter()
                .map(|file| {
                    let output = context.render(&settings.output.replace("{file}", &file))?;
                    craftguild_core::paths::relative(&output)?;
                    let base = match settings.output_root {
                        crate::build_settings::OutputRoot::TargetDirectory => {
                            context.target_dir.as_path()
                        }
                        crate::build_settings::OutputRoot::Repository => root,
                    };
                    let path = base.join(output);
                    if path.file_name() != Some(std::ffi::OsStr::new(&file)) {
                        return Err(
                            "Output template must preserve the compiler output filename".into()
                        );
                    }
                    if let Ok(relative) = path.strip_prefix(root) {
                        crate::build_config::output_path(
                            relative.to_str().ok_or("Non-UTF8 output")?,
                            &artifact.command,
                        )?;
                    }
                    Ok(path)
                })
                .collect::<Result<_>>()?,
            Kind::Frontend => {
                vec![root.join(&artifact.bundle.as_ref().ok_or("Missing bundle")?.directory)]
            }
        };
        context.output = context
            .outputs
            .first()
            .ok_or("Artifact has no output")?
            .clone();
        Ok(context)
    }

    pub fn render(&self, template: &str) -> Result<String> {
        let output = template
            .replace("{target}", &self.target)
            .replace(
                "{target_path}",
                &if self.explicit_target {
                    format!("{}/", self.target)
                } else {
                    String::new()
                },
            )
            .replace("{profile}", self.profile.name())
            .replace("{command}", &self.command)
            .replace("{file}", &self.file)
            .replace(
                "{exe}",
                if self.target.contains("windows") {
                    ".exe"
                } else {
                    ""
                },
            );
        if output.contains(['{', '}']) {
            return Err("Unknown execution substitution".into());
        }
        Ok(output)
    }

    pub fn artifact(&self, original: &Artifact) -> Artifact {
        let mut artifact = original.clone();
        artifact.features = self.features.clone();
        artifact.no_default_features = self.no_default_features;
        artifact
    }

    pub fn env(&self, prepared: &str, run: &str) -> Result<BTreeMap<String, String>> {
        Ok([
            ("CRAFTGUILD_ARTIFACT", self.artifact.clone()),
            ("CRAFTGUILD_COMMAND", self.command.clone()),
            ("CRAFTGUILD_HOST", self.host.clone().unwrap_or_default()),
            (
                "CRAFTGUILD_VARIANT",
                self.variant.clone().unwrap_or_default(),
            ),
            (
                "CRAFTGUILD_TOOLCHAIN",
                self.toolchain.clone().unwrap_or_default(),
            ),
            ("CRAFTGUILD_TARGET", self.target.clone()),
            ("CRAFTGUILD_PROFILE", self.profile.name().into()),
            (
                "CRAFTGUILD_FEATURES",
                serde_json::to_string(&self.features)?,
            ),
            (
                "CRAFTGUILD_TARGET_DIR",
                self.target_dir.to_string_lossy().into(),
            ),
            ("CRAFTGUILD_PREPARED", prepared.into()),
            ("CRAFTGUILD_RUN", run.into()),
            ("CRAFTGUILD_OUTPUT", self.output.to_string_lossy().into()),
            ("CRAFTGUILD_OUTPUTS", serde_json::to_string(&self.outputs)?),
        ]
        .into_iter()
        .map(|(k, v)| (k.into(), v))
        .collect())
    }
}

pub fn absolute(root: &Path, name: &str) -> Result<PathBuf> {
    let path = Path::new(name);
    if name.is_empty() || path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err("Output/destination path cannot be empty or traverse parents".into());
    }
    let path = if path.is_absolute() {
        if path.is_symlink() {
            return Err("Symlink output/destination leaf".into());
        }
        let mut ancestor = path;
        let mut missing = vec![];
        while !ancestor.exists() {
            missing.push(
                ancestor
                    .file_name()
                    .ok_or("Invalid absolute output path")?
                    .to_owned(),
            );
            ancestor = ancestor
                .parent()
                .ok_or("Missing absolute output ancestor")?;
        }
        let mut resolved = ancestor.canonicalize()?;
        for part in missing.into_iter().rev() {
            resolved.push(part);
        }
        resolved
    } else {
        root.join(path)
    };
    for parent in path.ancestors() {
        if parent.is_symlink() {
            return Err("Symlink in output/destination path".into());
        }
    }
    Ok(path)
}

fn tooling(
    root: &Path,
    kind: Kind,
    toolchain: Option<String>,
    options: &Options,
) -> Result<(Option<String>, String)> {
    if options.offline {
        return Ok((
            options.known_host.clone(),
            options.known_compiler.clone().unwrap_or_default(),
        ));
    }
    let runner = Runner {
        toolchain,
        ..Runner::default()
    };
    if kind == Kind::Rust {
        let facts = String::from_utf8(runner.capture(root, &["rustc".into(), "-vV".into()])?)?;
        let host = facts
            .lines()
            .find_map(|line| line.strip_prefix("host: ").map(str::to_owned))
            .ok_or("Compiler did not identify its host target")?;
        Ok((
            Some(host),
            facts
                .lines()
                .next()
                .ok_or("Compiler did not identify its version")?
                .into(),
        ))
    } else {
        Ok((
            None,
            format!(
                "node {}",
                String::from_utf8(runner.capture(root, &["node".into(), "--version".into()])?)?
                    .trim()
            ),
        ))
    }
}
