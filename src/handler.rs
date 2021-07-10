pub struct Handler{
	pub name: Cow<'static, str>,
	pub cmd: Cmd,
	pub help: Cow<'static, str>,
	pub usage: Cow<'static, str>,
	pub aliases: Vec<Cow<'static, str>>,
}

impl Handler{
	pub fn is_match(&self, s: &str) -> bool{
		self.name == s ||
		self
		.aliases
		.iter()
		.any(|a| a == s)
	}
	
	pub fn show_help(&self) {
		println!("# {name}
aliases: {aliases}
usage:
  {usage}

{help}",
	name=&self.name,
	aliases=&self.aliases,
	usage=&self.usage,
	help=&self.help);
	}
	
	pub fn show_usage(&self) {
		println!("usage:\n  {}", &self.usage);
	}
}
