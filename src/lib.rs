use std::{
	io::{self, BufRead, Write},
};

pub fn read_input(msg: &str) -> String{
	print!("{}: ", msg);
	io::stdout().flush().ok();
	io::stdin()
	.lock()
	.lines()
	.next()
	.unwrap()
	.unwrap_or_default()
}

pub fn prompt(msg: &str) -> String{
	print!("{} ", msg);
	io::stdout().flush().ok();
	io::stdin()
	.lock()
	.lines()
	.next()
	.unwrap()
	.unwrap_or_default()
}
