use super::Controller;
use crate::command::AlbumCmd;
use rspotify::model::album::SimplifiedAlbum;

impl Controller {
    pub fn album_cmd_play(
        &mut self,
        albums: &[SimplifiedAlbum],
        arg: Option<&str>,
    ) -> Result<bool, failure::Error> {
        let arg = match arg {
            Some(a) => a,
            None => {
                self.show_album_usage(AlbumCmd::Play);
                return Ok(false);
            }
        };
        let n = match arg.parse::<usize>() {
            Ok(n) if n >= albums.len() => {
                println!("please enter a value between 0 and {}", albums.len());
                return Ok(false);
            }
            Ok(n) => n,
            Err(_) => {
                self.show_album_usage(AlbumCmd::Play);
                return Ok(false);
            }
        };

        self.play_album(&albums[n]).map(|_| true)
    }

    pub fn album_cmd_queue(
        &self,
        albums: &[SimplifiedAlbum],
        arg: Option<&str>,
    ) -> Result<bool, failure::Error> {
        let arg = match arg {
            Some(a) => a,
            None => {
                self.show_album_usage(AlbumCmd::Queue);
                return Ok(false);
            }
        };

        let _n = match arg.parse::<usize>() {
            Ok(n) if n >= albums.len() => {
                println!("please enter a number between 0 and {}", albums.len());
                return Ok(false);
            }
            Ok(n) => n,
            Err(_) => {
                self.show_album_usage(AlbumCmd::Queue);
                return Ok(false);
            }
        };

        // TODO: fetch all the tracks and queue them all
        todo!()
    }

    pub fn album_cmd_save(
        &self,
        albums: &[SimplifiedAlbum],
        arg: Option<&str>,
    ) -> Result<bool, failure::Error> {
        let arg = match arg {
            Some(a) => a,
            None => {
                self.show_album_usage(AlbumCmd::Save);
                return Ok(false);
            }
        };

        let n = match arg.parse::<usize>() {
            Ok(n) if n >= albums.len() => {
                println!("please enter a number between 0 and {}", albums.len());
                return Ok(false);
            }
            Ok(n) => n,
            Err(_) => {
                self.show_album_usage(AlbumCmd::Save);
                return Ok(false);
            }
        };

        self.save_album(&albums[n]).map(|_| true)
    }

    pub fn album_cmd_help(&self, arg: Option<&str>) {
        if let Some(a) = arg {
            if let Some(h) = self.album_handlers.iter().find(|h| h.is_match(a)) {
                h.show_help();
            } else {
                println!("{} is not a known command or alias\nrun `help` for a list of available commands", a);
            }
        } else {
            for h in &self.album_handlers {
                h.show_short_help();
            }
        }
    }

    pub fn show_album_usage(&self, cmd: AlbumCmd) {
        for h in &self.album_handlers {
            if h.cmd == cmd {
                h.show_usage();
            }
        }
    }
}
