//! Bound best-effort metadata subprocesses so a broken tool cannot hang a build.

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub fn output(command: &mut Command) -> Option<Vec<u8>> {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let stdout = child.stdout.take()?;
    let (send, receive) = std::sync::mpsc::sync_channel(1);
    let reader = std::thread::Builder::new()
        .name("build-facts".into())
        .spawn(move || {
            let mut bytes = Vec::new();
            let result = stdout
                .take(1024 * 1024 + 1)
                .read_to_end(&mut bytes)
                .ok()
                .filter(|_| bytes.len() <= 1024 * 1024)
                .map(|_| bytes);
            let _ = send.send(result);
        });
    let bytes = reader.ok().and_then(|_| {
        receive
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .ok()
            .flatten()
    });
    if let Some(bytes) = bytes {
        loop {
            match child.try_wait() {
                Ok(Some(status)) => return status.success().then_some(bytes),
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(5))
                }
                _ => break,
            }
        }
    }
    let _ = child.kill();
    let _ = child.wait();
    None
}
