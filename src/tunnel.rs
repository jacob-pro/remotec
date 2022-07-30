use crate::config::SshForwardArgument;
use crate::select::select_profile_by_name;
use crate::ssh::{invoke_ssh, ssh_args};
use crate::{Config, Tunnel};
use anyhow::bail;

impl SshForwardArgument {
    fn ssh_arg(&self) -> String {
        format!(
            "{}:{}:{}",
            self.local_port, self.remote_host, self.remote_port
        )
    }
}

pub fn launch_tunnel(config: &Config, cli: &Tunnel) -> anyhow::Result<()> {
    let profile = select_profile_by_name("Tunnel", &config.tunnels, &cli.name)?;
    if profile.forwards.is_empty() {
        bail!("Profile doesn't contain any forwards");
    }
    let mut ssh_args = ssh_args(config, &cli.common, &profile.ssh_profile)?;

    for f in &profile.forwards {
        log::info!(
            "Forwards {}:{} -> local port {}",
            f.remote_host,
            f.remote_port,
            f.local_port
        );
        ssh_args.push("-L".to_string());
        ssh_args.push(f.ssh_arg());
    }
    ssh_args.append(
        &mut ["sleep", "2147483647"]
            .into_iter()
            .map(str::to_string)
            .collect(),
    );

    invoke_ssh(ssh_args, cli.common.stdout)?;
    Ok(())
}
