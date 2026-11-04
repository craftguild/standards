//! Test-only OpenPGP signing in a disposable keyring; never use personal keys.

use std::path::Path;
use std::process::{Command, Output};

fn successful(output: Output) -> String {
    assert!(
        output.status.success(),
        "Signing fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}

pub fn commit(root: &Path, message: &str) {
    let home = tempfile::tempdir().unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(home.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let gpg = |args: &[&str]| {
        Command::new("gpg")
            .env("GNUPGHOME", home.path())
            .args(args)
            .output()
            .unwrap()
    };
    successful(gpg(&[
        "--batch",
        "--pinentry-mode",
        "loopback",
        "--passphrase",
        "",
        "--quick-generate-key",
        "Conformance <conformance@example.invalid>",
        "ed25519",
        "sign",
        "0",
    ]));
    let keys = successful(gpg(&["--batch", "--with-colons", "--list-secret-keys"]));
    let key = keys
        .lines()
        .find_map(|line| {
            let fields: Vec<_> = line.split(':').collect();
            (fields[0] == "fpr").then(|| fields[9])
        })
        .expect("Generated signing key fingerprint");
    let git = |args: &[&str]| {
        Command::new("git")
            .env("GNUPGHOME", home.path())
            .env(
                "GIT_CONFIG_GLOBAL",
                if cfg!(unix) { "/dev/null" } else { "NUL" },
            )
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .args([
                "-c",
                "gpg.format=openpgp",
                "-c",
                "gpg.program=gpg",
                "-c",
                &format!("user.signingkey={key}"),
                "-c",
                "user.name=Conformance",
                "-c",
                "user.email=conformance@example.invalid",
            ])
            .args(args)
            .current_dir(root)
            .output()
            .unwrap()
    };
    successful(git(&["commit", "-S", "-qm", message]));
    let revision = successful(git(&["rev-parse", "HEAD"]));
    successful(git(&["verify-commit", &revision]));
    let _ = Command::new("gpgconf")
        .args([
            "--homedir",
            home.path().to_str().unwrap(),
            "--kill",
            "gpg-agent",
        ])
        .output();
}
