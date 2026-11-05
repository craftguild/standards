//! External command execution reports missing tools and failed checks explicitly.

use craftguild_core::Result;
use std::path::Path;

pub fn capture(root: &Path, argv: &[String]) -> Result<Vec<u8>> {
    crate::execution_process::Runner::default().capture(root, argv)
}

pub fn run(root: &Path, argv: &[String]) -> Result<()> {
    crate::execution_process::Runner::default().run(root, argv)
}
