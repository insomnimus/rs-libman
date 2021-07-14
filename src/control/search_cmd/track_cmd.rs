use super::Controller;
use crate::command::TrackCmd;
use rspotify::model::track::FullTrack;

impl Controller {
    pub fn track_cmd_play(
        &mut self,
        tracks: &[FullTrack],
        arg: Option<&str>,
    ) -> Result<bool, failure::Error> {
        let arg = match arg {
            Some(a) => a,
            None => {
                self.show_track_usage(TrackCmd::Play);
                return Ok(false);
            }
        };
        let n = match arg.parse::<usize>() {
            Ok(n) if n >= tracks.len() => {
                println!("please enter a value between 0 and {}", tracks.len());
                return Ok(false);
            }
            Ok(n) => n,
            Err(_) => {
                self.show_track_usage(TrackCmd::Play);
                return Ok(false);
            }
        };

        self.play_track(&tracks[n]).map(|_| true)
    }

    pub fn track_cmd_queue(
        &self,
        tracks: &[FullTrack],
        arg: Option<&str>,
    ) -> Result<bool, failure::Error> {
        let arg = match arg {
            Some(a) => a,
            None => {
                self.show_track_usage(TrackCmd::Queue);
                return Ok(false);
            }
        };

        let n = match arg.parse::<usize>() {
            Ok(n) if n >= tracks.len() => {
                println!("please enter a number between 0 and {}", tracks.len());
                return Ok(false);
            }
            Ok(n) => n,
            Err(_) => {
                self.show_track_usage(TrackCmd::Queue);
                return Ok(false);
            }
        };

        self.queue_track(&tracks[n]).map(|_| true)
    }

    pub fn track_cmd_like(
        &self,
        tracks: &[FullTrack],
        arg: Option<&str>,
    ) -> Result<bool, failure::Error> {
        let arg = match arg {
            Some(a) => a,
            None => {
                self.show_track_usage(TrackCmd::Like);
                return Ok(false);
            }
        };

        let n = match arg.parse::<usize>() {
            Ok(n) if n >= tracks.len() => {
                println!("please enter a number between 0 and {}", tracks.len());
                return Ok(false);
            }
            Ok(n) => n,
            Err(_) => {
                self.show_track_usage(TrackCmd::Like);
                return Ok(false);
            }
        };

        self.like_track(&tracks[n]).map(|_| true)
    }

    pub fn track_cmd_help(&self, arg: Option<&str>) {
        if let Some(a) = arg {
            if let Some(h) = self.track_handlers.iter().find(|h| h.is_match(a)) {
                h.show_help();
            } else {
                println!("{} is not a known command or alias\nrun `help` for a list of available commands", a);
            }
        } else {
            for h in &self.track_handlers {
                h.show_short_help();
            }
        }
    }

    pub fn track_cmd_save(
        &mut self,
        tracks: &[FullTrack],
        arg: Option<&str>,
    ) -> Result<bool, failure::Error> {
        let arg = match arg {
            Some(a) => a,
            None => {
                self.show_track_usage(TrackCmd::Save);
                return Ok(false);
            }
        };

        let (index, arg) = crate::split_command(arg);
        let n = match index.parse::<usize>() {
            Ok(n) if n >= tracks.len() => {
                println!("please enter a number between 0 and {}", tracks.len());
                return Ok(false);
            }
            Ok(n) => n,
            Err(_) => {
                self.show_track_usage(TrackCmd::Save);
                return Ok(false);
            }
        };

        self.save_track(tracks[n].clone(), arg).map(|_| true)
    }

    pub fn show_track_usage(&self, cmd: TrackCmd) {
        for h in &self.track_handlers {
            if h.cmd == cmd {
                h.show_usage();
            }
        }
    }
}
