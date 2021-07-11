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

pub fn split_command<'a>(s: &'a str) -> (&'a str, Option<&'a str>) {
	todo!()
}
