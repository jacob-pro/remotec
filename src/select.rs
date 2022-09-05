use crate::config::{CommandProfile, RdpProfile, SshProfile, TunnelProfile};
use anyhow::Context;

pub trait NamedProfile {
    fn name(&self) -> &str;
}

impl NamedProfile for RdpProfile {
    fn name(&self) -> &str {
        &self.name
    }
}

impl NamedProfile for SshProfile {
    fn name(&self) -> &str {
        &self.name
    }
}

impl NamedProfile for TunnelProfile {
    fn name(&self) -> &str {
        &self.name
    }
}

impl NamedProfile for CommandProfile {
    fn name(&self) -> &str {
        &self.name
    }
}

pub fn select_profile_by_name<'a, T: NamedProfile>(
    profile_type: &'static str,
    list: &'a [T],
    name: &str,
) -> anyhow::Result<&'a T> {
    let matched = list.iter().filter(|t| t.name() == name).collect::<Vec<_>>();
    if matched.len() > 1 {
        log::warn!("Found multiple {profile_type} profiles found for `{name}` - using first");
    }
    matched
        .into_iter()
        .next()
        .context(format!("No {profile_type} profile found for `{name}`"))
}
