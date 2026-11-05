//! Real Cargo subprocesses exercise execution ordering and publication boundaries.

use serde_json::Value;
use std::path::Path;
use std::process::{Command, Output};

fn write(root: &Path, name: &str, bytes: impl AsRef<[u8]>) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
}

fn fixture(hooks: &str) -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "Cargo.toml",
        "[package]\nname='demo'\nversion='1.2.3'\nedition='2021'\n[features]\noptional-part=[]\n",
    );
    write(
        root,
        "src/main.rs",
        r#"use std::io::Write;
fn main() {
    if std::env::args().nth(1).as_deref() == Some("sbom") {
        std::io::stdout().write_all(include_bytes!(env!("FIXTURE_SBOM"))).unwrap();
    } else { println!("{}:{}", env!("CRAFTGUILD_PROFILE"), cfg!(feature="optional-part")); }
}
"#,
    );
    write(
        root,
        "build.rs",
        r#"fn main() {
    let record = std::env::var("CRAFTGUILD_PREPARED").unwrap();
    let bom = std::path::Path::new(&record).parent().unwrap().join("bom.json");
    println!("cargo:rustc-env=FIXTURE_SBOM={}/{}", std::env::var("CARGO_MANIFEST_DIR").unwrap(), bom.display());
    println!("cargo:rerun-if-env-changed=CRAFTGUILD_PREPARED");
}
"#,
    );
    write(
        root,
        "craftguild.toml",
        format!(
            r#"schema_version=2
standards_version="0.1.0"
[[artifacts]]
id="demo"
command="demo"
project="demo"
kind="rust"
manifest="Cargo.toml"
lockfile="Cargo.lock"
package="demo"
{hooks}
"#
        ),
    );
    assert!(Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(root)
        .status()
        .unwrap()
        .success());
    temp
}

fn cli(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_craftguild-standards-bootstrap"))
        .args(args)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("TARGET")
        .env_remove("PROFILE")
        .current_dir(root)
        .output()
        .unwrap()
}

fn success(output: Output) -> Output {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn receipt(root: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(root.join(".craftguild/demo/receipt.json")).unwrap())
        .unwrap()
}

#[test]
fn actual_debug_release_features_output_and_copy_only_install() {
    let temp = fixture("");
    let root = temp.path();
    for profile in ["debug", "release"] {
        success(cli(
            root,
            &[
                "build",
                "--profile",
                profile,
                "--features",
                "optional-part",
                "--target-dir",
                "products",
            ],
        ));
        let receipt = receipt(root);
        assert_eq!(receipt["context"]["profile"], profile);
        assert_eq!(receipt["prepared"]["build"]["profile"], profile);
        assert_eq!(
            receipt["prepared"]["features"],
            serde_json::json!(["optional-part"])
        );
        let binary = Path::new(receipt["context"]["output"].as_str().unwrap());
        assert_eq!(
            Command::new(binary).output().unwrap().stdout,
            format!("{profile}:true\n").as_bytes()
        );
        let bom = root.join(receipt["prepared"]["sbom"].as_str().unwrap());
        assert_eq!(
            Command::new(binary).arg("sbom").output().unwrap().stdout,
            std::fs::read(bom).unwrap()
        );
        let staging = root.join("stage");
        let installed = Command::new(env!("CARGO_BIN_EXE_craftguild-standards-bootstrap"))
            .args([
                "install",
                "--profile",
                profile,
                "--features",
                "optional-part",
                "--target-dir",
                "products",
                "--prefix",
                "/opt/demo",
                "--destdir",
            ])
            .arg(&staging)
            .current_dir(root)
            .env("PATH", "")
            .env("RUSTC", "missing")
            .output()
            .unwrap();
        success(installed);
        assert_eq!(
            std::fs::read(staging.join("opt/demo/bin/demo")).unwrap(),
            std::fs::read(binary).unwrap()
        );
        assert!(!cli(
            root,
            &["install", "--profile", profile, "--target-dir", "products"]
        )
        .status
        .success());
    }
    let receipt = receipt(root);
    let binary = receipt["context"]["output"].as_str().unwrap();
    std::fs::write(binary, "tampered").unwrap();
    assert!(!cli(
        root,
        &[
            "install",
            "--features",
            "optional-part",
            "--target-dir",
            "products",
            "--destdir",
            "stage"
        ]
    )
    .status
    .success());
}

