mod config;

use crate::config::Config;
use num_integer::Integer;
use shell_completion::{BashCompletionInput, CompletionInput};
use std::collections::VecDeque;

struct Context {
    config: Config,
    up_to_cursor: String,
    unparsed_args_up_to_cursor: VecDeque<String>,
    input: BashCompletionInput,
}

impl Context {
    fn cursor_on_new_word(&self) -> bool {
        self.up_to_cursor.ends_with(' ') || !self.unparsed_args_up_to_cursor.is_empty()
    }
}

fn main() {
    let input = BashCompletionInput::from_args().expect("Missing expected environment variables");

    // We will only cover up to the current position
    let mut trim = input.line[0..input.cursor_position].to_string();

    // Split can fail when input contains unmatched quote
    // If the user is part way through a quote lets assume it to be one word
    if trim.matches('\'').count().is_odd() {
        trim.push('\'');
    }
    if trim.matches('\"').count().is_odd() {
        trim.push('\"');
    }
    let split = match shell_words::split(&trim) {
        Err(_) => return,
        Ok(s) => s,
    };

    let mut ctx = Context {
        config: Config::load().unwrap_or_default(),
        up_to_cursor: trim.to_string(),
        unparsed_args_up_to_cursor: split.into_iter().skip(1).collect(),
        input,
    };

    let subcommands = vec!["rdp", "ssh", "tunnel", "config"];
    match ctx.unparsed_args_up_to_cursor.pop_front() {
        None => {
            ctx.input.complete_subcommand(subcommands);
        }
        Some(arg) => {
            if ctx.cursor_on_new_word() {
                match arg.as_str() {
                    "rdp" => complete_rdp(ctx),
                    "ssh" => complete_ssh(ctx),
                    "tunnel" => complete_tunnel(ctx),
                    _ => {}
                }
            } else {
                ctx.input.complete_subcommand(subcommands);
            }
        }
    }
}

fn complete_rdp(mut ctx: Context) {
    let possibilities = ctx
        .config
        .rdp
        .iter()
        .map(|r| r.name.as_str())
        .collect::<Vec<_>>();
    match ctx.unparsed_args_up_to_cursor.pop_front() {
        None => {
            ctx.input.complete_subcommand(possibilities);
        }
        Some(_arg) => {
            if ctx.cursor_on_new_word() {
                unimplemented!()
            } else {
                ctx.input.complete_subcommand(possibilities);
            }
        }
    }
}

fn complete_ssh(mut ctx: Context) {
    let possibilities = ctx
        .config
        .ssh
        .iter()
        .map(|r| r.name.as_str())
        .collect::<Vec<_>>();
    match ctx.unparsed_args_up_to_cursor.pop_front() {
        None => {
            ctx.input.complete_subcommand(possibilities);
        }
        Some(_arg) => {
            if ctx.cursor_on_new_word() {
                unimplemented!()
            } else {
                ctx.input.complete_subcommand(possibilities);
            }
        }
    }
}

fn complete_tunnel(mut ctx: Context) {
    let possibilities = ctx
        .config
        .tunnels
        .iter()
        .map(|r| r.name.as_str())
        .collect::<Vec<_>>();
    match ctx.unparsed_args_up_to_cursor.pop_front() {
        None => {
            ctx.input.complete_subcommand(possibilities);
        }
        Some(_arg) => {
            if ctx.cursor_on_new_word() {
                unimplemented!()
            } else {
                ctx.input.complete_subcommand(possibilities);
            }
        }
    }
}
