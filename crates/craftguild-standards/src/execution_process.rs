//! Cancellable subprocesses with isolated process groups and bounded execution.

use craftguild_core::Result;
#[cfg(unix)]
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
#[cfg(unix)]
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::process::{Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, OnceLock,
};
#[cfg(unix)]
use std::time::Instant;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static CANCELLED: AtomicBool = AtomicBool::new(false);
static SIGNALS: OnceLock<std::result::Result<(), String>> = OnceLock::new();

pub fn initialize_signals() -> Result<()> {
    SIGNALS
        .get_or_init(|| {
            ctrlc::set_handler(|| CANCELLED.store(true, Ordering::SeqCst))
                .map_err(|e| e.to_string())
        })
        .clone()
        .map_err(Into::into)
}

pub fn now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[derive(Clone, Debug)]
pub struct Runner {
    pub cleanup_failed: Arc<AtomicBool>,
    pub env: BTreeMap<String, String>,
    pub unset_env: BTreeSet<String>,
    pub toolchain: Option<String>,
    pub compiler_version: Option<String>,
    pub log: Option<PathBuf>,
    pub cancel_file: Option<PathBuf>,
    pub timeout: Duration,
}

impl Default for Runner {
    fn default() -> Self {
        Self {
            cleanup_failed: Arc::default(),
            env: BTreeMap::new(),
            unset_env: BTreeSet::new(),
            toolchain: None,
            compiler_version: None,
            log: None,
            cancel_file: None,
            timeout: Duration::from_secs(3600),
        }
    }
}

impl Runner {
    pub fn rollback_safe(&self) -> Result<()> {
        if self.cleanup_failed.load(Ordering::SeqCst) {
            return Err(
                "Subprocess termination could not be verified; rollback is deferred".into(),
            );
        }
        Ok(())
    }
    pub fn cancelled(&self) -> bool {
        CANCELLED.load(Ordering::SeqCst) || self.cancel_file.as_ref().is_some_and(|p| p.exists())
    }

    pub fn check(&self) -> Result<()> {
        if self.cancelled() {
            Err("Execution cancelled".into())
        } else {
            Ok(())
        }
    }