const HOOKS: &str = r#"
[artifacts.build]
[[artifacts.build.hooks]]
id="sdk"
stage="prerequisite"
argv=["python3","hook.py","sdk"]
inputs=["hook.py"]
[[artifacts.build.hooks]]
id="generate"
stage="prepare"
argv=["python3","hook.py","generate"]
inputs=["hook.py","source.txt"]
outputs=["generated.txt"]
depends_on=["sdk"]
[[artifacts.build.hooks]]
id="ownership"
stage="evidence"
argv=["python3","hook.py","ownership"]
inputs=["generated.txt"]
outputs=[".craftguild/derived/ownership.json"]
depends_on=["generate"]
[[artifacts.build.hooks]]
id="gate"
stage="verify"
argv=["python3","hook.py","gate"]
inputs=["generated.txt"]
depends_on=["ownership"]
"#;

const SCRIPT: &str = r#"import os, pathlib, sys
stage = sys.argv[1]
with open('order.txt','a') as stream: stream.write(stage + '\n')
assert os.environ['CRAFTGUILD_ARTIFACT'] == 'demo'
assert os.environ['CRAFTGUILD_PROFILE'] == 'debug'
assert os.environ['CRAFTGUILD_TARGET']
assert os.environ['CRAFTGUILD_PREPARED'].endswith('prepared.json')
if stage == 'generate':
    pathlib.Path('generated.txt').write_text(pathlib.Path('source.txt').read_text())
if stage == 'ownership':
    pathlib.Path('.craftguild/derived/ownership.json').write_text('{"verified":true}')
if stage == 'gate' and pathlib.Path('fail').exists(): sys.exit(7)
"#;

