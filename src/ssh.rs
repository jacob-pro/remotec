use crate::config::{SshJumpHost, SshProfile};
use crate::select::select_profile_by_name;
use crate::{Config, Ssh, SshCommon};
use anyhow::{bail, Context};
use std::process::Command;

pub fn ssh_args(
    config: &Config,
    cli: &SshCommon,
    profile: &str,
    print_description: bool,
) -> anyhow::Result<Vec<String>> {
    let profile = select_profile_by_name("SSH", &config.ssh, profile, print_description)?;
    let jumps = jump_hosts(profile, cli)?;
    let address = profile.address.choose_address(cli.ipv4, cli.ipv6)?;
    let username = username(profile, config);

    let mut args = Vec::new();
    if jumps.is_empty() {
        args.push(format!("{username}@{address}"));
        if let Some(port) = profile.address.port {
            args.push("-p".to_string());
            args.push(port.to_string());
        }
    } else {
        args.push("-J".to_string());
        for j in jumps {
            let username = j.username.as_ref().unwrap_or(&username);
            let port = j.port.map(|p| format!(":{p}")).unwrap_or_else(String::new);
            args.push(format!("{username}@{}{port}", j.hostname));
        }
        let port = profile
            .address
            .port
            .map(|p| format!(":{p}"))
            .unwrap_or_else(String::new);
        args.push(format!("{username}@{}{port}", address));
    }
    Ok(args)
}

pub fn invoke_ssh(args: Vec<String>, stdout: bool) -> anyhow::Result<()> {
    let command = format!("ssh {}", args.join(" "));
    if stdout {
        println!("{}", command);
    } else {
        log::info!("Invoking: `{}`", command);
        Command::new("ssh")
            .args(args)
            .status()
            .context("Error invoking ssh")?;
    }
    Ok(())
}

pub fn launch_ssh(config: &Config, cli: &Ssh) -> anyhow::Result<()> {
    let args = ssh_args(config, &cli.common, &cli.name, true)?;
    invoke_ssh(args, cli.common.stdout)?;
    Ok(())
}

fn jump_hosts<'a>(
    profile: &'a SshProfile,
    cli: &SshCommon,
) -> anyhow::Result<Vec<&'a SshJumpHost>> {
    if cli.use_jump_hosts {
        if profile.jump_hosts.is_empty() {
            bail!("Profile doesn't contain any jumphosts");
        }
        return Ok(profile.jump_hosts.iter().collect());
    }
    if cli.disable_jump_hosts || profile.disable_jump_hosts {
        return Ok(Vec::new());
    }
    return Ok(profile.jump_hosts.iter().collect());
}

fn username(profile: &SshProfile, config: &Config) -> String {
    if let Some(username) = &profile.username {
        return username.to_string();
    }
    if let Some(username) = &config.rdp_defaults.username {
        return username.to_string();
    }
    whoami::username()
}
