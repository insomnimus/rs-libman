use crate::{
	prompt, split_command,
	command::Cmd, handler::Handler, read_bool, read_input, read_option, read_option_bool,
	SpotifyResult,
};
use rspotify::{
	blocking::client::Spotify,
model::{
	playlist::{SimplifiedPlaylist, FullPlaylist},
	playing::Playing,
	},
};
use regex::Regex;

pub struct Controller {
	client: Spotify,
	user: String,
	handlers: Vec<Handler>,
	prompt: String,
	playing: bool,
	last_pl: Option<SimplifiedPlaylist>,
}

impl Controller {
	pub fn start(&mut self) {
		let re_vol= Regex::new(r"^\s*(\-|\+)\s*(\d+)\s*$").unwrap();
		loop {
			let input = prompt(&self.prompt);
			// check for some builtins
			if input.is_empty() {
				self.toggle();
			} else if input.starts_with("prompt") {
				self.set_prompt(input.strip_prefix("prompt").unwrap_or(""));
			} else if let Some(cap) = re_vol.captures(&input) {
				let op = cap.get(1).unwrap().as_str();
				let n = cap.get(2).unwrap().as_str().parse::<i32>().unwrap();
				self.change_volume(if op == "+" { n } else { -n });
			} else {
				// check handlers
				let (cmd, args) = split_command(&input);
				let h = match self.handlers.iter().find(|h| h.is_match(cmd)) {
					None => {
						println!("{} is not a known command", cmd);
					}
					Some(h) => {
						self.exec_cmd(&h.cmd, args);
					}
				};
			}
		}
	}
}

impl Controller {
	fn exec_cmd(&mut self, c: &Cmd, args: Option<&str>) -> SpotifyResult{
		use Cmd::*;
		match c {
			// search commands
			Search => self.search(args),
			SearchTrack => self.search_track(args),
			SearchArtist => self.search_artist(args),
			SearchPlaylist => self.search_playlist(args),
			SearchAlbum => self.search_album(args),

			// play commands (i'm feeling lucky kind)
			PlayTrack => self.play_first_track(args),
			PlayAlbum => self.play_first_album(args),
			PlayArtist => self.play_first_artist(args),
			PlayPlaylist => self.play_first_playlist(args),

			// player commands
			Volume => self.set_volume(args),
			Shuffle => self.shuffle(args),
			Repeat => self.repeat(args),
			Next => self.next(),
			Prev => self.prev(),

			// library commands
			CreatePlaylist => self.create_playlist(args),
			EditPlaylist => self.edit_playlist(args),
			DeletePlaylist => self.delete_playlist(args),
			SavePlaying => self.save_playing(args),
			RemovePlaying => self.remove_playing(args),

			// misc commands
			PlayUserPlaylist => self.play_user_playlist(args),
			Show => self.show(args),
			Device => self.set_device(args),
		}
	}
}

// library commands
impl Controller {
	fn create_playlist(&self, arg: Option<&str>) -> SpotifyResult {
		let name = match arg.as_ref() {
			None => {
				let s = read_input("playlist name");
				if s.is_empty() {
					println!("cancelled");
					return Ok(());
				}
				s
			}
			Some(s) => {
				println!("playlist name: {}", s);
				s.to_string()
			}
		};

		let description = read_input("playlist description");
		let public = read_option_bool("should the playlist be public?");
		let confirm = read_bool(&format!("create playlist {}?", &name));
		if confirm {
			self.client
				.user_playlist_create(&self.user, &name, public, description)
				.map(|_| {
					println!("created new playlist {}", &name);
				})
		} else {
			println!("aborted");
			Ok(())
		}
	}

	fn edit_playlist(&self, arg: Option<&str>) -> SpotifyResult {
		let pl = match self.choose_playlist(arg)? {
			None => {
				return Ok(());
			}
			Some(p) => p,
		};

		let name = read_option(&format!("playlist name ({})", &pl.name));
		let description = read_option("playlist description (skip to not change)");
		let public = read_option_bool("public");

		if read_bool(&format!("change details for {}?", &pl.name)) {
			self.client
				.user_playlist_change_detail(&self.user, &pl.id, name.map(|s| s.as_str()), public, description, None)
				.map(|_| {
					println!("edited {}", &pl.name);
				})
		} else {
			println!("cancelled");
			Ok(())
		}
	}