#[test]
fn hooks_order_context_evidence_and_failed_build_preserve_success() {
    let temp = fixture(HOOKS);
    let root = temp.path();
    write(root, "hook.py", SCRIPT);
    write(root, "source.txt", "first");
    success(cli(root, &["build", "--profile", "debug"]));
    assert_eq!(
        std::fs::read_to_string(root.join("order.txt")).unwrap(),
        "sdk\ngenerate\nownership\ngate\n"
    );
    let first = receipt(root);
    let binary = first["context"]["output"].as_str().unwrap();
    let bytes = std::fs::read(binary).unwrap();
    assert!(first["prepared"]["inputs"].get("generated.txt").is_some());
    assert!(first["prepared"]["inputs"]
        .get(".craftguild/derived/ownership.json")
        .is_some());
    write(root, "source.txt", "second");
    write(root, "fail", "");
    assert!(!cli(root, &["build", "--profile", "debug"]).status.success());
    assert_eq!(std::fs::read(binary).unwrap(), bytes);
    assert_eq!(receipt(root), first);
    assert_eq!(
        std::fs::read_to_string(root.join("generated.txt")).unwrap(),
        "first"
    );
    let runs = std::fs::read_dir(root.join(".craftguild/demo/runs")).unwrap();
    let results: Vec<Value> = runs
        .map(|entry| {
            serde_json::from_slice(
                &std::fs::read(entry.unwrap().path().join("result.json")).unwrap(),
            )
            .unwrap()
        })
        .collect();
    assert_eq!(results.len(), 2);
    let failed = results.iter().find(|r| r["state"] == "failed").unwrap();
    assert!(failed["steps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s["id"] == "hook:gate" && s["state"] == "failed"));
    assert!(failed["steps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s["id"] == "publish" && s["state"] == "pending"));
}

#[test]
fn strict_schema_and_cycle_refusal_precede_commands() {
    let temp = fixture(HOOKS);
    let root = temp.path();
    let source = std::fs::read_to_string(root.join("craftguild.toml")).unwrap();
    for invalid in [
        source.replace("schema_version=2", "schema_version=1"),
        source.replace("depends_on=[\"sdk\"]", "depends_on=[\"ownership\"]"),
        source.replace("stage=\"prepare\"", "stage=\"unknown\""),
        source.replace("[artifacts.build]", "[artifacts.build]\nunknown=true"),
    ] {
        write(root, "craftguild.toml", invalid);
        assert!(!cli(root, &["build"]).status.success());
        assert!(!root.join("order.txt").exists());
    }
}

#[test]
fn missing_hook_outputs_cannot_reuse_previous_files() {
    let temp = fixture(HOOKS);
    let root = temp.path();
    write(root, "source.txt", "input");
    write(root, "generated.txt", "old output");
    write(root, "hook.py", "pass\n");
    assert!(!cli(root, &["build", "--profile", "debug"]).status.success());
    assert_eq!(
        std::fs::read_to_string(root.join("generated.txt")).unwrap(),
        "old output"
    );
    assert!(!root.join(".craftguild/demo/receipt.json").exists());
}

#[cfg(unix)]
#[test]
fn signal_cancel_file_and_timeout_stop_descendants_and_restore_outputs() {
    use std::time::{Duration, Instant};
    for cancellation in ["signal", "file", "timeout"] {
        let temp = fixture(
            r#"
[artifacts.build]
[[artifacts.build.hooks]]
id="wait"
stage="prepare"
argv=["python3","wait.py"]
inputs=["wait.py"]
outputs=["generated.txt"]
"#,
        );
        let root = temp.path().canonicalize().unwrap();
        write(&root, "generated.txt", "previous verified output");
        write(
            &root,
            "wait.py",
            r#"import pathlib, subprocess, sys, time
pathlib.Path('generated.txt').write_text('unverified candidate')
subprocess.Popen([sys.executable, '-c', "import pathlib,time; time.sleep(2); pathlib.Path('survived').write_text('bad')"])
pathlib.Path('ready').write_text('ready')
time.sleep(120)
"#,
        );
        let output = std::fs::File::create(root.join("engine.log")).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_craftguild-standards-bootstrap"));
        command
            .args([
                "build",
                "--profile",
                "debug",
                "--cancel-file",
                "cancel",
                "--timeout",
                if cancellation == "timeout" { "1" } else { "30" },
            ])
            .current_dir(&root)
            .stdout(output.try_clone().unwrap())
            .stderr(output);
        let mut child = command.spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while !root.join("ready").exists() && Instant::now() < deadline {
            assert!(
                child.try_wait().unwrap().is_none(),
                "Engine exited before hook started"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(root.join("ready").exists());
        match cancellation {
            "signal" => {
                assert!(Command::new("kill")
                    .args(["-TERM", "--", &child.id().to_string()])
                    .status()
                    .unwrap()
                    .success());
            }
            "file" => write(&root, "cancel", "cancel"),
            _ => {}
        }
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                panic!("Cancellation did not terminate execution");
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        assert!(!status.success());
        assert_eq!(
            std::fs::read_to_string(root.join("generated.txt")).unwrap(),
            "previous verified output",
            "{cancellation}: {}",
            std::fs::read_to_string(root.join("engine.log")).unwrap()
        );
        assert!(!root.join(".craftguild/demo/receipt.json").exists());
        let run = std::fs::read_dir(root.join(".craftguild/demo/runs"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let record: Value =
            serde_json::from_slice(&std::fs::read(run.join("result.json")).unwrap()).unwrap();
        assert_eq!(
            record["state"],
            if cancellation == "timeout" {
                "failed"
            } else {
                "cancelled"
            }
        );
        std::thread::sleep(Duration::from_millis(2100));
        assert!(
            !root.join("survived").exists(),
            "A descendant outlived cancellation"
        );
    }
}

#[test]
fn compiler_failure_and_stale_inputs_preserve_previous_binary_and_receipt() {
    let temp = fixture("");
    let root = temp.path();
    success(cli(root, &["build", "--profile", "debug"]));
    let first = receipt(root);
    let binary = first["context"]["output"].as_str().unwrap();
    let original = std::fs::read(binary).unwrap();
    write(root, "src/main.rs", "invalid Rust source");
    assert!(!cli(root, &["build", "--profile", "debug"]).status.success());
    assert_eq!(receipt(root), first);
    assert_eq!(std::fs::read(binary).unwrap(), original);

    let temp = fixture(HOOKS);
    let root = temp.path();
    write(root, "source.txt", "first");
    write(root, "hook.py", format!("{SCRIPT}\nif stage == 'gate': pathlib.Path('source.txt').write_text('changed after seal')\n"));
    assert!(!cli(root, &["build", "--profile", "debug"]).status.success());
    assert!(!root.join(".craftguild/demo/receipt.json").exists());
}

#[test]
fn native_and_variant_layouts_use_shared_resolver_and_sealed_sdk_environment() {
    let host =
        craftguild_standards::distribution::host(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let temp = fixture(&format!(
        r#"
[artifacts.build]
toolchain="1.95.0"
output="{{target_path}}{{profile}}/{{command}}{{exe}}"
compiler=["python3","compiler.py"]
[artifacts.build.environment]
FIXTURE_FLAG="{{profile}}"
[[artifacts.build.variants]]
id="distribution"
target="{host}"
output="dist/{{target}}/{{command}}{{exe}}"
output_root="repository"
[artifacts.build.variants.environment]
FIXTURE_FLAG="distribution-{{profile}}"
[[artifacts.build.hooks]]
id="sdk"
stage="prerequisite"
argv=["python3","sdk.py"]
inputs=["sdk.py"]
outputs=[".craftguild/derived/sdk.json"]
environment_output=".craftguild/derived/sdk.json"
unset_env=["CARGO_TARGET_DIR"]
"#
    ));
    let root = temp.path();
    write(
        root,
        "sdk.py",
        r#"import json,os,pathlib
assert 'CARGO_TARGET_DIR' not in os.environ
assert os.environ['CRAFTGUILD_TARGET_DIR']
pathlib.Path('.craftguild/derived/sdk.json').write_text(json.dumps({'FIXTURE_SDK':'verified-sdk'}))
"#,
    );
    write(
        root,
        "compiler.py",
        r#"import os,pathlib,subprocess,sys
assert os.environ['FIXTURE_SDK'] == 'verified-sdk'
assert sys.argv[1] == 'build'
assert '--locked' in sys.argv
assert sys.argv[sys.argv.index('--target')+1] == os.environ['CRAFTGUILD_TARGET']
expected = 'distribution-debug' if os.environ['CRAFTGUILD_VARIANT'] else 'debug'
assert os.environ['FIXTURE_FLAG'] == expected
pathlib.Path('compiler-invoked').write_text('one bounded driver')
sys.exit(subprocess.call(['cargo',*sys.argv[1:]]))
"#,
    );
    success(cli(root, &["build", "--profile", "debug"]));
    let native = receipt(root);
    assert!(Path::new(native["context"]["output"].as_str().unwrap()).ends_with("target/debug/demo"));
    assert_eq!(native["environment"]["FIXTURE_SDK"], "verified-sdk");
    assert!(native["prepared"]["inputs"]
        .get(".craftguild/derived/sdk.json")
        .is_some());
    success(cli(
        root,
        &["install", "--profile", "debug", "--destdir", "stage"],
    ));
    success(cli(
        root,
        &[
            "build",
            "--variant",
            "distribution",
            "--profile",
            "debug",
            "--target-dir",
            "compiler-output",
        ],
    ));
    let dist = receipt(root);
    assert!(Path::new(dist["context"]["output"].as_str().unwrap())
        .ends_with(format!("dist/{host}/demo")));
    assert!(Path::new(dist["context"]["target_dir"].as_str().unwrap()).ends_with("compiler-output"));
    assert_eq!(dist["environment"]["FIXTURE_FLAG"], "distribution-debug");
    success(cli(
        root,
        &[
            "install",
            "--variant",
            "distribution",
            "--target-dir",
            "compiler-output",
            "--profile",
            "debug",
            "--destdir",
            "stage",
        ],
    ));
    assert!(!cli(
        root,
        &["install", "--profile", "debug", "--destdir", "stage"]
    )
    .status
    .success());
    write(root, "sdk.py", "import pathlib; pathlib.Path('.craftguild/derived/sdk.json').write_text('{\"CARGO_TARGET_DIR\":\"unowned\"}')\n");
    assert!(!cli(root, &["build", "--profile", "debug"]).status.success());
    assert_eq!(receipt(root), dist);
}

#[test]
fn actual_library_formats_share_seal_and_publish_an_exact_output_inventory() {
    let temp = fixture(
        r#"
[artifacts.build.library]
name="fixture_library"
formats=["cdylib","staticlib","rlib"]
[artifacts.build.library.properties]
"fixture:abi"="Retained library target contract"
"#,
    );
    let root = temp.path();
    let manifest = std::fs::read_to_string(root.join("Cargo.toml"))
        .unwrap()
        .replace("edition='2021'", "edition='2021'\nlicense='MIT'");
    write(
        root,
        "Cargo.toml",
        format!(
            "{manifest}\n[lib]\nname='fixture_library'\ncrate-type=['cdylib','staticlib','rlib']\n"
        ),
    );
    write(
        root,
        "src/lib.rs",
        r#"const SBOM: &[u8] = include_bytes!(env!("FIXTURE_SBOM"));
#[no_mangle]
pub extern "C" fn fixture_sbom_bytes() -> *const u8 { SBOM.as_ptr() }
#[no_mangle]
pub extern "C" fn fixture_sbom_size() -> usize { SBOM.len() }
"#,
    );
    success(cli(root, &["build", "--profile", "debug"]));
    let receipt = receipt(root);
    assert_eq!(receipt["outputs"].as_object().unwrap().len(), 3);
    assert_eq!(receipt["context"]["outputs"].as_array().unwrap().len(), 3);
    let bytes = std::fs::read(root.join(receipt["prepared"]["sbom"].as_str().unwrap())).unwrap();
    let bom: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(bom["metadata"]["component"]["type"], "library");
    assert_eq!(
        bom["metadata"]["component"]["licenses"][0]["expression"],
        "MIT"
    );
    assert!(bom["metadata"]["component"]["properties"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["name"] == "fixture:abi"));
    for output in receipt["context"]["outputs"].as_array().unwrap() {
        let actual = std::fs::read(output.as_str().unwrap()).unwrap();
        assert!(actual.windows(bytes.len()).any(|part| part == bytes));
    }
    let parsed: craftguild_standards::execution_record::Receipt =
        serde_json::from_value(receipt.clone()).unwrap();
    parsed.verify(root).unwrap();
    std::fs::write(
        receipt["context"]["outputs"][1].as_str().unwrap(),
        "tampered library",
    )
    .unwrap();
    assert!(parsed.verify(root).is_err());
    assert!(!cli(
        root,
        &[
            "install",
            "--artifact",
            "demo",
            "--profile",
            "debug",
            "--destdir",
            "stage"
        ]
    )
    .status
    .success());
}

#[test]
fn operation_preparation_precedes_ordinary_gates_and_restores_derived_cache_on_failure() {
    let temp = fixture(
        r#"
[artifacts.build]
profile="debug"
[[operations]]
name="test"
artifact="demo"
prepare=["demo"]
[[operations.hooks]]
id="sdk"
stage="prerequisite"
argv=["python3","operation-sdk.py"]
inputs=["operation-sdk.py"]
outputs=[".craftguild/derived/operation-sdk.json"]
environment_output=".craftguild/derived/operation-sdk.json"
[[operations.hooks]]
id="ownership"
stage="evidence"
argv=["python3","ownership.py"]
inputs=["ownership.py","owner-source"]
output_dirs=[".craftguild/ownership"]
[[commands]]
id="ordinary"
purpose="Assert prepared context and fresh ownership before ordinary tests"
phase="test"
argv=["python3","gate.py"]
[[commands]]
id="host"
purpose="Explicit host acceptance never joins ordinary tests"
phase="acceptance"
argv=["python3","-c","open('acceptance-ran','w').write('bad')"]
"#,
    );
    let root = temp.path();
    write(root, "operation-sdk.py", "import json,pathlib; pathlib.Path('.craftguild/derived/operation-sdk.json').write_text(json.dumps({'FIXTURE_SDK':'operation-sdk'}))\n");
    write(root, "ownership.py", "import pathlib; p=pathlib.Path('.craftguild/ownership'); p.mkdir(); (p/'proof').write_text(pathlib.Path('owner-source').read_text())\n");
    write(root, "owner-source", "first");
    write(
        root,
        "gate.py",
        r#"import json,os,pathlib,sys
assert os.environ['CRAFTGUILD_OPERATION'] == 'test'
assert os.environ['FIXTURE_SDK'] == 'operation-sdk'
record = json.loads(pathlib.Path(os.environ['CRAFTGUILD_PREPARED']).read_text())
assert record['build']['profile'] == 'debug'
assert '.craftguild/derived/operation-sdk.json' in record['inputs']
assert pathlib.Path('.craftguild/ownership/proof').read_text() == pathlib.Path('owner-source').read_text()
sys.exit(7 if pathlib.Path('fail').exists() else 0)
"#,
    );
    success(cli(root, &["test"]));
    assert!(!root.join("acceptance-ran").exists());
    assert!(!root.join(".craftguild/demo/receipt.json").exists());
    write(root, "owner-source", "second");
    write(root, "fail", "");
    assert!(!cli(root, &["test"]).status.success());
    assert_eq!(
        std::fs::read_to_string(root.join(".craftguild/ownership/proof")).unwrap(),
        "first"
    );
    assert!(!root.join("acceptance-ran").exists());
}

#[test]
fn source_changes_update_local_seal_and_binary_without_changing_dependency_sbom() {
    let temp = fixture("");
    let root = temp.path();
    success(cli(root, &["build", "--profile", "debug"]));
    let first = receipt(root);
    let bom =
        std::fs::read_to_string(root.join(first["prepared"]["sbom"].as_str().unwrap())).unwrap();
    assert_eq!(first["prepared"]["schema_version"], 2);
    for name in ["craftguild:inputs", "craftguild:inventories", "src/main.rs"] {
        assert!(!bom.contains(name));
    }
    assert!(!bom.contains(first["prepared"]["inputs"]["src/main.rs"].as_str().unwrap()));
    let source = std::fs::read_to_string(root.join("src/main.rs")).unwrap();
    write(
        root,
        "src/main.rs",
        source.replace("println!(\"{}:{}\"", "println!(\"updated {}:{}\""),
    );
    success(cli(root, &["build", "--profile", "debug"]));
    let second = receipt(root);
    assert_ne!(
        first["prepared"]["inputs"]["src/main.rs"],
        second["prepared"]["inputs"]["src/main.rs"]
    );
    assert_ne!(first["outputs"], second["outputs"]);
    assert_eq!(
        first["prepared"]["sbom_sha256"],
        second["prepared"]["sbom_sha256"]
    );
    let binary = Path::new(second["context"]["output"].as_str().unwrap());
    assert_eq!(
        Command::new(binary).output().unwrap().stdout,
        b"updated debug:false\n"
    );
}

#[test]
fn later_hook_cannot_rebind_earlier_inputs_or_mutate_resolved_configuration() {
    for mutation in ["source", "configuration"] {
        let temp = fixture(
            r#"
[artifacts.build]
[[artifacts.build.hooks]]
id="inspect"
stage="prerequisite"
argv=["python3","-c","pass"]
inputs=["source.txt"]
[[artifacts.build.hooks]]
id="generate"
stage="prepare"
argv=["python3","mutate.py"]
inputs=["mutate.py"]
outputs=["generated.txt"]
"#,
        );
        let root = temp.path();
        write(root, "source.txt", "inspected input");
        write(root, "generated.txt", "previous output");
        write(
            root,
            "mutate.py",
            if mutation == "source" {
                "import pathlib; pathlib.Path('source.txt').write_text('changed later'); pathlib.Path('generated.txt').write_text('candidate')\n"
            } else {
                "import pathlib; p=pathlib.Path('craftguild.toml'); p.write_text(p.read_text()+'\\n# changed after resolution\\n'); pathlib.Path('generated.txt').write_text('candidate')\n"
            },
        );
        let result = cli(root, &["build", "--profile", "debug"]);
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("Stale execution evidence"));
        assert_eq!(
            std::fs::read_to_string(root.join("generated.txt")).unwrap(),
            "previous output"
        );
        assert!(!root.join(".craftguild/demo/receipt.json").exists());
        let run = std::fs::read_dir(root.join(".craftguild/demo/runs"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let record: Value =
            serde_json::from_slice(&std::fs::read(run.join("result.json")).unwrap()).unwrap();
        assert!(record["evidence"]["inspect"]["inputs"]
            .get("source.txt")
            .is_some());
        assert!(record["steps"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["id"] == "seal" && s["state"] == "failed"));
    }
}

#[test]
fn missing_implicit_toolchain_fails_without_acquiring_it() {
    let temp = fixture("");
    let root = temp.path().canonicalize().unwrap();
    write(
        &root,
        "rust-toolchain.toml",
        "[toolchain]\nchannel='1.84.99'\n",
    );
    let rustup = root.join("isolated-rustup");
    std::fs::create_dir(&rustup).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_craftguild-standards-bootstrap"))
        .args(["build", "--profile", "debug"])
        .current_dir(&root)
        .env_remove("RUSTUP_TOOLCHAIN")
        .env_remove("TOOLCHAIN")
        .env_remove("TARGET")
        .env("RUSTUP_HOME", &rustup)
        .env("RUSTUP_AUTO_INSTALL", "1")
        .env(
            "RUSTUP_DIST_SERVER",
            format!("file://{}", root.join("unavailable-dist").display()),
        )
        .output()
        .unwrap();
    assert!(!result.status.success());
    let error = String::from_utf8_lossy(&result.stderr);
    assert!(error.contains("not installed"), "{error}");
    assert!(!error.contains("syncing channel updates"), "{error}");
    assert!(!root.join(".craftguild/demo/receipt.json").exists());
    assert!(!rustup.join("toolchains").exists());
}

#[test]
fn release_configuration_gate_validates_in_memory_without_git_or_file_edits() {
    let temp = fixture("");
    let root = temp.path();
    let config = std::fs::read_to_string(root.join("craftguild.toml")).unwrap();
    write(
        root,
        "craftguild.toml",
        config
            + r#"
[[releases]]
id="demo"
initial_version="1.2.3"
tag_prefix="v"
notes="notes"
paths=["src","Cargo.toml","Cargo.lock"]
[[releases.cargo]]
manifest="Cargo.toml"
lockfile="Cargo.lock"
packages=["demo"]
"#,
    );
    let before: Vec<_> = [
        "Cargo.toml",
        "Cargo.lock",
        "craftguild.toml",
        "src/main.rs",
        "build.rs",
    ]
    .into_iter()
    .map(|name| (name, std::fs::read(root.join(name)).unwrap()))
    .collect();
    success(cli(root, &["check-release-config"]));
    for (name, bytes) in before {
        assert_eq!(std::fs::read(root.join(name)).unwrap(), bytes);
    }
    assert!(!root.join("notes").exists());
    assert!(!root.join(".craftguild").exists());
}

#[test]
fn target_settings_preserve_native_layout_and_select_cross_driver_without_make_detection() {
    let temp = fixture(
        r#"
[artifacts.build]
output="{target_path}{profile}/{file}"
[[artifacts.build.targets]]
target="x86_64-pc-windows-msvc"
cross_only=true
output="dist/{target}/{file}"
compiler=["cargo","xwin"]
[artifacts.build.targets.environment]
FIXTURE_PLATFORM="windows"
"#,
    );
    let root = temp.path();
    success(cli(root, &["prepare", "--profile", "debug"]));
    let runs = root.join(".craftguild/demo/runs");
    let first = std::fs::read_dir(&runs)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let native: Value =
        serde_json::from_slice(&std::fs::read(first.join("plan.json")).unwrap()).unwrap();
    assert!(Path::new(native["context"]["output"].as_str().unwrap()).ends_with("target/debug/demo"));
    assert!(native["context"]["host"].is_string());
    success(cli(
        root,
        &[
            "prepare",
            "--target",
            "x86_64-pc-windows-msvc",
            "--profile",
            "debug",
        ],
    ));
    let cross: Value = std::fs::read_dir(&runs)
        .unwrap()
        .map(|entry| {
            serde_json::from_slice::<Value>(
                &std::fs::read(entry.unwrap().path().join("plan.json")).unwrap(),
            )
            .unwrap()
        })
        .find(|record| record["context"]["target"] == "x86_64-pc-windows-msvc")
        .unwrap();
    if cross["context"]["host"] != "x86_64-pc-windows-msvc" {
        assert_eq!(
            cross["context"]["compiler"],
            serde_json::json!(["cargo", "xwin"])
        );
        assert!(Path::new(cross["context"]["output"].as_str().unwrap())
            .ends_with("target/dist/x86_64-pc-windows-msvc/demo.exe"));
    }
    assert!(!root.join(".craftguild/demo/receipt.json").exists());
}

#[test]
fn explicit_acceptance_gate_preserves_independent_artifact_scope() {
    let temp = fixture("");
    let root = temp.path();
    let mut config = std::fs::read_to_string(root.join("craftguild.toml")).unwrap();
    for id in ["cli", "font-runtime"] {
        config.push_str(&format!("\n[[commands]]\nid='{id}'\nphase='acceptance'\npurpose='Isolated public acceptance scope'\nargv=['python3','accept.py','{id}']\n"));
    }
    write(root, "craftguild.toml", config);
    write(
        root,
        "accept.py",
        "import pathlib,sys; pathlib.Path(sys.argv[1]).touch()\n",
    );
    success(cli(root, &["acceptance", "--gate", "cli"]));
    assert!(root.join("cli").is_file());
    assert!(!root.join("font-runtime").exists());
    assert!(!cli(root, &["acceptance", "--gate", "unknown"])
        .status
        .success());
    assert!(!cli(root, &["test", "--gate", "cli"]).status.success());
}

#[cfg(unix)]
#[test]
fn interrupted_execution_waits_for_children_then_restores_outputs_and_history() {
    use std::time::{Duration, Instant};
    let temp = fixture(
        r#"
[[operations]]
name="test"
artifact="demo"
[[operations.hooks]]
id="owned-cache"
stage="prepare"
argv=["python3","interrupt.py"]
inputs=["interrupt.py"]
outputs=[".craftguild/derived/cache.txt"]
"#,
    );
    let root = temp.path().canonicalize().unwrap();
    write(
        &root,
        ".craftguild/derived/cache.txt",
        "previous verified cache",
    );
    write(&root, "interrupt.py", "import pathlib,time\npathlib.Path('.craftguild/derived/cache.txt').write_text('candidate')\npathlib.Path('ready').touch()\ntime.sleep(1.5)\n");
    let mut engine = Command::new(env!("CARGO_BIN_EXE_craftguild-standards-bootstrap"))
        .arg("test")
        .current_dir(&root)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !root.join("ready").exists() && Instant::now() < deadline {
        assert!(engine.try_wait().unwrap().is_none());
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(root.join("ready").exists());
    engine.kill().unwrap();
    engine.wait().unwrap();
    let run = std::fs::read_dir(root.join(".craftguild/demo/runs"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let log = run.join("process.jsonl");
    let mut bytes = std::fs::read(&log).unwrap();
    let committed = bytes
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |i| i + 1);
    bytes.truncate(committed);
    bytes.extend_from_slice(b"{\"argv\":");
    std::fs::write(log, bytes).unwrap();
    let live = craftguild_standards::execution_transaction::Lock::acquire(&root);
    assert!(live.err().unwrap().to_string().contains("still live"));
    assert_eq!(
        std::fs::read_to_string(root.join(".craftguild/derived/cache.txt")).unwrap(),
        "candidate"
    );
    std::thread::sleep(Duration::from_millis(1600));
    let recovered = craftguild_standards::execution_transaction::Lock::acquire(&root).unwrap();
    drop(recovered);
    assert_eq!(
        std::fs::read_to_string(root.join(".craftguild/derived/cache.txt")).unwrap(),
        "previous verified cache"
    );
    let result: Value =
        serde_json::from_slice(&std::fs::read(run.join("result.json")).unwrap()).unwrap();
    assert_eq!(result["state"], "interrupted");
    assert_eq!(result["steps"][0]["state"], "interrupted");
    assert!(result["steps"][0]["started_at"].is_number());
    assert!(result["steps"][0]["finished_at"].is_number());
    assert!(result["steps"][1]["state"] == "pending");
    assert!(!root.join(".craftguild/demo/receipt.json").exists());
}

#[test]
fn acceptance_can_install_its_completed_build_without_tools_or_parent_lock_deadlock() {
    let temp = fixture(
        r#"
[[operations]]
name="acceptance"
artifact="demo"
[[commands]]
id="install-completed-output"
phase="acceptance"
purpose="Install completed output inside a locked acceptance operation"
argv=["python3","accept.py"]
"#,
    );
    let root = temp.path();
    let engine = env!("CARGO_BIN_EXE_craftguild-standards-bootstrap");
    write(root, "accept.py", format!("import os\nos.environ['PATH']=''\nos.execv({engine:?},[{engine:?},'install','--profile','debug','--destdir','stage'])\n"));
    success(cli(root, &["build", "--profile", "debug"]));
    success(cli(root, &["acceptance"]));
    assert_eq!(
        std::fs::read(root.join("stage/usr/local/bin/demo")).unwrap(),
        std::fs::read(receipt(root)["context"]["output"].as_str().unwrap()).unwrap()
    );
}

#[cfg(unix)]
#[test]
fn exited_hook_leaders_cannot_leave_descendants_to_overwrite_published_or_restored_outputs() {
    for code in [7, 0] {
        let temp = fixture(
            r#"
[[artifacts.build.hooks]]
id="generate"
stage="prepare"
argv=["python3","generate.py"]
inputs=["generate.py"]
outputs=["generated.txt"]
"#,
        );
        let root = temp.path();
        write(
            root,
            "generate.py",
            "import pathlib; pathlib.Path('generated.txt').write_text('verified output')\n",
        );
        success(cli(root, &["build", "--profile", "debug"]));
        let before = receipt(root);
        write(
            root,
            "generate.py",
            format!(
                r#"import pathlib, subprocess, sys
subprocess.Popen([sys.executable, '-c', "import pathlib,time; time.sleep(1.5); pathlib.Path('generated.txt').write_text('late unverified child output')"])
pathlib.Path('generated.txt').write_text('candidate')
raise SystemExit({code})
"#
            ),
        );
        let result = cli(root, &["build", "--profile", "debug"]);
        assert_eq!(
            result.status.success(),
            code == 0,
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        if code != 0 {
            assert_eq!(receipt(root), before);
        }
        std::thread::sleep(std::time::Duration::from_millis(1700));
        assert_eq!(
            std::fs::read_to_string(root.join("generated.txt")).unwrap(),
            if code == 0 {
                "candidate"
            } else {
                "verified output"
            }
        );
    }
}

#[test]
fn common_make_defaults_match_direct_build_and_install_with_configured_selectors() {
    let temp = fixture("[artifacts.build]\nprofile='debug'\ntarget_dir='configured-output'\n");
    let root = temp.path();
    write(
        root,
        "common.mk",
        include_str!("../../../templates/make/common.mk"),
    );
    write(root, "Makefile", "include common.mk\n");
    success(cli(root, &["build"]));
    let direct = receipt(root);
    let make = |args: &[&str]| {
        Command::new("make")
            .args(args)
            .arg(format!(
                "CRAFTGUILD_LAUNCH=\"{}\"",
                env!("CARGO_BIN_EXE_craftguild-standards-bootstrap")
            ))
            .env_remove("PROFILE")
            .env_remove("CARGO_TARGET_DIR")
            .env_remove("TARGET")
            .current_dir(root)
            .output()
            .unwrap()
    };
    success(make(&["build"]));
    assert_eq!(receipt(root)["context"], direct["context"]);
    assert_eq!(
        receipt(root)["prepared"]["sbom_sha256"],
        direct["prepared"]["sbom_sha256"]
    );
    success(make(&[
        "install",
        "PREFIX=/opt/configured",
        "DESTDIR=stage",
    ]));
    assert_eq!(
        std::fs::read(root.join("stage/opt/configured/bin/demo")).unwrap(),
        std::fs::read(direct["context"]["output"].as_str().unwrap()).unwrap()
    );
    success(make(&[
        "build",
        "PROFILE=release",
        "CARGO_TARGET_DIR=explicit-output",
    ]));
    let explicit = receipt(root);
    assert_eq!(explicit["context"]["profile"], "release");
    assert!(
        Path::new(explicit["context"]["target_dir"].as_str().unwrap()).ends_with("explicit-output")
    );
    success(make(&[
        "build",
        "PROFILE=release",
        "BUILD_ARGS=--profile debug --target-dir cli-output",
    ]));
    let selected = receipt(root);
    assert_eq!(selected["context"]["profile"], "debug");
    assert!(Path::new(selected["context"]["target_dir"].as_str().unwrap()).ends_with("cli-output"));
}
