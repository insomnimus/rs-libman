use crate::{
    command::Cmd, handler::Handler, prompt, read_bool, read_input, read_number, read_option,
    read_option_bool, split_command, SpotifyResult,
};
use regex::Regex;
use rspotify::{
    blocking::client::Spotify,
    model::{
        device::Device,
        playlist::{FullPlaylist, SimplifiedPlaylist},
        track::FullTrack,
        PlayingItem,
    },
};
use std::{convert::TryFrom, mem};

pub struct Controller {
    client: Spotify,
    user: String,
    handlers: Vec<Handler>,
    prompt: String,
    playing: bool,
    last_pl: Option<SimplifiedPlaylist>,
}

impl Controller {
    pub fn start(mut self) {
        let re_vol = Regex::new(r"^\s*(\-|\+)\s*(\d+)\s*$").unwrap();
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
                        self.exec_cmd(h.cmd, args);
                    }
                };
            }
        }
    }

    fn exec_cmd(&mut self, c: Cmd, args: Option<&str>) -> SpotifyResult {
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
            SetVolume => self.set_volume(args),
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

    fn show_usage(&self, cmd: Cmd) {
        self.handlers.iter().find(|h| h.cmd == cmd).map(|h| {
            h.show_usage();
        });
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

        let name = read_input(&format!("playlist name ({})", &pl.name));
        let name = if name.is_empty() {
            None
        } else {
            Some(&name[..])
        };

        let description = read_option("playlist description (skip to not change)");
        let public = read_option_bool("public");

        if read_bool(&format!("change details for {}?", &pl.name)) {
            self.client
                .user_playlist_change_detail(&self.user, &pl.id, name, public, description, None)
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
        let id = if track.id.is_none() {
            track.uri
        } else {
            track.id.unwrap()
        };
        self.client
            .user_playlist_add_tracks(&self.user, &pl.id, &[id], Some(0))
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
        let id = if track.id.is_none() {
            track.uri
        } else {
            track.id.unwrap()
        };
        self.client
            .user_playlist_remove_all_occurrences_of_tracks(&self.user, &pl.id, &[id], None)
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
        let n = match arg {
            Some(s) => match s.parse::<u8>() {
                Ok(n) => n,
                Err(_) => {
                    println!("{}: the value must be an integer between 0 and 100", s);
                    return Ok(());
                }
            },
            None => {
                self.show_usage(Cmd::SetVolume);
                return Ok(());
            }
        };

        let n = if n > 100 { 100_u8 } else { n };

        self.client.volume(n, None)
    }

    fn change_volume(&self, mut n: i32) -> SpotifyResult {
        let mut active_device = match self
            .client
            .device()?
            .devices
            .into_iter()
            .find(|d| d.is_active)
        {
            Some(d) => d,
            None => {
                println!("no active device detected");
                return Ok(());
            }
        };

        n += active_device.volume_percent as i32;
        if n < 0 {
            n = 0;
        } else if n > 100 {
            n = 100;
        }

        self.client
            .volume(
                u8::try_from(n).unwrap(),
                Some(mem::take(&mut active_device.id)),
            )
            .map(|_| {
                println!("{}: set to {}%", &active_device.name, n);
            })
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
        if let Some(dev) = self.choose_device(arg)? {
            self.client.transfer_playback(&dev.id, false).map(|_| {
                println!("playing on {}", &dev.name);
            })
        } else {
            println!("cancelled");
            Ok(())
        }
    }

    fn show(&self, arg: Option<&str>) -> SpotifyResult {
        todo!()
    }

    fn play_user_playlist(&mut self, arg: Option<&str>) -> SpotifyResult {
        todo!()
    }
}

impl Controller {
    fn choose_playlist(&self, arg: Option<&str>) -> Result<Option<FullPlaylist>, failure::Error> {
        todo!()
    }

    fn playing_track(&self) -> Result<Option<FullTrack>, failure::Error> {
        self.client.current_playing(None, None).map(|resp| {
            resp.and_then(|x| x.item).and_then(|item| {
                if let PlayingItem::Track(t) = item {
                    Some(t)
                } else {
                    None
                }
            })
        })
    }

    fn choose_device(&self, arg: Option<&str>) -> Result<Option<Device>, failure::Error> {
        let mut devs = self.client.device()?.devices;
        Ok(if let Some(name) = arg {
            devs.into_iter().find(|d| d.name.eq(name))
        } else {
            if devs.is_empty() {
                println!("did not detect any device");
                None
            } else {
                for (i, d) in devs.iter().enumerate() {
                    println!("# {} : {}", i, &d.name);
                }
                read_number(0, devs.len()).map(|n| devs.remove(n))
            }
        })
    }
}
