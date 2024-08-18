use crate::select::NamedProfile;
use crate::ProfileType;
use remotec::config::Config;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct Profile<'t> {
    #[tabled(rename = "Type")]
    profile_type: ProfileType,
    #[tabled(rename = "Name")]
    name: &'t str,
    #[tabled(rename = "Description")]
    description: &'t str,
}

pub fn list_profiles(config: &Config, profile_type: Option<ProfileType>) -> anyhow::Result<()> {
    let mut profiles: Vec<&dyn NamedProfile> = vec![];
    config.rdp.iter().for_each(|r| profiles.push(r));
    config.ssh.iter().for_each(|r| profiles.push(r));
    config.tunnels.iter().for_each(|r| profiles.push(r));
    config.commands.iter().for_each(|r| profiles.push(r));
    let output = profiles
        .iter()
        .map(|p| Profile {
            profile_type: p.profile_type(),
            name: p.name(),
            description: p.description().unwrap_or(""),
        })
        .filter(|p| profile_type.map(|t| t == p.profile_type).unwrap_or(true));
    println!("{}", Table::new(output));
    Ok(())
}
