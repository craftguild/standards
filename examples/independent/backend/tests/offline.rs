use std::process::Command;

#[test]
fn offline_process_contract() {
    let directory = std::env::temp_dir().join(format!(
        "craftguild-sample-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    std::fs::write(directory.join("craftguild.toml"), "invalid configuration [").unwrap();
    let run = |args: &[&str]| {
        let result = Command::new(env!("CARGO_BIN_EXE_backend-demo"))
            .args(args)
            .current_dir(&directory)
            .env("PATH", "")
            .output()
            .unwrap();
        assert!(result.status.success());
        assert!(result.stderr.is_empty());
        result.stdout
    };
    let help = run(&[]);
    assert_eq!(help, run(&["-h"]));
    assert_eq!(help, run(&["--help"]));
    assert_eq!(run(&["-V"]), run(&["--version"]));
    let sbom = String::from_utf8(run(&["sbom"])).unwrap();
    assert!(sbom.starts_with('{') && sbom.contains("CycloneDX"));
    assert!(!sbom.contains("craftguild/backend-demo"));
    assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);
    std::fs::remove_dir_all(directory).unwrap();
}
