pub mod command;
pub mod control;
pub mod handler;

use std::io::{self, BufRead, Write};
pub type SpotifyResult = ::std::result::Result<(), failure::Error>;

pub fn read_input(msg: &str) -> String {
    print!("{}: ", msg);
    io::stdout().flush().ok();
    io::stdin()
        .lock()
        .lines()
        .next()
        .unwrap()
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

pub fn read_option(msg: &str) -> Option<String> {
    let s = read_input(msg);
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

pub fn prompt(msg: &str) -> String {
    print!("{} ", msg);
    io::stdout().flush().ok();
    io::stdin()
        .lock()
        .lines()
        .next()
        .unwrap()
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

pub fn read_bool(msg: &str) -> bool {
    let msg = format!("{} [y/n]", msg);
    loop {
        let s = read_input(&msg);
        match &s.to_lowercase()[..] {
            "y" | "yes" | "true" => {
                return true;
            }
            "n" | "no" | "false" => {
                return false;
            }
            _ => {
                println!("please enter 'yes' or 'no'");
            }
        }
    }
}

pub fn read_option_bool(msg: &str) -> Option<bool> {
    let msg = format!("{} [y/n/empty]", msg);
    loop {
        let s = read_input(&msg);
        match &s.to_lowercase()[..] {
            "" => {
                return None;
            }
            "n" | "no" | "false" => {
                return Some(false);
            }
            "y" | "yes" | "true" => {
                return Some(true);
            }
            _ => {
                println!("please enter 'yes', 'no' or nothing");
            }
        }
    }
}

/// This would be `splitn_whitespace(2)` if it existed.
pub fn split_command<'a>(s: &'a str) -> (&'a str, Option<&'a str>) {
    let first_space = s
        .chars()
        .enumerate()
        .find(|(_, c)| c.is_whitespace())
        .map(|(i, _)| i);

    if first_space.is_none() {
        return (s, None);
    }

    let first_space = first_space.unwrap();
    let arg = s
        .chars()
        .skip(first_space)
        .enumerate()
        .find(|(_, c)| !c.is_whitespace())
        .map(|(i, _)| i + first_space)
        .map(|i| &s[i + 1..])
        .and_then(|s| if s.is_empty() { None } else { Some(s) });

    (&s[..first_space], arg)
}

pub fn read_number(min: usize, max: usize) -> Option<usize> {
    let msg = format!("[{}-{}, blank to cancel]>", min, max);
    loop {
        let input = prompt(&msg);
        if input.is_empty() {
            return None;
        }
        if let Ok(n) = input.parse::<usize>() {
            return Some(n);
        } else {
            println!("invalid input, please enter again");
        }
    }
}
