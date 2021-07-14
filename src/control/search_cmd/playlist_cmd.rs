use super::Controller;
use crate::command::PlaylistCmd;
use crate::playlist::Playlist;

impl Controller {
    pub fn playlist_cmd_play(
        &mut self,
        pls: &[Playlist],
        arg: Option<&str>,
    ) -> Result<bool, failure::Error> {
        let arg = match arg {
            Some(a) => a,
            None => {
                self.show_playlist_usage(PlaylistCmd::Play);
                return Ok(false);
            }
        };
        let n = match arg.parse::<usize>() {
            Ok(n) if n >= pls.len() => {
                println!("please enter a value between 0 and {}", pls.len());
                return Ok(false);
            }
            Ok(n) => n,
            Err(_) => {
                self.show_playlist_usage(PlaylistCmd::Play);
                return Ok(false);
            }
        };

        self.play_playlist(&pls[n]).map(|_| true)
    }

    pub fn playlist_cmd_queue(
        &self,
        pls: &[Playlist],
        arg: Option<&str>,
    ) -> Result<bool, failure::Error> {
        let arg = match arg {
            Some(a) => a,
            None => {
                self.show_playlist_usage(PlaylistCmd::Queue);
                return Ok(false);
            }
        };

        let n = match arg.parse::<usize>() {
            Ok(n) if n >= pls.len() => {
                println!("please enter a number between 0 and {}", pls.len());
                return Ok(false);
            }
            Ok(n) => n,
            Err(_) => {
                self.show_playlist_usage(PlaylistCmd::Queue);
                return Ok(false);
            }
        };

        self.queue(pls[n].uri().to_string()).map(|_| {
            println!("added {} to the queue", &pls[n].name());
            true
        })
    }

    pub fn playlist_cmd_follow(
        &self,
        pls: &[Playlist],
        arg: Option<&str>,
    ) -> Result<bool, failure::Error> {
        let arg = match arg {
            Some(a) => a,
            None => {
                self.show_playlist_usage(PlaylistCmd::Follow);
                return Ok(false);
            }
        };

        let n = match arg.parse::<usize>() {
            Ok(n) if n >= pls.len() => {
                println!("please enter a number between 0 and {}", pls.len());
                return Ok(false);
            }
            Ok(n) => n,
            Err(_) => {
                self.show_playlist_usage(PlaylistCmd::Follow);
                return Ok(false);
            }
        };

        self.follow_playlist(&pls[n]).map(|_| true)
    }

    pub fn playlist_cmd_help(&self, arg: Option<&str>) {
        if let Some(a) = arg {
            if let Some(h) = self.playlist_handlers.iter().find(|h| h.is_match(a)) {
                h.show_help();
            } else {
                println!("{} is not a known command or alias\nrun `help` for a list of available commands", a);
            }
        } else {
            for h in &self.playlist_handlers {
                h.show_short_help();
            }
        }
    }

    pub fn show_playlist_usage(&self, cmd: PlaylistCmd) {
        for h in &self.playlist_handlers {
            if h.cmd == cmd {
                h.show_usage();
            }
        }
    }
}
