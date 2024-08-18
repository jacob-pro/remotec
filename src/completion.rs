// inspired by: https://www.joshmcguigan.com/blog/shell-completions-pure-rust/
// this is my first ever attempt at a completion script - it's probably not very good!

mod config;

use crate::config::Config;
use shell_completion::{BashCompletionInput, CompletionInput, CompletionSet};
use std::collections::HashSet;

struct Context<'t> {
    config: &'t Config,
    input: &'t BashCompletionInput,
    current_idx: usize,
}

impl Context<'_> {
    /// Gets the next argument to process
    /// boolean indicates if it is the current argument under the cursor
    fn next_arg(&mut self) -> Option<(&str, bool)> {
        let binding = self.input.args();
        let arg = binding.get(self.current_idx);
        let is_current = self.current_idx == self.input.arg_index();
        self.current_idx += 1;
        arg.map(|x| (*x, is_current))
    }

    /// Filters out options which have already been specified in the command line
    fn filter_existing_options(&self, options: &'static [CliOption]) -> Vec<&'static CliOption> {
        let mut args = self
            .input
            .args()
            .iter()
            .skip(1)
            .map(|x| *x)
            .collect::<HashSet<_>>();
        // Remove the current word under the cursor
        args.remove(self.input.args()[self.input.arg_index()]);
        // Replace with the right-hand side of the word under the cursor
        args.insert(
            self.input.args()[self.input.arg_index()]
                .split_at(self.input.char_index())
                .1,
        );
        options
            .iter()
            .filter(|o| args.intersection(&o.representations()).count() == 0)
            .collect()
    }
}

fn main() {
    if let Ok(input) = BashCompletionInput::from_env() {
        let config = Config::load().unwrap_or_default();
        let completions = get_completions(&config, &input);
        completions.suggest();
    }
}

fn get_completions(config: &Config, input: &BashCompletionInput) -> Vec<String> {
    let mut ctx = Context {
        config,
        input,
        current_idx: 0,
    };

    let subcommands = vec!["rdp", "ssh", "tunnel", "command", "config"]
        .into_iter()
        .map(ToString::to_string)
        .collect();

    // Skip the program name
    ctx.next_arg();

    // Get the subcommand
    let (subcommand, is_current_arg) = ctx.next_arg().unwrap();
    log::info!("Subcommand: {}", subcommand);
    let completions = if is_current_arg {
        subcommands
    } else {
        match subcommand {
            "rdp" => complete_rdp(ctx),
            "ssh" => complete_ssh(ctx),
            "tunnel" => complete_tunnel(ctx),
            "command" => complete_command(ctx),
            // Unsupported subcommand
            _ => vec![],
        }
    };

    // Filter matching subcommands
    input.complete_subcommand(completions.iter().map(String::as_str))
}

fn complete_connection(
    mut ctx: Context,
    connection_names: Vec<String>,
    options: &'static [CliOption],
) -> Vec<String> {
    let (connection_name, is_current_arg) = ctx.next_arg().unwrap();
    log::info!("Connection name: {:?}", connection_name);

    if is_current_arg {
        connection_names
    } else {
        let filtered = ctx.filter_existing_options(options);
        let options = filtered.iter().map(|c| c.suggestion()).collect::<Vec<_>>();
        options
    }
}

fn complete_rdp(ctx: Context) -> Vec<String> {
    let possibilities = ctx
        .config
        .rdp
        .iter()
        .map(|r| r.name.to_owned())
        .collect::<Vec<_>>();
    complete_connection(ctx, possibilities, RDP_OPTIONS)
}

fn complete_ssh(ctx: Context) -> Vec<String> {
    let possibilities = ctx
        .config
        .ssh
        .iter()
        .map(|r| r.name.to_owned())
        .collect::<Vec<_>>();
    complete_connection(ctx, possibilities, SSH_OPTIONS)
}

fn complete_tunnel(ctx: Context) -> Vec<String> {
    let possibilities = ctx
        .config
        .tunnels
        .iter()
        .map(|r| r.name.to_owned())
        .collect::<Vec<_>>();
    complete_connection(ctx, possibilities, SSH_OPTIONS)
}

fn complete_command(ctx: Context) -> Vec<String> {
    let possibilities = ctx
        .config
        .commands
        .iter()
        .map(|r| r.name.to_owned())
        .collect::<Vec<_>>();
    complete_connection(ctx, possibilities, SSH_OPTIONS)
}

struct CliOption {
    short: Option<&'static str>,
    long: Option<&'static str>,
}

impl CliOption {
    pub const fn new(short: Option<&'static str>, long: Option<&'static str>) -> Self {
        Self { short, long }
    }

