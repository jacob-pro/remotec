use crate::config::Address;
use anyhow::Context;

pub fn choose_address(address: &Address, cli_ipv4: bool, cli_ipv6: bool) -> anyhow::Result<&str> {
    if cli_ipv4 {
        return address
            .ipv4
            .as_deref()
            .context("An IPv4 address is not configured for this profile");
    }
    if cli_ipv6 {
        return address
            .ipv6
            .as_deref()
            .context("An IPv6 address is not configured for this profile");
    }
    [&address.hostname, &address.ipv6, &address.ipv4]
        .into_iter()
        .flat_map(|x| x.iter())
        .next()
        .map(|x| x.as_str())
        .context("No addresses configured for this profile")
}