    pub fn capture(&self, root: &Path, argv: &[String]) -> Result<Vec<u8>> {
        #[cfg(not(unix))]
        {
            let _ = (root, argv);
            Err("Owned subprocess execution requires Unix process groups on this engine".into())
        }
        #[cfg(unix)]
        {
            self.check()?;
            let mut effective = argv.to_vec();
            if matches!(argv.first().map(String::as_str), Some("cargo" | "rustc")) {
                if let Some(toolchain) = &self.toolchain {
                    effective.splice(0..0, ["rustup".into(), "run".into(), toolchain.clone()]);
                }
            }
            let (program, args) = effective.split_first().ok_or("Empty required command")?;
            let mut stdout = tempfile::tempfile()?;
            let mut stderr = tempfile::tempfile()?;
            let mut command = Command::new(program);
            command
                .args(args)
                .current_dir(root)
                .envs(&self.env)
                .stdin(Stdio::null())
                .stdout(stdout.try_clone()?)
                .stderr(stderr.try_clone()?);
            for name in &self.unset_env {
                command.env_remove(name);
            }
            // A rustup proxy must never acquire a missing toolchain during execution.
            command.env("RUSTUP_AUTO_INSTALL", "0");
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                command.process_group(0);
            }
            self.event(json!({"argv":effective,"state":"running","at":now()}))?;
            let active = self.log.as_ref().map(|p| p.with_extension("active"));
            if let Some(path) = &active {
                // A crash between spawn and PID recording must defer recovery.
                std::fs::write(path, b"pending")?;
            }
            let mut child = match command.spawn() {
                Ok(child) => child,
                Err(error) => {
                    if let Some(path) = &active {
                        std::fs::remove_file(path)?;
                    }
                    return Err(format!("Cannot execute required tool {program}: {error}").into());
                }
            };
            let result = (|| -> Result<Vec<u8>> {
                if let Some(path) = &active {
                    std::fs::write(path, child.id().to_string())?;
                }
                self.event(
                    json!({"argv":effective,"state":"spawned","pid":child.id(),"at":now()}),
                )?;
                let started = Instant::now();
                let status = loop {
                    if self.cancelled() || started.elapsed() >= self.timeout {
                        return Err(if self.cancelled() {
                            "Execution cancelled"
                        } else {
                            "Command timed out"
                        }
                        .into());
                    }
                    match child.try_wait() {
                        Ok(Some(status)) => break status,
                        Ok(None) => std::thread::sleep(Duration::from_millis(10)),
                        Err(error) => return Err(error.into()),
                    }
                };
                stdout.seek(SeekFrom::Start(0))?;
                stderr.seek(SeekFrom::Start(0))?;
                let mut bytes = Vec::new();
                stdout.read_to_end(&mut bytes)?;
                let mut errors = Vec::new();
                stderr.read_to_end(&mut errors)?;
                self.event(
                    json!({"argv":effective,"stdout":String::from_utf8_lossy(&bytes),
                "stderr":String::from_utf8_lossy(&errors),"status":status.code(),"at":now()}),
                )?;
                if !status.success() {
                    std::io::stderr().write_all(&bytes)?;
                    std::io::stderr().write_all(&errors)?;
                    return Err(format!(
                        "Required command failed ({status}): {}",
                        effective.join(" ")
                    )
                    .into());
                }
                std::io::stderr().write_all(&errors)?;
                Ok(bytes)
            })();
            // Even an exited leader may leave writers behind. Stop and wait for its
            // owned group on every path, including success and logging/read failures.
            let result = match terminate(&mut child) {
                Ok(()) => {
                    if let Some(path) = &active {
                        std::fs::remove_file(path)?;
                    }
                    result
                }
                Err(error) => {
                    self.cleanup_failed.store(true, Ordering::SeqCst);
                    Err(format!("Subprocess cleanup failed: {error}").into())
                }
            };
            if result.is_err() {
                stdout.seek(SeekFrom::Start(0))?;
                stderr.seek(SeekFrom::Start(0))?;
                let mut out = Vec::new();
                let mut err = Vec::new();
                stdout.read_to_end(&mut out)?;
                stderr.read_to_end(&mut err)?;
                self.event(
                    json!({"argv":effective,"stdout":String::from_utf8_lossy(&out),
                "stderr":String::from_utf8_lossy(&err),"at":now()}),
                )?;
            }
            self.event(
                json!({"argv":effective,"state":if result.is_ok(){"succeeded"}else{"failed"},
            "error":result.as_ref().err().map(ToString::to_string),"at":now()}),
            )?;
            result
        }
    }

    pub fn run(&self, root: &Path, argv: &[String]) -> Result<()> {
        let output = self.capture(root, argv)?;
        std::io::stderr().write_all(&output)?;
        Ok(())
    }

    #[cfg(unix)]
    fn event(&self, value: serde_json::Value) -> Result<()> {
        if let Some(path) = &self.log {
            let mut bytes = serde_json::to_vec(&value)?;
            bytes.push(b'\n');
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;
            file.write_all(&bytes)?;
            file.sync_data()?;
        }
        Ok(())
    }
}

#[cfg(unix)]
fn terminate(child: &mut std::process::Child) -> Result<()> {
    use nix::{
        errno::Errno,
        sys::signal::{killpg, Signal},
        unistd::Pid,
    };
    match killpg(Pid::from_raw(child.id().try_into()?), Signal::SIGKILL) {
        Ok(()) | Err(Errno::ESRCH) => {}
        Err(error) => return Err(error.into()),
    }
    child.wait()?;
    let deadline = Instant::now() + Duration::from_secs(5);
    while group_live(child.id())? {
        if Instant::now() >= deadline {
            return Err("Owned subprocess group is still live after termination".into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}

#[cfg(unix)]
pub(crate) fn group_live(pid: u32) -> Result<bool> {
    use nix::{errno::Errno, sys::signal::killpg, unistd::Pid};
    match killpg(Pid::from_raw(pid.try_into()?), None) {
        Ok(()) => {
            #[cfg(target_os = "macos")]
            {
                // macOS may retain orphan zombies after SIGKILL. They are
                // terminal and cannot write; query only this owned group.
                let output = Command::new("/bin/ps")
                    .args(["-g", &pid.to_string(), "-o", "stat="])
                    .output()?;
                let states = String::from_utf8(output.stdout)?;
                if output.status.success() {
                    return Ok(states.split_whitespace().any(|s| !s.starts_with('Z')));
                }
                if output.status.code() == Some(1) && states.is_empty() && output.stderr.is_empty()
                {
                    return Ok(false);
                }
                Err("Cannot verify owned subprocess group states".into())
            }
            #[cfg(not(target_os = "macos"))]
            Ok(true)
        }
        Err(Errno::ESRCH) => Ok(false),
        Err(error) => Err(error.into()),
    }
}
