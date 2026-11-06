//! Filesystem permission failures must not strand a new entry behind the old pin.
use craftguild_standards::init_transaction::{self, Edit};
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

struct Fixture(tempfile::TempDir);
impl Fixture {
    fn new() -> Self {
        let fixture = Self(tempfile::tempdir().unwrap());
        fs::create_dir_all(fixture.root().join("standards/make")).unwrap();
        for (name, bytes, mode) in [
            ("standards/launcher.py", "old launcher", 0o444),
            ("standards/make/common.mk", "old Make entry", 0o444),
            ("standards/engine.json", "old pin", 0o440),
        ] {
            fs::write(fixture.root().join(name), bytes).unwrap();
            fixture.mode(name, mode);
        }
        fs::create_dir_all(fixture.root().join(".craftguild/engine/valid-old")).unwrap();
        fs::write(
            fixture.root().join(".craftguild/engine/valid-old/engine"),
            "valid cached engine",
        )
        .unwrap();
        fixture
    }
    fn root(&self) -> &Path {
        self.0.path()
    }
    fn mode(&self, name: &str, mode: u32) {
        fs::set_permissions(self.root().join(name), fs::Permissions::from_mode(mode)).unwrap();
    }
    fn edits(&self) -> Vec<Edit> {
        [
            ("standards/launcher.py", "new launcher"),
            ("standards/make/common.mk", "new Make entry"),
            ("standards/engine.json", "new pin"),
        ]
        .into_iter()
        .map(|(name, after)| {
            Edit::new(self.root(), name, after.as_bytes().to_vec(), false).unwrap()
        })
        .collect()
    }
    fn restored(&self) {
        for (name, bytes, mode) in [
            ("standards/launcher.py", "old launcher", 0o444),
            ("standards/make/common.mk", "old Make entry", 0o444),
            ("standards/engine.json", "old pin", 0o440),
        ] {
            assert_eq!(
                fs::read(self.root().join(name)).unwrap(),
                bytes.as_bytes(),
                "{name}"
            );
            assert_eq!(
                fs::metadata(self.root().join(name))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                mode,
                "{name}"
            );
        }
        assert_eq!(
            fs::read(self.root().join(".craftguild/engine/valid-old/engine")).unwrap(),
            b"valid cached engine"
        );
        self.no_staging();
    }
    fn no_staging(&self) {
        assert!(!self.root().join(".craftguild/init.lock").exists());
        assert!(fs::read_dir(self.root().join(".craftguild"))
            .unwrap()
            .all(|item| item.unwrap().file_name() == "engine"));
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.mode("standards/make", 0o755);
    }
}

#[test]
fn natural_make_directory_failure_restores_read_only_launcher_and_old_pin() {
    let fixture = Fixture::new();
    let edits = fixture.edits();
    fixture.mode("standards/make", 0o555);
    let error = init_transaction::publish(
        fixture.root(),
        &edits,
        |_| Ok(()),
        |index| {
            if index == 1 {
                assert_eq!(
                    fs::read(fixture.root().join("standards/launcher.py"))?,
                    b"new launcher"
                );
            }
            Ok(())
        },
    )
    .expect_err("This permission regression requires a nonprivileged filesystem user");
    assert!(
        error
            .to_string()
            .contains("initialization rolled back owned writes"),
        "{error}"
    );
    fixture.restored();
}

#[test]
fn late_failure_restores_all_read_only_files_and_permissions_including_pin() {
    let fixture = Fixture::new();
    let edits = fixture.edits();
    let error = init_transaction::publish_with_finish(
        fixture.root(),
        &edits,
        |_| Ok(()),
        |_| Ok(()),
        || {
            assert_eq!(
                fs::read(fixture.root().join("standards/engine.json"))?,
                b"new pin"
            );
            Err("success output failed".into())
        },
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("success output failed"),
        "{error}"
    );
    fixture.restored();
}

#[test]
fn read_only_concurrent_replacement_is_preserved_during_rollback() {
    let fixture = Fixture::new();
    let edits = fixture.edits();
    let error = init_transaction::publish(
        fixture.root(),
        &edits,
        |_| Ok(()),
        |index| {
            if index == 1 {
                let other = fixture.root().join("other-writer");
                fs::write(&other, "concurrent owner")?;
                fs::set_permissions(&other, fs::Permissions::from_mode(0o444))?;
                fs::rename(other, fixture.root().join("standards/launcher.py"))?;
                return Err("concurrent replacement".into());
            }
            Ok(())
        },
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("preserved concurrent files: standards/launcher.py"),
        "{error}"
    );
    assert_eq!(
        fs::read(fixture.root().join("standards/launcher.py")).unwrap(),
        b"concurrent owner"
    );
    assert_eq!(
        fs::metadata(fixture.root().join("standards/launcher.py"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o444
    );
    assert_eq!(
        fs::read(fixture.root().join("standards/engine.json")).unwrap(),
        b"old pin"
    );
    fixture.no_staging();
}

#[test]
fn failed_restore_keeps_recovery_files_and_continues_other_owned_restores() {
    let fixture = Fixture::new();
    let edits = fixture.edits();
    let error = init_transaction::publish(
        fixture.root(),
        &edits,
        |_| Ok(()),
        |index| {
            if index == 2 {
                fixture.mode("standards/make", 0o555);
                return Err("late interruption with a blocked rollback directory".into());
            }
            Ok(())
        },
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("rollback incomplete"), "{error}");
    assert!(error.contains("standards/make/common.mk"), "{error}");
    assert!(error.contains("previous files retained at"), "{error}");
    assert_eq!(
        fs::read(fixture.root().join("standards/launcher.py")).unwrap(),
        b"old launcher"
    );
    assert_eq!(
        fs::read(fixture.root().join("standards/engine.json")).unwrap(),
        b"old pin"
    );
    assert_eq!(
        fs::read(fixture.root().join("standards/make/common.mk")).unwrap(),
        b"new Make entry"
    );
    assert!(!fixture.root().join(".craftguild/init.lock").exists());
    let recoveries: Vec<_> = fs::read_dir(fixture.root().join(".craftguild"))
        .unwrap()
        .map(|item| item.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("init-recovery-")
        })
        .collect();
    assert_eq!(recoveries.len(), 1);
    assert_eq!(
        fs::read(recoveries[0].join("standards/make/common.mk")).unwrap(),
        b"old Make entry"
    );
    assert_eq!(
        fs::metadata(recoveries[0].join("standards/make/common.mk"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o444
    );
    assert!(error.contains(recoveries[0].to_str().unwrap()));
}
