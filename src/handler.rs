use crate::command::Cmd;
use std::borrow::Cow;

pub struct Handler {
    pub name: Cow<'static, str>,
    pub cmd: Cmd,
    pub description: Cow<'static, str>,
    pub help: Cow<'static, str>,
    pub usage: Cow<'static, str>,
    pub aliases: Vec<Cow<'static, str>>,
}

impl Handler {
    pub fn is_match(&self, s: &str) -> bool {
        crate::equalfold(&self.name, s) || self.aliases.iter().any(|a| crate::equalfold(a, s))
    }

    pub fn show_help(&self) {
        println!(
            "# {name}
aliases: {aliases}
usage:
  {usage}

{help}",
            name = &self.name,
            aliases = self.aliases.join(", "),
            usage = &self.usage,
            help = &self.help
        );
    }

    pub fn show_usage(&self) {
        println!("usage:\n  {}", &self.usage);
    }

    pub fn show_short_help(&self) {
        if self.aliases.is_empty() {
            println!("{cmd}", cmd = &self.name);
        } else {
            println!(
                "{cmd} [aliases: {aliases}]",
                cmd = &self.name,
                aliases = self.aliases.join(", ")
            );
        }
        println!("	{}", &self.description);
    }
}

pub fn default_handlers() -> Vec<Handler> {
    todo!()
}
