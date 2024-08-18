use crate::ProfileType;
use anyhow::Context;
use remotec::config::{CommandProfile, RdpProfile, SshProfile, TunnelProfile};
use std::fmt::{Display, Formatter};

impl Display for ProfileType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileType::Rdp => f.write_str("RDP"),
            ProfileType::Ssh => f.write_str("SSH"),
            ProfileType::Tunnel => f.write_str("Tunnel"),
            ProfileType::Command => f.write_str("Command"),
        }
    }
}

pub trait NamedProfile {
    fn profile_type(&self) -> ProfileType;
    fn name(&self) -> &str;
    fn description(&self) -> Option<&str>;
}

impl NamedProfile for RdpProfile {
    fn profile_type(&self) -> ProfileType {
        ProfileType::Rdp
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

impl NamedProfile for SshProfile {
    fn profile_type(&self) -> ProfileType {
        ProfileType::Ssh
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

impl NamedProfile for TunnelProfile {
    fn profile_type(&self) -> ProfileType {
        ProfileType::Tunnel
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

impl NamedProfile for CommandProfile {
    fn profile_type(&self) -> ProfileType {
        ProfileType::Command
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

pub fn select_profile_by_name<'a, T: NamedProfile>(
    profile_type: &'static str,
    list: &'a [T],
    name: &str,
    print_description: bool,
) -> anyhow::Result<&'a T> {
    let matched = list.iter().filter(|t| t.name() == name).collect::<Vec<_>>();
    if matched.len() > 1 {
        log::warn!("Found multiple {profile_type} profiles found for `{name}` - using first");
    }
    let profile = matched
        .into_iter()
        .next()
        .context(format!("No {profile_type} profile found for `{name}`"))?;
    if print_description {
        if let Some(description) = profile.description() {
            log::info!("Description: {}", description);
        }
    }
    Ok(profile)
}
