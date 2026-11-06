//! Shared numeric version policy for both initialization entry points.
use crate::repository_pin::{Identity, MachineIdentity};
use craftguild_core::Result;
use std::cmp::Ordering;

#[derive(Debug, PartialEq, Eq)]
pub enum Change {
    Initialize,
    Revalidate,
    Upgrade,
    Downgrade,
    Replace,
}

impl Change {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Initialize => "initialize",
            Self::Revalidate => "revalidate",
            Self::Upgrade => "upgrade",
            Self::Downgrade => "downgrade",
            Self::Replace => "same-version replacement",
        }
    }
}

pub fn select(tool: &MachineIdentity, previous: Option<&Identity>, force: bool) -> Result<Change> {
    let selected = crate::release::versions::stable(&tool.engine.version)?;
    if !tool.clean
        || tool.engine.revision.len() != 40
        || !tool
            .engine
            .revision
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("Init requires a clean published engine identity; unpublished source cannot create or update a public pin".into());
    }
    let Some(previous) = previous else {
        return Ok(Change::Initialize);
    };
    let pinned = crate::release::versions::stable(&previous.version)?;
    let change = match selected.cmp(&pinned) {
        Ordering::Greater => Change::Upgrade,
        Ordering::Less => Change::Downgrade,
        Ordering::Equal if tool.engine.revision == previous.revision => Change::Revalidate,
        Ordering::Equal => Change::Replace,
    };
    if !force && matches!(change, Change::Downgrade | Change::Replace) {
        return Err(format!(
            "Refusing {}: CLI {} @ {}; pin {} @ {}. Use init --force to authorize this version transition",
            change.label(), tool.engine.version, tool.engine.revision, previous.version, previous.revision
        ).into());
    }
    Ok(change)
}
