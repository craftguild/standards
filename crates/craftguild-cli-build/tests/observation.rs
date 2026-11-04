use craftguild_cli_build::observe::{capture, repository};
use craftguild_core::Dirty;
use std::path::Path;
use std::process::Command;

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn initialized() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    git(temp.path(), &["init", "-b", "main"]);
    git(temp.path(), &["config", "user.name", "Conformance"]);
    git(temp.path(), &["config", "commit.gpgsign", "false"]);
    git(
        temp.path(),
        &["config", "user.email", "conformance@example.invalid"],
    );
    std::fs::write(temp.path().join("source"), "original").unwrap();
    std::fs::write(temp.path().join(".gitignore"), "ignored\n").unwrap();
    git(temp.path(), &["add", "."]);
    git(temp.path(), &["commit", "-m", "initial"]);
    temp
}

#[test]
fn clean_each_dirty_kind_and_ignored_files() {
    let temp = initialized();
    let root = temp.path();
    let original = repository(root);
    assert_eq!(original.0.len(), 40);
    assert_eq!(original.1, Dirty::Clean);
    std::fs::write(root.join("ignored"), "ignored").unwrap();
    assert_eq!(repository(root), original);
    std::fs::write(root.join("new"), "untracked").unwrap();
    assert_eq!(repository(root).1, Dirty::Dirty);
    std::fs::remove_file(root.join("new")).unwrap();
    std::fs::write(root.join("source"), "unstaged").unwrap();
    assert_eq!(repository(root).1, Dirty::Dirty);
    git(root, &["add", "source"]);
    assert_eq!(repository(root).1, Dirty::Dirty);
    git(root, &["commit", "-m", "next"]);
    let next = repository(root);
    assert_ne!(next.0, original.0);
    assert_eq!(next.1, Dirty::Clean);
}

#[test]
fn linked_worktree_and_archive_inside_unrelated_parent() {
    let temp = initialized();
    let linked = tempfile::tempdir().unwrap();
    let worktree = linked.path().join("linked");
    git(
        temp.path(),
        &[
            "worktree",
            "add",
            "-b",
            "linked",
            worktree.to_str().unwrap(),
        ],
    );
    assert_eq!(repository(&worktree), repository(temp.path()));
    std::fs::write(worktree.join("new"), "dirty").unwrap();
    assert_eq!(repository(&worktree).1, Dirty::Dirty);
    let archive = temp.path().join("archive");
    std::fs::create_dir(&archive).unwrap();
    assert_eq!(repository(&archive), ("unknown".into(), Dirty::Unknown));
    assert_eq!(
        repository(linked.path()),
        ("unknown".into(), Dirty::Unknown)
    );
}

#[test]
fn captures_caller_command_compiler_target_and_profile() {
    let temp = initialized();
    let info = capture(
        temp.path(),
        "public-command",
        "3.2.1",
        "custom-target",
        "release",
    );
    assert_eq!(info.command, "public-command");
    assert_eq!(info.version, "3.2.1");
    assert_eq!(info.target, "custom-target");
    assert_eq!(info.profile, "release");
    assert!(info.compiler.starts_with("rustc "));
    info.validate().unwrap();
}

#[test]
fn absent_git_and_explicit_compiler_are_observed_in_an_isolated_process() {
    if let Ok(root) = std::env::var("CRAFTGUILD_OBSERVATION_CHILD") {
        let info = capture(
            Path::new(&root),
            "demo",
            "3.2.1",
            "selected-target",
            "release",
        );
        assert_eq!(info.commit, "unknown");
        assert_eq!(info.dirty, Dirty::Unknown);
        assert_eq!(
            info.compiler,
            std::env::var("CRAFTGUILD_EXPECTED_COMPILER").unwrap()
        );
        return;
    }
    let temp = initialized();
    let run = |compiler: &Path, expected: &str| {
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "absent_git_and_explicit_compiler_are_observed_in_an_isolated_process",
            ])
            .env("CRAFTGUILD_OBSERVATION_CHILD", temp.path())
            .env("CRAFTGUILD_EXPECTED_COMPILER", expected)
            .env("PATH", "")
            .env("RUSTC", compiler)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    };
    run(&temp.path().join("missing-compiler"), "unknown");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let wrapper = temp.path().join("compiler-wrapper");
        std::fs::write(&wrapper, "#!/bin/sh\necho 'rustc explicit-wrapper 3.2.1'\n").unwrap();
        std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755)).unwrap();
        run(&wrapper, "rustc explicit-wrapper 3.2.1");
    }
}
