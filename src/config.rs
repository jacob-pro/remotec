use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Serialize, Default)]
struct ConfigFile {
    #[serde(default)]
    include: Vec<PathBuf>,
    #[serde(flatten)]
    this: SatelliteConfig,
    #[serde(default)]
    rdp_defaults: RdpDefaults,
    #[serde(default)]
    ssh_defaults: SshDefaults,
    #[serde(default)]
    commands: Vec<CommandProfile>,
}

#[derive(Deserialize, Serialize, Default)]
struct SatelliteConfig {
    #[serde(default)]
    rdp: Vec<RdpProfile>,
    #[serde(default)]
    ssh: Vec<SshProfile>,
    #[serde(default)]
    tunnels: Vec<TunnelProfile>,
    #[serde(default)]
    commands: Vec<CommandProfile>,
}

#[derive(Deserialize, Serialize, Default)]
pub struct RdpDefaults {
    pub username: Option<String>,
    pub backend: Option<RdpBackend>,
}

#[derive(Deserialize, Serialize, Default)]
pub struct SshDefaults {
    pub username: Option<String>,
}

/// At present we only support launching the Microsoft Windows Remote Desktop client (mstsc.exe)
/// But in future we could support Linux clients etc.
#[derive(Deserialize, Serialize, Copy, Clone)]
pub enum RdpBackend {
    #[cfg(windows)]
    Mstsc,
}

#[derive(Deserialize, Serialize)]
pub struct Address {
    pub hostname: Option<String>,
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
    pub port: Option<u16>,
}

#[derive(Deserialize, Serialize)]
pub struct RdpProfile {
    pub name: String,
    #[serde(flatten)]
    pub address: Address,
    pub username: Option<String>,
    pub domain: Option<String>,
    pub gateway: Option<String>,
    #[serde(default)]
    pub gateway_policy: GatewayPolicy,
    #[serde(default)]
    pub separate_credentials: bool,
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct SshProfile {
    pub name: String,
    #[serde(flatten)]
    pub address: Address,
    pub username: Option<String>,
    #[serde(default)]
    pub disable_jump_hosts: bool,
    #[serde(default)]
    pub jump_hosts: Vec<SshJumpHost>,
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct TunnelProfile {
    pub name: String,
    pub ssh_profile: String,
    pub forwards: Vec<SshForwardArgument>,
    pub open: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct SshForwardArgument {
    pub local_port: u16,
    pub remote_port: u16,
    pub remote_host: String,
}

#[derive(Deserialize, Serialize)]
pub struct SshJumpHost {
    pub username: Option<String>,
    pub hostname: String,
    pub port: Option<u16>,
}

#[derive(Deserialize, Serialize)]
pub struct CommandProfile {
    pub name: String,
    pub ssh_profile: String,
    pub command: Vec<String>,
    pub description: Option<String>,
}

#[derive(Default)]
pub struct Config {
    pub rdp: Vec<RdpProfile>,
    pub ssh: Vec<SshProfile>,
    pub tunnels: Vec<TunnelProfile>,
    pub commands: Vec<CommandProfile>,
    pub rdp_defaults: RdpDefaults,
    pub ssh_defaults: SshDefaults,
}

pub fn config_path() -> anyhow::Result<PathBuf> {
    Ok(dirs::config_dir()
        .context("Unable to get config directory")?
        .join("remotec")
        .join("config.json"))
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = config_path()?;
        if !config_path.exists() {
            log::warn!(
                "Config file {} doesn't exist, creating defaults...",
                config_path.display()
            );
            fs::create_dir_all(config_path.parent().unwrap())
                .context("Failed to create remotec config directory")?;
            let default = ConfigFile::default();
            fs::write(
                &config_path,
                serde_json::to_string_pretty(&default).unwrap(),
            )
            .context("Unable to write config defaults")?;
            open::that(config_path).context("Unable to open config file")?;
            bail!("Try again after saving your config changes")
        }
        let cfg_file = fs::read_to_string(&config_path).context("Unable to read config file")?;
        let cfg_file: ConfigFile =
            serde_json::from_str(&cfg_file).context("Unable to deserialize config file")?;
        let mut config = Config {
            rdp: cfg_file.this.rdp,
            ssh: cfg_file.this.ssh,
            tunnels: cfg_file.this.tunnels,
            commands: cfg_file.this.commands,
            rdp_defaults: cfg_file.rdp_defaults,
            ssh_defaults: cfg_file.ssh_defaults,
        };
        for s in cfg_file.include {
            if let Some(mut s) = load_satellite_config(&s) {
                config.rdp.append(&mut s.rdp);
                config.ssh.append(&mut s.ssh);
                config.tunnels.append(&mut s.tunnels);
                config.commands.append(&mut s.commands);
            }
        }
        Ok(config)
    }
}

fn load_satellite_config(path: &Path) -> Option<SatelliteConfig> {
    if !path.exists() {
        log::warn!("Config include {} doesn't exist", path.display());
        return None;
    }
    let config = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(err) => {
            log::warn!("Unable to deserialize config include: {err}");
            return None;
        }
    };
    let config: SatelliteConfig = match serde_json::from_str(&config) {
        Ok(c) => c,
        Err(err) => {
            log::warn!("Unable to deserialize config include: {err}");
            return None;
        }
    };
    Some(config)
}

#[derive(Deserialize, Serialize, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GatewayPolicy {
    Disable = 0,
    Enable = 1,
    Fallback = 2,
}

impl Default for GatewayPolicy {
    fn default() -> Self {
        Self::Fallback
    }
}