	fn delete_playlist(&mut self, arg: Option<&str>) -> SpotifyResult {
		let pl = match self.choose_playlist(arg)? {
			None => {
				println!("cancelled");
				return Ok(());
			}
			Some(p) => p,
		};

		if read_bool(&format!("delete {}?", &pl.name)) {
			self.client
				.user_playlist_unfollow(&self.user, &pl.id)
				.map(|_| {
					if let Some(s) = self.last_pl.as_ref() {
						if s.id.eq(&pl.id) {
							self.last_pl = None;
						}
					}
					println!("deleted {}", &pl.name);
				})
		} else {
			println!("cancelled");
			Ok(())
		}
	}

	fn save_playing(&self, arg: Option<&str>) -> SpotifyResult {
		let track = match self.playing_track()? {
			None => {
				println!("not playing anything");
				return Ok(());
			}
			Some(t) => t,
		};

		let pl = match self.choose_playlist(arg)? {
			None => {
				println!("cancelled");
				return Ok(());
			}
			Some(p) => p,
		};

		self.client
			.user_playlist_add_tracks(&self.user, &pl.id, &[track.id], Some(0))
			.map(|_| {
				println!("saved to {}", &pl.name);
			})
	}

	fn remove_playing(&self, arg: Option<&str>) -> SpotifyResult {
		let track = match self.playing_track()? {
			Some(t) => t,
			None => {
				println!("not playing anything");
				return Ok(());
			}
		};

		let pl = match self.choose_playlist(arg)? {
			Some(p) => p,
			None => {
				println!("cancelled");
				return Ok(());
			}
		};

		self.client
			.user_playlist_remove_all_occurrences_of_tracks(&self.user, &pl.id, &[track.id], None)
			.map(|_| {
				println!("removed from {}", &pl.name);
			})
	}
}

// search commands
impl Controller {
	fn search(&self, arg: Option<&str>) -> SpotifyResult {
		todo!()
	}

	fn search_track(&self, arg: Option<&str>) -> SpotifyResult {
		todo!()
	}

	fn search_artist(&self, arg: Option<&str>) -> SpotifyResult {
		todo!()
	}

	fn search_album(&self, arg: Option<&str>) -> SpotifyResult {
		todo!()
	}

	fn search_playlist(&self, arg: Option<&str>) -> SpotifyResult {
		todo!()
	}
}

// play-first commands
impl Controller {
	fn play_first_track(&self, arg: Option<&str>) -> SpotifyResult {
		todo!()
	}

	fn play_first_album(&self, arg: Option<&str>) -> SpotifyResult {
		todo!()
	}

	fn play_first_artist(&self, arg: Option<&str>) -> SpotifyResult {
		todo!()
	}

	fn play_first_playlist(&self, arg: Option<&str>) -> SpotifyResult {
		todo!()
	}
}

// player commands
impl Controller {
	fn set_volume(&self, arg: Option<&str>) -> SpotifyResult {
		todo!()
	}

	fn change_volume(&self, n: i32) -> SpotifyResult {
		todo!()
	}

	fn shuffle(&self, arg: Option<&str>) -> SpotifyResult {
		todo!()
	}

	fn repeat(&self, arg: Option<&str>) -> SpotifyResult {
		todo!()
	}

	fn toggle(&mut self) -> SpotifyResult {
		todo!()
	}

	fn prev(&mut self) -> SpotifyResult {
		todo!()
	}

	fn next(&mut self) -> SpotifyResult {
		todo!()
	}
}

// misc commands
impl Controller {
	fn set_prompt(&mut self, s: &str) -> SpotifyResult {
		let s = s.trim();
		if s.is_empty() {
			println!("missing argument: text\ntype `help prompt` for the usage");
		} else {
			self.prompt.clear();
			self.prompt.push_str(s);
			self.prompt.push(' ');
		}
		Ok(())
	}

	fn set_device(&self, arg: Option<&str>) -> SpotifyResult {
		todo!()
	}

	fn show(&self, arg: Option<&str>) -> SpotifyResult {
		todo!()
	}

	fn play_user_playlist(&mut self, arg: Option<&str>) -> SpotifyResult {
		todo!()
	}
}

impl Controller{
	fn choose_playlist(&self, arg: Option<&str>) -> Result<Option<FullPlaylist>, failure::Error> {
		todo!()
	}
	
	fn playing_track(&self) -> Result<Option<Playing>, failure::Error> {
		todo!()
	}
}