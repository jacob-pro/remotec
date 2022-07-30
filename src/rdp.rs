// See: https://docs.microsoft.com/en-us/windows-server/remote/remote-desktop-services/clients/rdp-files

use crate::address::choose_address;
use crate::config::{GatewayPolicy, RdpBackend, RdpProfile};
use crate::select::select_profile_by_name;
use crate::{Config, Rdp};
use anyhow::{bail, Context};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn cache_directory() -> anyhow::Result<PathBuf> {
    let dir = dirs::cache_dir()
        .context("Unable to get cache directory")?
        .join("remotec");
    if !dir.exists() {
        fs::create_dir_all(&dir).context("Unable to create cache directory")?;
    }
    Ok(dir)
}

pub fn launch_rdp(config: &Config, cli: &Rdp) -> anyhow::Result<()> {
    let profile = select_profile_by_name(&config.rdp, &cli.name)?;

    let mut rdp_config = Vec::new();
    rdp_config.push(format!(
        "full address:s:{}",
        choose_address(&profile.address, cli.ipv4, cli.ipv6)?
    ));
    rdp_config.push(format!("username:s:{}", username(profile, config)));
    if let Some(value) = &profile.domain {
        rdp_config.push(format!("domain:s:{value}"))
    }
    if let Some(value) = &profile.gateway {
        rdp_config.push(format!("gatewayhostname:s:{value}"))
    }
    rdp_config.push(format!(
        "gatewayusagemethod:i:{}",
        gateway_policy(profile, cli)? as u8
    ));
    rdp_config.push("gatewayprofileusagemethod:i:1".to_string());
    rdp_config.push(format!(
        "promptcredentialonce:i:{}",
        profile.separate_credentials.then_some(0).unwrap_or(1)
    ));
    rdp_config.push("".to_string());

    let rdp_config = rdp_config.join("\n");

    if cli.stdout {
        print!("{}", rdp_config);
    } else {
        let backend = match config.rdp_defaults.backend {
            None => RdpBackend::default_for_platform()?,
            Some(s) => s,
        };

        let dest = cache_directory()?
            .join(format!("rdp-{}", profile.name))
            .with_extension("rdp");
        fs::write(&dest, &rdp_config).context("Unable to write RDP config")?;

        backend.open(&dest, cli.edit)?
    }

    Ok(())
}

fn gateway_policy(profile: &RdpProfile, cli: &Rdp) -> anyhow::Result<GatewayPolicy> {
    if cli.enable_gateway {
        if profile.gateway.is_none() {
            bail!("Profile doesn't contain a gateway")
        }
        return Ok(GatewayPolicy::Enable);
    }
    if cli.disable_gateway {
        return Ok(GatewayPolicy::Disable);
    }
    Ok(profile.gateway_policy)
}

fn username(profile: &RdpProfile, config: &Config) -> String {
    if let Some(username) = &profile.username {
        return username.to_string();
    }
    if let Some(username) = &config.rdp_defaults.username {
        return username.to_string();
    }
    whoami::username()
}

impl RdpBackend {
    fn default_for_platform() -> anyhow::Result<Self> {
        cfg_if::cfg_if! {
            if #[cfg(windows)] {
                Ok(RdpBackend::Mstsc)
            } else {
                bail!("No RDP backend supported for this platform")
            }
        }
    }

    fn open(&self, rdp_file: &Path, edit: bool) -> anyhow::Result<()> {
        match &self {
            RdpBackend::Mstsc => {
                let mut cmd = Command::new("mstsc");
                if edit {
                    cmd.arg("/edit");
                }
                cmd.arg(&rdp_file);
                cmd.spawn()
                    .context("Unable to launch Microsoft Remote Desktop")?;
            }
        }
        Ok(())
    }
}