    /// All possible representation of the option
    fn representations(&self) -> HashSet<&'static str> {
        self.long
            .iter()
            .chain(self.short.iter())
            .map(|x| *x)
            .collect()
    }

    /// Suggest long if possible otherwise short
    fn suggestion(&self) -> String {
        self.long
            .iter()
            .chain(self.short.iter())
            .map(ToString::to_string)
            .next()
            .expect("Option should have either long or short")
    }
}

const RDP_OPTIONS: &[CliOption] = &[
    CliOption::new(Some("-d"), Some("--disable-gateway")),
    CliOption::new(None, Some("edit")),
    CliOption::new(Some("-g"), Some("--enable-gateway")),
    CliOption::new(None, Some("--help")),
    CliOption::new(None, Some("--ipv4")),
    CliOption::new(None, Some("--ipv6")),
    CliOption::new(None, Some("--stdout")),
];

const SSH_OPTIONS: &[CliOption] = &[
    CliOption::new(Some("-d"), Some("--disable-jump-hosts")),
    CliOption::new(None, Some("--help")),
    CliOption::new(None, Some("--ipv4")),
    CliOption::new(None, Some("--ipv6")),
    CliOption::new(Some("-j"), Some("--use-jump-hosts")),
    CliOption::new(None, Some("--stdout")),
];

#[cfg(test)]
mod tests {
    use super::*;
    use env_logger::{Env, Target};

    #[derive(Debug)]
    struct TestCase<'t> {
        input: &'t str,
        output: &'t [&'t str],
    }

    impl<'t> TestCase<'_> {
        fn new(input: &'t str, output: &'t [&'t str]) -> TestCase<'t> {
            TestCase { input, output }
        }
    }

    #[test]
    fn test() {
        env_logger::Builder::from_env(Env::default().default_filter_or("info"))
            .target(Target::Stderr)
            .init();

        let test_profile = include_bytes!("../test_resources/test_profile.json");
        let config = Config::from_bytes(test_profile).unwrap();

        let cases = &[
            // Subcommands
            TestCase::new("remotec |", &["rdp", "ssh", "tunnel", "command", "config"]),
            TestCase::new(
                "remotec |rdp",
                &["rdp", "ssh", "tunnel", "command", "config"],
            ),
            TestCase::new("remotec co|", &["command", "config"]),
            TestCase::new("remotec co| --after", &["command", "config"]),
            TestCase::new("remotec unknown|", &[]),
            // Rdp
            TestCase::new("remotec rdp |", &["rdp_example"]),
            TestCase::new("remotec rdp rdp_ex|", &["rdp_example"]),
            TestCase::new("remotec rdp abc|", &[]),
            TestCase::new("remotec rdp rdp_example |", &["--disable-gateway", "edit", "--enable-gateway", "--help", "--ipv4", "--ipv6", "--stdout"]),
            TestCase::new("remotec rdp rdp_example |--ipv4", &["--disable-gateway", "edit", "--enable-gateway", "--help", "--ipv6", "--stdout"]),
            TestCase::new("remotec rdp rdp_example |--disable-gateway edit --enable-gateway --help --ipv4 --ipv6 --stdout", &[]),
            TestCase::new("remotec rdp rdp_example |-d edit -g --help --ipv4 --ipv6 --stdout", &[]),
            TestCase::new("remotec rdp rdp_example --ipv4 --disable|-gateway edit -g --help --ipv6 --stdout", &["--disable-gateway"]),

            // SSH
            TestCase::new("remotec ssh |", &["ssh_example"]),
            TestCase::new("remotec ssh ssh_ex|", &["ssh_example"]),
            TestCase::new("remotec ssh ssh_example --ipv4|", &["--ipv4"]),
            TestCase::new("remotec ssh ssh_example --ipv4 |", &["--disable-jump-hosts", "--help", "--ipv6", "--use-jump-hosts", "--stdout"]),
            // Tunnel
            TestCase::new("remotec tunnel |", &["tunnel_example"]),
            TestCase::new("remotec tunnel tunnel_ex|", &["tunnel_example"]),
            // Command
            TestCase::new("remotec command |", &["command_example"]),
            TestCase::new("remotec command comma|", &["command_example"]),
        ];

        for (idx, case) in cases.iter().enumerate() {
            log::info!("Beginning test case {}: \"{}\"", idx, case.input);
            let comp_point = case.input.find('|').unwrap();
            let line = case.input.replace('|', "");
            let input = BashCompletionInput::new(&line, comp_point).unwrap();
            log::info!(
                "BashCompletionInput{{args: {:?}, arg_index: {}, char_index: {}}}",
                input.args(),
                input.arg_index(),
                input.char_index()
            );

            let completions = get_completions(&config, &input);
            assert_eq!(case.output, completions, "expected = left, actual = right");
            log::info!("Finished test case with suggestions: {:?}\n", completions);
        }
    }
}
