use crate::select::select_profile_by_name;
use crate::ssh::{invoke_ssh, ssh_args};
use crate::{Command, Config};
use anyhow::bail;

pub fn launch_command(config: &Config, cli: &Command) -> anyhow::Result<()> {
    let profile = select_profile_by_name("Command", &config.commands, &cli.name, true)?;
    if profile.command.is_empty() {
        bail!("Profile doesn't contain any command");
    }
    let mut ssh_args = ssh_args(config, &cli.common, &profile.ssh_profile, false)?;

    ssh_args.append(&mut profile.command.clone());

    invoke_ssh(ssh_args, cli.common.stdout)?;
    Ok(())
}
