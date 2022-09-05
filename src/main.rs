mod address;
mod command;
mod config;
mod rdp;
mod select;
mod ssh;
mod tunnel;

use crate::command::launch_command;
use crate::config::Config;
use crate::rdp::launch_rdp;
use crate::ssh::launch_ssh;
use crate::tunnel::launch_tunnel;
use anyhow::Context;
use clap::{Args, Parser};
use env_logger::{Env, Target};

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Parser)]
enum Subcommand {
    /// Launch an RDP connection
    Rdp(Rdp),
    /// Launch an SSH session
    Ssh(Ssh),
    /// Open SSH tunnel(s)
    Tunnel(Tunnel),
    /// Run a remote command using SSH
    Command(Command),
    /// Open config file
    Config,
}

#[derive(Args)]
pub struct Rdp {
    /// Name of the RDP profile to launch
    name: String,
    /// Connect via IPv4 address
    #[clap(long)]
    ipv4: bool,
    /// Connect via IPv6 address
    #[clap(long, conflicts_with = "ipv4")]
    ipv6: bool,
    /// Connect using the remote desktop gateway
    #[clap(long, short = 'g')]
    enable_gateway: bool,
    /// Connect directly (without a gateway)
    #[clap(long, short, conflicts_with = "enable-gateway")]
    disable_gateway: bool,
    /// Print the config to stdout instead of connecting
    #[clap(long)]
    stdout: bool,
    /// Open the profile in edit mode instead of connecting
    #[clap(long, conflicts_with = "stdout")]
    edit: bool,
}

#[derive(Args)]
pub struct SshCommon {
    /// Connect via IPv4 address
    #[clap(long)]
    ipv4: bool,
    /// Connect via IPv6 address
    #[clap(long, conflicts_with = "ipv4")]
    ipv6: bool,
    /// Connect using the jump hosts
    #[clap(long, short = 'j')]
    use_jump_hosts: bool,
    /// Connect directly (without jump hosts)
    #[clap(long, short, conflicts_with = "use-jump-hosts")]
    disable_jump_hosts: bool,
    /// Print the command to stdout instead of connecting
    #[clap(long)]
    stdout: bool,
}

#[derive(Args)]
pub struct Ssh {
    /// Name of the SSH profile to launch
    name: String,
    #[clap(flatten)]
    common: SshCommon,
}

#[derive(Args)]
pub struct Tunnel {
    /// Name of the tunnel profile to launch
    name: String,
    #[clap(flatten)]
    common: SshCommon,
}

#[derive(Args)]
pub struct Command {
    /// Name of the command to run
    name: String,
    #[clap(flatten)]
    common: SshCommon,
}

fn main() {
    let args = Cli::parse();
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .target(Target::Stderr)
        .init();
    if let Err(e) = run(args) {
        log::error!("{:#}", e);
        std::process::exit(1);
    }
}

fn run(args: Cli) -> anyhow::Result<()> {
    let config = Config::load()?;
    match args.subcommand {
        Subcommand::Rdp(rdp) => launch_rdp(&config, &rdp),
        Subcommand::Ssh(ssh) => launch_ssh(&config, &ssh),
        Subcommand::Tunnel(tunnel) => launch_tunnel(&config, &tunnel),
        Subcommand::Command(cmd) => launch_command(&config, &cmd),
        Subcommand::Config => {
            let cfg_path = config::config_path()?;
            open::that(&cfg_path).context("Unable to open config file")
        }
    }?;
    Ok(())
}
