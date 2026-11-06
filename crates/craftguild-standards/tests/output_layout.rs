//! Offline resolver matrices complement real Cargo build/install/package tests.

use craftguild_standards::{
    config::Config,
    execution_context::{Context, Options},
};

fn source(extra: &str, build: &str) -> String {
    format!(
        r#"schema_version=2
standards_version="0.1.0"
{extra}
[[artifacts]]
id="demo"
command="demo"
project="demo"
kind="rust"
manifest="Cargo.toml"
lockfile="Cargo.lock"
package="demo"
{build}
"#
    )
}

#[test]
fn canonical_matrix_retains_target_profile_filenames_variants_and_target_root() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    for target in [
        "aarch64-apple-darwin",
        "x86_64-unknown-linux-gnu",
        "x86_64-pc-windows-msvc",
        "wasm32-unknown-unknown",
    ] {
        for profile in ["debug", "release"] {
            for library in [false, true] {
                let build = format!(
                    r#"[artifacts.build]
target_dir="configured"
[[artifacts.build.variants]]
id="special"
target="{target}"
{}"#,
                    if library {
                        "[artifacts.build.library]\nname=\"demo_lib\"\nformats=[\"cdylib\",\"staticlib\",\"rlib\"]\n"
                    } else {
                        ""
                    }
                );
                let config: Config =
                    toml::from_str(&source("rust_output_layout=\"standard\"", &build)).unwrap();
                config.validate().unwrap();
                for variant in [false, true] {
                    let mut options =
                        Options::parse(&["--profile".into(), profile.into()]).unwrap();
                    options.offline = true;
                    options.standard_layout = true;
                    options.known_host = Some(target.into());
                    if variant {
                        options.values.insert("variant".into(), "special".into());
                    }
                    let context =
                        Context::resolve(root.as_path(), &config.artifacts[0], &options).unwrap();
                    assert_eq!(context.explicit_target, variant);
                    assert_eq!(context.outputs.len(), if library { 3 } else { 1 });
                    for output in &context.outputs {
                        assert_eq!(
                            output.parent().unwrap(),
                            root.as_path().join("configured").join(target).join(profile)
                        );
                    }
                    if library {
                        let names = config.artifacts[0]
                            .build
                            .as_ref()
                            .unwrap()
                            .library
                            .as_ref()
                            .unwrap()
                            .filenames(target);
                        assert_eq!(
                            context
                                .outputs
                                .iter()
                                .map(|p| p.file_name().unwrap().to_str().unwrap())
                                .collect::<Vec<_>>(),
                            names
                        );
                    }
                    options
                        .values
                        .insert("target-dir".into(), "override".into());
                    assert!(
                        Context::resolve(root.as_path(), &config.artifacts[0], &options)
                            .unwrap()
                            .output
                            .starts_with(root.as_path().join("override"))
                    );
                }
            }
        }
    }
}

#[test]
fn strict_layout_validation_and_legacy_templates() {
    for build in [
        "[artifacts.build]\noutput=\"{target}/{profile}/{file}\"",
        "[artifacts.build]\noutput_root=\"target-directory\"",
        "[[artifacts.build.targets]]\ntarget=\"aarch64-apple-darwin\"\noutput=\"{profile}/{file}\"",
        "[[artifacts.build.targets]]\ntarget=\"aarch64-apple-darwin\"\noutput_root=\"repository\"",
        "[[artifacts.build.variants]]\nid=\"special\"\noutput=\"{profile}/{file}\"",
        "[[artifacts.build.variants]]\nid=\"special\"\noutput_root=\"repository\"",
    ] {
        let config: Config =
            toml::from_str(&source("rust_output_layout=\"standard\"", build)).unwrap();
        assert!(config.validate().is_err());
        let legacy: Config = toml::from_str(&source("", build)).unwrap();
        legacy.validate().unwrap();
    }
    assert!(toml::from_str::<Config>(&source("rust_output_layout=\"other\"", "")).is_err());
    let text = source("rust_output_layout=\"standard\"", "")
        .replace("schema_version=2", "schema_version=1");
    assert!(toml::from_str::<Config>(&text).unwrap().validate().is_err());
}

#[test]
fn standard_mode_keeps_frontend_bundle_placement_without_rust_tools() {
    let root = tempfile::tempdir().unwrap();
    let canonical = root.path().canonicalize().unwrap();
    let text = source(
        "rust_output_layout=\"standard\"",
        r#"[artifacts.bundle]
root="web"
lockfile="web/package-lock.json"
metafile="web/metafile.json"
directory="web/dist"
command=["npm","run","build"]
"#,
    )
    .replace("kind=\"rust\"", "kind=\"frontend\"");
    let config: Config = toml::from_str(&text).unwrap();
    config.validate().unwrap();
    let options = Options {
        offline: true,
        standard_layout: true,
        known_compiler: Some("node fixture".into()),
        ..Options::default()
    };
    let context = Context::resolve(&canonical, &config.artifacts[0], &options).unwrap();
    assert_eq!(context.output, canonical.join("web/dist"));
    assert_eq!(context.target, "browser");
}
