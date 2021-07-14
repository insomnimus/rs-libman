use super::Controller;
use crate::command::ArtistCmd;
use rspotify::model::artist::FullArtist;

impl Controller {
    pub fn artist_cmd_play(
        &mut self,
        artists: &[FullArtist],
        arg: Option<&str>,
    ) -> Result<bool, failure::Error> {
        let arg = match arg {
            Some(a) => a,
            None => {
                self.show_artist_usage(ArtistCmd::Play);
                return Ok(false);
            }
        };

        let n = match arg.parse::<usize>() {
            Ok(n) if n >= artists.len() => {
                println!("please enter a value between 0 and {}", artists.len());
                return Ok(false);
            }
            Ok(n) => n,
            Err(_) => {
                self.show_artist_usage(ArtistCmd::Play);
                return Ok(false);
            }
        };

        self.play_artist(&artists[n]).map(|_| true)
    }

    pub fn artist_cmd_queue(
        &self,
        artists: &[FullArtist],
        arg: Option<&str>,
    ) -> Result<bool, failure::Error> {
        let arg = match arg {
            Some(a) => a,
            None => {
                self.show_artist_usage(ArtistCmd::Queue);
                return Ok(false);
            }
        };

        let n = match arg.parse::<usize>() {
            Ok(n) if n >= artists.len() => {
                println!("please enter a number between 0 and {}", artists.len());
                return Ok(false);
            }
            Ok(n) => n,
            Err(_) => {
                self.show_artist_usage(ArtistCmd::Queue);
                return Ok(false);
            }
        };

        self.queue(artists[n].uri.clone()).map(|_| {
            println!("added {} to the queue", &artists[n].name);
            true
        })
    }

    pub fn artist_cmd_follow(
        &self,
        artists: &[FullArtist],
        arg: Option<&str>,
    ) -> Result<bool, failure::Error> {
        let arg = match arg {
            Some(a) => a,
            None => {
                self.show_artist_usage(ArtistCmd::Follow);
                return Ok(false);
            }
        };

        let n = match arg.parse::<usize>() {
            Ok(n) if n >= artists.len() => {
                println!("please enter a number between 0 and {}", artists.len());
                return Ok(false);
            }
            Ok(n) => n,
            Err(_) => {
                self.show_artist_usage(ArtistCmd::Follow);
                return Ok(false);
            }
        };

        self.follow_artist(&artists[n]).map(|_| true)
    }

    pub fn artist_cmd_help(&self, arg: Option<&str>) {
        if let Some(a) = arg {
            if let Some(h) = self.artist_handlers.iter().find(|h| h.is_match(a)) {
                h.show_help();
            } else {
                println!("{} is not a known command or alias\nrun `help` for a list of available commands", a);
            }
        } else {
            for h in &self.artist_handlers {
                h.show_short_help();
            }
        }
    }

    pub fn show_artist_usage(&self, cmd: ArtistCmd) {
        for h in &self.artist_handlers {
            if h.cmd == cmd {
                h.show_usage();
            }
        }
    }
}
