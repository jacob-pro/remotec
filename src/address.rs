use crate::config::Address;
use anyhow::Context;

impl Address {
    pub fn choose_address(&self, force_ipv4: bool, force_ipv6: bool) -> anyhow::Result<&str> {
        if force_ipv4 {
            return self
                .ipv4
                .as_deref()
                .context("An IPv4 address is not configured for this profile");
        }
        if force_ipv6 {
            return self
                .ipv6
                .as_deref()
                .context("An IPv6 address is not configured for this profile");
        }
        [&self.hostname, &self.ipv6, &self.ipv4]
            .into_iter()
            .flat_map(|x| x.iter())
            .next()
            .map(|x| x.as_str())
            .context("No addresses configured for this profile")
    }
}
