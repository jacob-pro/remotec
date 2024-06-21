// inspired by: https://www.joshmcguigan.com/blog/shell-completions-pure-rust/
// this is my first ever attempt at a completion script - it's probably not very good!

mod config;

use crate::config::Config;
use num_integer::Integer;
use shell_completion::{BashCompletionInput, CompletionInput};
use std::collections::{HashSet, VecDeque};

struct Context<'t> {
    config: &'t Config,
    up_to_cursor_str: &'t str,
    pending_args_up_to_cursor: VecDeque<String>,
    all_args: Vec<String>,
    current_idx: usize,
}

impl Context<'_> {
    /// If there are more arguments to parse, or the user is about to start a new argument
    fn new_arg(&self) -> bool {
        self.up_to_cursor_str.ends_with(' ') || !self.pending_args_up_to_cursor.is_empty()
    }

    /// Gets the next argument to parse
    fn next_arg(&mut self) -> Option<String> {
        let arg = self.pending_args_up_to_cursor.pop_front();
        if arg.is_some() {
            self.current_idx += 1;
        }
        arg
    }

    /// Filters out options which have already been specified
    fn filter_existing_options(&self, options: &'static [CliOption]) -> Vec<&'static CliOption> {
        let unparsed_args = self
            .all_args
            .iter()
            .skip(self.current_idx)
            .map(ToString::to_string)
            .collect::<HashSet<_>>();
        options
            .iter()
            .filter(|o| {
                let repr = o.strings();
                unparsed_args.intersection(&repr).count() == 0
            })
            .collect()
    }
}

fn balance_and_split(s: &str) -> Option<Vec<String>> {
    let mut s = s.to_string();
    // Split can fail when input contains unmatched quote
    // If the user is part way through a quote lets assume it to be one word
    if s.matches('\'').count().is_odd() {
        s.push('\'');
    }
    if s.matches('\"').count().is_odd() {
        s.push('\"');
    }
    shell_words::split(&s).ok()
}

fn main() {
    let input = BashCompletionInput::from_args().expect("Missing expected environment variables");
    let config = Config::load().unwrap_or_default();
    let completions = get_completions(&config, &input.line, input.cursor_position);
    input.complete_subcommand(completions.iter().map(|x| x.as_str()).collect::<Vec<_>>());
}

fn get_completions(config: &Config, line: &str, cursor_position: usize) -> Vec<String> {
    let trim = &line[0..cursor_position];

    let args_up_to_cursor = match balance_and_split(&trim) {
        None => return Vec::new(),
        Some(s) => s,
    };
    let all_args = match balance_and_split(line) {
        None => return Vec::new(),
        Some(s) => s,
    };
    let mut ctx = Context {
        config,
        up_to_cursor_str: trim,
        pending_args_up_to_cursor: args_up_to_cursor.into_iter().skip(1).collect(),
        all_args,
        current_idx: 1,
    };

    let subcommands = vec!["rdp", "ssh", "tunnel", "command", "config"]
        .into_iter()
        .map(ToString::to_string)
        .collect();
    match ctx.next_arg() {
        None => subcommands,
        Some(arg) => {
            if ctx.new_arg() {
                match arg.as_str() {
                    "rdp" => complete_rdp(ctx),
                    "ssh" => complete_ssh(ctx),
                    "tunnel" => complete_tunnel(ctx),
                    "command" => complete_command(ctx),
                    _ => vec![],
                }
            } else {
                subcommands
            }
        }
    }
}

fn complete_rdp(mut ctx: Context) -> Vec<String> {
    let next = ctx.next_arg();
    let possibilities = ctx
        .config
        .rdp
        .iter()
        .map(|r| r.name.to_owned())
        .collect::<Vec<_>>();
    match next {
        None => possibilities,
        Some(_arg) => {
            if ctx.new_arg() {
                let filtered = ctx.filter_existing_options(RDP_OPTIONS);
                let options = filtered
                    .iter()
                    .flat_map(|c| c.suggestion())
                    .collect::<Vec<_>>();
                options
            } else {
                possibilities
            }
        }
    }
}

fn complete_ssh(mut ctx: Context) -> Vec<String> {
    let next = ctx.next_arg();
    let possibilities = ctx
        .config
        .ssh
        .iter()
        .map(|r| r.name.to_owned())
        .collect::<Vec<_>>();
    match next {
        None => possibilities,
        Some(_arg) => {
            if ctx.new_arg() {
                let filtered = ctx.filter_existing_options(SSH_OPTIONS);
                let options = filtered
                    .iter()
                    .flat_map(|c| c.suggestion())
                    .collect::<Vec<_>>();
                options
            } else {
                possibilities
            }
        }
    }
}

fn complete_tunnel(mut ctx: Context) -> Vec<String> {
    let next = ctx.next_arg();
    let possibilities = ctx
        .config
        .tunnels
        .iter()
        .map(|r| r.name.to_owned())
        .collect::<Vec<_>>();
    match next {
        None => possibilities,
        Some(_arg) => {
            if ctx.new_arg() {
                let filtered = ctx.filter_existing_options(SSH_OPTIONS);
                let options = filtered
                    .iter()
                    .flat_map(|c| c.suggestion())
                    .collect::<Vec<_>>();
                options
            } else {
                possibilities
            }
        }
    }
}

fn complete_command(mut ctx: Context) -> Vec<String> {
    let next = ctx.next_arg();
    let possibilities = ctx
        .config
        .commands
        .iter()
        .map(|r| r.name.to_owned())
        .collect::<Vec<_>>();
    match next {
        None => possibilities,
        Some(_arg) => {
            if ctx.new_arg() {
                let filtered = ctx.filter_existing_options(SSH_OPTIONS);
                let options = filtered
                    .iter()
                    .flat_map(|c| c.suggestion())
                    .collect::<Vec<_>>();
                options
            } else {
                possibilities
            }
        }
    }
}

struct CliOption {
    short: Option<&'static str>,
    long: Option<&'static str>,
}

impl CliOption {
    pub const fn new(short: Option<&'static str>, long: Option<&'static str>) -> Self {
        Self { short, long }
    }

    fn strings(&self) -> HashSet<String> {
        self.long
            .iter()
            .chain(self.short.iter())
            .map(ToString::to_string)
            .collect()
    }

    fn suggestion(&self) -> Option<String> {
        self.long
            .iter()
            .chain(self.short.iter())
            .map(ToString::to_string)
            .next()
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

    fn owned(input: Vec<&'static str>) -> Vec<String> {
        input.into_iter().map(|x| x.to_owned()).collect()
    }

    fn get_completions_from_end(config: &Config, line: &str) -> Vec<String> {
        get_completions(config, line, line.len() - 1)
    }

    #[test]
    fn test() {
        let config = Config::default();
        assert_eq!(
            get_completions_from_end(&config, "remotec"),
            owned(vec!["rdp", "ssh", "tunnel", "command", "config"])
        );
        assert_eq!(
            get_completions_from_end(&config, "remotec "),
            owned(vec!["rdp", "ssh", "tunnel", "command", "config"])
        );
        assert_eq!(
            get_completions_from_end(&config, "remotec r"),
            owned(vec!["rdp", "ssh", "tunnel", "command", "config"])
        );
        assert_eq!(
            get_completions_from_end(&config, "remotec unknown"),
            owned(vec!["rdp", "ssh", "tunnel", "command", "config"])
        );
    }
}
