pub mod search_cmd;

use crate::{
    command::{AlbumCmd, ArtistCmd, Cmd, PlaylistCmd, TrackCmd},
    handler::{self, Handler},
    playlist::Playlist,
    prompt, read_bool, read_input, read_number, read_option, read_option_bool, search,
    split_command, SpotifyResult,
};

use chrono::Utc;
use regex::Regex;

use rspotify::{
    blocking::client::Spotify,
    model::{
        album::SimplifiedAlbum, artist::FullArtist, device::Device, offset::Offset,
        playlist::PlaylistTrack, track::FullTrack, PlayingItem,
    },
    senum::{AdditionalType, RepeatState},
};
use std::{convert::TryFrom, mem};

pub struct Controller {
    client: Spotify,
    user: String,
    handlers: Vec<Handler<Cmd>>,
    track_handlers: Vec<Handler<TrackCmd>>,
    artist_handlers: Vec<Handler<ArtistCmd>>,
    album_handlers: Vec<Handler<AlbumCmd>>,
    playlist_handlers: Vec<Handler<PlaylistCmd>>,
    prompt: String,
    playing: bool,
    last_pl: Option<Playlist>,
    pl_cache: Option<Vec<Playlist>>,
    device: Option<String>,
}

impl Controller {
    pub fn new(
        client: Spotify,
        user_id: String,
        handlers: Vec<Handler<Cmd>>,
        prompt: String,
    ) -> Self {
        Self {
            client,
            prompt,
            user: user_id,
            handlers,
            track_handlers: handler::default_track_handlers(),
            artist_handlers: handler::default_artist_handlers(),
            album_handlers: handler::default_album_handlers(),
            playlist_handlers: handler::default_playlist_handlers(),
            playing: false,
            last_pl: None,
            pl_cache: None,
            device: None,
        }
    }

    pub fn start(&mut self) {
        let re_vol = Regex::new(r"^\s*(\-|\+)\s*(\d+)\s*$").unwrap();
        loop {
            let input = prompt(&self.prompt);

            if let Err(e) = if input.is_empty() {
                self.toggle()
            } else if let Some(cap) = re_vol.captures(&input) {
                let op = cap.get(1).unwrap().as_str();
                let n = cap.get(2).unwrap().as_str().parse::<i32>().unwrap();
                self.change_volume(if op == "+" { n } else { -n })
            } else {
                // check handlers
                let (cmd, args) = split_command(&input);
                let cmd = match self.handlers.iter().find(|h| h.is_match(cmd)) {
                    None => {
                        println!("{} is not a known command", cmd);
                        continue;
                    }
                    Some(h) => h.cmd,
                };
                self.exec_cmd(cmd, args)
            } {
                println!("error: {}", e);
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

            // play-first commands
            PlayFirstTrack => self.play_first_track(args),
            PlayFirstAlbum => self.play_first_album(args),
            PlayFirstArtist => self.play_first_artist(args),
            PlayFirstPlaylist => self.play_first_playlist(args),

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
            Help => self.show_help(args),
            PlayUserPlaylist => self.play_user_playlist(args),
            Show => self.show(args),
            SetDevice => self.set_device(args),
            Prompt => self.set_prompt(args),
        }
    }

    fn show_usage(&self, cmd: Cmd) {
        if let Some(h) = self.handlers.iter().find(|h| h.cmd == cmd) {
            h.show_usage();
        }
    }
}

// library commands
impl Controller {
    fn create_playlist(&mut self, arg: Option<&str>) -> SpotifyResult {
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
                .map(|pl| {
                    println!("created new playlist {}", &pl.name);
                    if let Some(v) = self.pl_cache.as_mut() {
                        v.insert(0, pl.into());
                    }
                })
        } else {
            println!("aborted");
            Ok(())
        }
    }

    fn edit_playlist(&mut self, arg: Option<&str>) -> SpotifyResult {
        let pl = match self.choose_user_playlist(arg)? {
            None => {
                return Ok(());
            }
            Some(p) => p,
        };

        let name = read_input(&format!("playlist name ({})", pl.name()));
        let name = if name.is_empty() {
            None
        } else {
            Some(&name[..])
        };

        let description = read_option("playlist description (skip to not change)");
        let public = read_option_bool("public");

        if read_bool(&format!("change details for {}?", &pl.name())) {
            self.client
                .user_playlist_change_detail(&self.user, pl.id(), name, public, description, None)
                .map(|_| {
                    println!("edited {}", pl.name());
                    if let Some(n) = name {
                        if !n.eq(pl.name()) {
                            if let Some(v) = self.pl_cache.as_mut() {
                                for p in v.iter_mut() {
                                    if p.id() == pl.id() {
                                        p.set_name(n);
                                    }
                                }
                            }
                        }
                    }
                })
        } else {
            println!("cancelled");
            Ok(())
        }
    }

    fn delete_playlist(&mut self, arg: Option<&str>) -> SpotifyResult {
        let pl = match self.choose_user_playlist(arg)? {
            None => {
                println!("cancelled");
                return Ok(());
            }
            Some(p) => p,
        };

        if read_bool(&format!("delete {}?", pl.name())) {
            self.client
                .user_playlist_unfollow(&self.user, pl.id())
                .map(|_| {
                    if let Some(s) = self.last_pl.as_ref() {
                        if s.id().eq(pl.id()) {
                            self.last_pl = None;
                        }
                    }
                    if let Some(v) = self.pl_cache.as_mut() {
                        v.retain(|p| p.id() != pl.id());
                    }
                    println!("deleted {}", pl.name());
                })
        } else {
            println!("cancelled");
            Ok(())
        }
    }

    fn save_playing(&mut self, arg: Option<&str>) -> SpotifyResult {
        let track = match self.playing_track()? {
            None => {
                println!("not playing anything");
                return Ok(());
            }
            Some(t) => t,
        };

        self.save_track(track, arg)
    }

    fn remove_playing(&mut self, arg: Option<&str>) -> SpotifyResult {
        let track = match self.playing_track()? {
            Some(t) => t,
            None => {
                println!("not playing anything");
                return Ok(());
            }
        };

        let mut pl = match self.choose_user_playlist(arg)? {
            Some(p) => p,
            None => {
                println!("cancelled");
                return Ok(());
            }
        };

        let id = match track.id.as_ref() {
            Some(i) => i.clone(),
            None => track.uri.clone(),
        };
        let was_simple = pl.is_simple();
        if was_simple {
            pl.make_full(&self.client, &self.user)?;
        }

        let mut contains = false;
        let pl = match pl {
            Playlist::Full(mut p) => {
                // remove track from playlist, if it's there
                p.tracks.items.retain(|pl_track| {
                    if pl_track
                        .track
                        .as_ref()
                        .map(|t| t.id.eq(&track.id) || t.uri.eq(&track.uri))
                        .unwrap_or(false)
                    {
                        // playlist contains the track
                        contains = true;
                        false
                    } else {
                        true
                    }
                });
                Playlist::from(p)
            }
            Playlist::Simple(_) => pl,
        };

        if !contains {
            if was_simple && !pl.is_simple() {
                // replace the playlist with the full one
                if let Some(v) = self.pl_cache.as_mut() {
                    for p in v.iter_mut() {
                        if p.id() == pl.id() {
                            *p = pl;
                            break;
                        }
                    }
                }
            }
            println!("the playlist does not have the track, no action taken");
            return Ok(());
        }
        self.client
            .user_playlist_remove_all_occurrences_of_tracks(&self.user, pl.id(), &[id], None)
            .map(|_| {
                println!("removed from {}", pl.name());
                if let Some(v) = self.pl_cache.as_mut() {
                    for p in v.iter_mut() {
                        if p.id() == pl.id() {
                            *p = pl;
                            break;
                        }
                    }
                }
            })
    }
}

// play-first commands
impl Controller {
    fn play_first_track(&mut self, arg: Option<&str>) -> SpotifyResult {
        let arg = match arg {
            Some(s) => s,
            None => {
                self.show_usage(Cmd::PlayFirstTrack);
                return Ok(());
            }
        };
        let query = search::track_query(arg);
        let tracks = search::tracks(&self.client, &query, 20)?;
        let track = match tracks.get(0) {
            Some(t) => t,
            None => {
                println!("no result for '{}'", &query);
                return Ok(());
            }
        };

        self.play_track(track)
    }

    fn play_first_album(&mut self, arg: Option<&str>) -> SpotifyResult {
        let arg = match arg {
            Some(s) => s,
            None => {
                self.show_usage(Cmd::PlayFirstAlbum);
                return Ok(());
            }
        };
        let query = search::album_query(arg);
        let albums = search::albums(&self.client, &query, 20)?;

        let alb = match albums.get(0) {
            Some(a) => a,
            None => {
                println!("no result for {}", &query);
                return Ok(());
            }
        };

        self.play_album(alb)
    }

    fn play_first_artist(&mut self, arg: Option<&str>) -> SpotifyResult {
        let arg = match arg {
            Some(a) => a,
            None => {
                self.show_usage(Cmd::PlayFirstArtist);
                return Ok(());
            }
        };

        let artists = search::artists(&self.client, arg, 20)?;
        let art = match artists.get(0) {
            Some(a) => a,
            None => {
                println!("no result for {}", arg);
                return Ok(());
            }
        };

        self.play_artist(art)
    }

    fn play_first_playlist(&mut self, arg: Option<&str>) -> SpotifyResult {
        let arg = match arg {
            Some(a) => a,
            None => {
                self.show_usage(Cmd::PlayFirstPlaylist);
                return Ok(());
            }
        };

        let pls = search::playlists(&self.client, arg, 20)?;
        let pl = match pls.get(0) {
            Some(p) => p,
            None => {
                println!("no result for {}", arg);
                return Ok(());
            }
        };

        self.play_playlist(pl)
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

        self.client.volume(n, self.device.clone())
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
        let sh = match arg {
            None => None,
            Some(s) => Some(match &s.to_lowercase()[..] {
                "on" | "true" | "yes" => true,
                "off" | "false" | "no" => false,
                _ => {
                    self.show_usage(Cmd::Shuffle);
                    return Ok(());
                }
            }),
        };
        if let Ok(Some(cont)) = self.client.current_playback(
            None,
            Some(vec![AdditionalType::Track, AdditionalType::Episode]),
        ) {
            match (sh, cont.shuffle_state) {
                (Some(x), y) if x == y => {
                    println!("shuffle = {}", cont.shuffle_state);
                    Ok(())
                }
                _ => self
                    .client
                    .shuffle(!cont.shuffle_state, self.device.clone())
                    .map(|_| {
                        println!("shuffle = {}", !cont.shuffle_state);
                    }),
            }
        } else if let Some(b) = sh {
            self.client.shuffle(b, self.device.clone()).map(|_| {
                println!("shuffle = {}", b);
            })
        } else {
            println!("could not determine shuffle state, try running `shuffle` with yes or no");
            Ok(())
        }
    }

    fn repeat(&self, arg: Option<&str>) -> SpotifyResult {
        let rep = match arg {
            Some(s) => match &s.to_lowercase()[..] {
                "off" | "false" | "no" => RepeatState::Off,
                "context" | "playlist" | "album" | "pl" => RepeatState::Context,
                "track" | "on" | "true" | "yes" => RepeatState::Track,
                _ => {
                    self.show_usage(Cmd::Repeat);
                    return Ok(());
                }
            },
            None => {
                self.show_usage(Cmd::Repeat);
                return Ok(());
            }
        };

        self.client.repeat(rep, self.device.clone()).map(|_| {
            println!("repeat = {}", rep.as_str());
        })
    }

    fn toggle(&mut self) -> SpotifyResult {
        self.playing = !self.playing;
        if self.playing {
            self.client
                .start_playback(self.device.clone(), None, None, None, None)
        } else {
            self.client.pause_playback(self.device.clone())
        }
    }

    fn prev(&mut self) -> SpotifyResult {
        self.client.previous_track(None).map(|_| {
            self.playing = true;
        })
    }

    fn next(&mut self) -> SpotifyResult {
        self.client.next_track(None).map(|_| {
            self.playing = true;
        })
    }
}

// misc commands
impl Controller {
    fn show_help(&self, arg: Option<&str>) -> SpotifyResult {
        if let Some(a) = arg {
            if let Some(h) = self.handlers.iter().find(|h| h.is_match(a)) {
                h.show_help();
            } else {
                println!(
                    "{} is not a known command or alias\nrun `help` for a list of the commands",
                    a
                );
            }
        } else {
            for h in &self.handlers {
                h.show_short_help();
            }
        }
        Ok(())
    }

    fn set_prompt(&mut self, arg: Option<&str>) -> SpotifyResult {
        match arg {
            Some(a) => {
                self.prompt.clear();
                self.prompt.push_str(a);
                self.prompt.push(' ');
            }
            None => self.show_usage(Cmd::Prompt),
        };
        Ok(())
    }

    fn set_device(&mut self, arg: Option<&str>) -> SpotifyResult {
        if let Some(dev) = self.choose_device(arg)? {
            self.client.transfer_playback(&dev.id, false).map(|_| {
                println!("playing on {}", &dev.name);
                self.device = Some(dev.id);
            })
        } else {
            println!("cancelled");
            Ok(())
        }
    }

    fn show(&mut self, arg: Option<&str>) -> SpotifyResult {
        if let Some(a) = arg {
            match a {
                "playing" | "track" => self.show_playing(),
                "lib" | "pl" => self.show_user_playlists(),
                _ => self.show_user_playlist(a),
            }
        } else {
            self.show_playback()
        }
    }

    fn play_user_playlist(&mut self, arg: Option<&str>) -> SpotifyResult {
        if let Some(pl) = self.choose_user_playlist(arg)? {
            self.client
                .start_playback(
                    self.device.clone(),
                    Some(pl.uri().to_string()),
                    None,
                    None,
                    None,
                )
                .map(|_| {
                    println!("playing {}", pl.name());
                    self.playing = true;
                    self.last_pl = Some(pl);
                })
        } else {
            println!("cancelled");
            Ok(())
        }
    }
}

// utilities
impl Controller {
    fn choose_user_playlist(
        &mut self,
        arg: Option<&str>,
    ) -> Result<Option<Playlist>, failure::Error> {
        let mut pls = self.get_playlists()?;
        if pls.is_empty() {
            println!("you don't seem to have any playlist");
            return Ok(None);
        }

        Ok(if let Some(a) = arg {
            pls.into_iter().find(|p| p.name_eq(a)).or_else(|| {
                println!("you don't seem to have a playlist named {}", a);
                None
            })
        } else {
            for (i, p) in pls.iter().enumerate() {
                println!("#{no:2} | {name}", no = i, name = p.name());
            }
            read_number(0, pls.len()).map(|n| pls.remove(n))
        })
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
        } else if devs.is_empty() {
            println!("did not detect any device");
            None
        } else {
            for (i, d) in devs.iter().enumerate() {
                println!("# {} : {}", i, &d.name);
            }
            read_number(0, devs.len()).map(|n| devs.remove(n))
        })
    }

    fn get_playlists(&mut self) -> Result<Vec<Playlist>, failure::Error> {
        if let Some(cache) = self.pl_cache.as_ref() {
            Ok(cache.to_vec())
        } else {
            let pls = self
                .client
                .current_user_playlists(Some(50), None)
                .map(|p| p.items.into_iter().map(Playlist::from).collect::<Vec<_>>())?;
            self.pl_cache = Some(pls.clone());
            Ok(pls)
        }
    }
}

impl Controller {
    fn play_track(&mut self, track: &FullTrack) -> SpotifyResult {
        self.client
            .start_playback(
                self.device.clone(),
                None,
                Some(vec![track.uri.clone()]),
                None,
                None,
            )
            .map(|_| {
                self.playing = true;
                println!(
                    "playing {} [{}] by {}",
                    &track.name,
                    &track.album.name,
                    crate::join_artists(&track.artists)
                );
            })
    }

    fn play_album(&mut self, alb: &SimplifiedAlbum) -> SpotifyResult {
        if let Some(uri) = alb.uri.as_ref() {
            self.client.start_playback(
                self.device.clone(),
                Some(uri.clone()),
                None,
                Some(Offset {
                    position: Some(0),
                    uri: None,
                }),
                None,
            )
        } else if let Some(id) = alb.id.as_ref() {
            self.client.start_playback(
                self.device.clone(),
                Some(id.clone()),
                None,
                Some(Offset {
                    position: Some(0),
                    uri: None,
                }),
                None,
            )
        } else {
            // can't do anything here, uri and id unavailable
            println!("error: the track uri and id can't be found");
            return Ok(());
        }
        .map(|_| {
            self.playing = true;
            println!(
                "playing {} by {}",
                &alb.name,
                crate::join_artists(&alb.artists)
            );
        })
    }

    fn play_artist(&mut self, art: &FullArtist) -> SpotifyResult {
        self.client
            .start_playback(self.device.clone(), Some(art.uri.clone()), None, None, None)
            .map(|_| {
                self.playing = true;
                println!("playing {}", &art.name);
            })
    }

    fn play_playlist(&mut self, pl: &Playlist) -> SpotifyResult {
        self.client
            .start_playback(
                self.device.clone(),
                Some(pl.uri().to_string()),
                None,
                Some(Offset {
                    position: Some(0),
                    uri: None,
                }),
                None,
            )
            .map(|_| {
                self.playing = true;
                println!("playing {}", &pl.name());
            })
    }
}

impl Controller {
    fn show_playing(&mut self) -> SpotifyResult {
        if let Some(playing) = self.client.current_playing(
            None,
            Some(vec![AdditionalType::Track, AdditionalType::Episode]),
        )? {
            self.playing = playing.is_playing;
            match playing.item.as_ref() {
                Some(PlayingItem::Track(t)) => {
                    println!("{} by {}", &t.name, crate::join_artists(&t.artists));
                    println!("playing = {}", &playing.is_playing);
                }
                Some(PlayingItem::Episode(e)) => {
                    println!(
                        "{} from {} by {}\n{}",
                        &e.name, &e.show.name, &e.show.publisher, &e.description
                    );
                    println!("playing = {}", &playing.is_playing);
                }
                None => {
                    println!("not playing anything");
                }
            };
        } else {
            println!("not playing anything");
        }
        Ok(())
    }

    fn show_playback(&mut self) -> SpotifyResult {
        if let Some(playing) = self.client.current_playback(
            None,
            Some(vec![AdditionalType::Track, AdditionalType::Episode]),
        )? {
            self.playing = playing.is_playing;
            println!("device  | {}", &playing.device.name);
            println!("repeat  | {}", playing.repeat_state.as_str());
            println!("shuffle | {}", playing.shuffle_state);
            println!("playing | {}", playing.is_playing);
            match &playing.item.as_ref() {
                Some(PlayingItem::Track(t)) => {
                    println!("{} by {}", &t.name, crate::join_artists(&t.artists));
                }
                Some(PlayingItem::Episode(e)) => {
                    println!(
                        "{} from {} by {}\n{}",
                        &e.name, &e.show.name, &e.show.publisher, &e.description
                    );
                }
                None => {}
            };
        } else {
            println!("not playing anything");
        }
        Ok(())
    }

    fn show_user_playlist(&self, _name: &str) -> SpotifyResult {
        println!("unimplemented");
        Ok(())
    }

    fn show_user_playlists(&mut self) -> SpotifyResult {
        self.get_playlists().map(|pls| {
            for pl in &pls {
                println!("{}", pl.name());
            }
        })
    }
}

impl Controller {
    fn save_track(&mut self, track: FullTrack, arg: Option<&str>) -> SpotifyResult {
        let mut pl = match self.choose_user_playlist(arg)? {
            None => {
                println!("cancelled");
                return Ok(());
            }
            Some(p) => p,
        };

        let id = match track.id.as_ref() {
            Some(i) => i.clone(),
            None => track.uri.clone(),
        };

        // do not add duplicate songs
        let was_simple = pl.is_simple();
        if was_simple {
            pl.make_full(&self.client, &self.user)?;
        }

        let mut dupe = false;
        let pl = if let Playlist::Full(mut p) = pl {
            if !p.tracks.items.iter().any(|playlist_track| {
                playlist_track
                    .track
                    .as_ref()
                    .map(|t| t.id.eq(&track.id) || t.uri.eq(&track.uri))
                    .unwrap_or(false)
            }) {
                // no dupes, add the track
                p.tracks.items.insert(
                    0,
                    PlaylistTrack {
                        added_at: Utc::now(),
                        added_by: None,
                        is_local: false,
                        track: Some(track),
                    },
                );
            } else {
                println!("the track is already in the playlist, no action taken");
                dupe = true;
            }
            Playlist::from(p)
        } else {
            pl
        };
        if dupe {
            if was_simple && !pl.is_simple() {
                if let Some(v) = self.pl_cache.as_mut() {
                    for p in v {
                        if p.id() == pl.id() {
                            *p = pl;
                            break;
                        }
                    }
                }
            }
            return Ok(());
        }

        self.client
            .user_playlist_add_tracks(&self.user, pl.id(), &[id], Some(0))
            .map(|_| {
                println!("saved to {}", pl.name());
                if let Some(v) = self.pl_cache.as_mut() {
                    for p in v {
                        if p.id() == pl.id() {
                            *p = pl;
                            break;
                        }
                    }
                }
            })
    }

    fn queue(&self, uri: String) -> SpotifyResult {
        self.client.add_item_to_queue(uri, self.device.clone())
    }

    fn like_track(&self, t: &FullTrack) -> SpotifyResult {
        let id = match t.id.as_ref() {
            Some(i) => i.clone(),
            None => t.uri.clone(),
        };
        if let Ok(Some(true)) = self
            .client
            .current_user_saved_tracks_contains(&[id.clone()])
            .map(|v| v.get(0).copied())
        {
            println!("{} is already in your favourites folder", &t.name);
            Ok(())
        } else {
            self.client.current_user_saved_tracks_add(&[id]).map(|_| {
                println!("added {} to your favourites folder", &t.name);
            })
        }
    }

    fn follow_artist(&self, art: &FullArtist) -> SpotifyResult {
        let id = art.id.clone();

        if let Ok(Some(true)) = self
            .client
            .user_artist_check_follow(&[id.clone()])
            .map(|v| v.get(0).copied())
        {
            println!("already following {}", &art.name);
            Ok(())
        } else {
            self.client.user_follow_artists(&[id]).map(|_| {
                println!("followed {}", &art.name);
            })
        }
    }

    fn save_album(&self, alb: &SimplifiedAlbum) -> SpotifyResult {
        let id = match alb.id.as_ref() {
            Some(i) => i.clone(),
            None => {
                println!("the album is missing an ID, can't save to the library");
                return Ok(());
            }
        };

        if let Ok(Some(true)) = self
            .client
            .current_user_saved_albums_contains(&[id.clone()])
            .map(|v| v.get(0).copied())
        {
            println!("{} is already in your library", &alb.name);
            Ok(())
        } else {
            self.client.current_user_saved_albums_add(&[id]).map(|_| {
                println!("saved {} to your library", &alb.name);
            })
        }
    }

    fn follow_playlist(&self, pl: &Playlist) -> SpotifyResult {
        let owner_id = &pl.owner().id;
        let pl_id = pl.id();
        if let Ok(Some(true)) = self
            .client
            .user_playlist_check_follow(owner_id, pl_id, &[self.user.clone()])
            .map(|v| v.get(0).copied())
        {
            println!("already following {}", pl.name());
            Ok(())
        } else {
            self.client
                .user_playlist_follow_playlist(owner_id, pl_id, None)
                .map(|_| {
                    println!("followed {}", pl.name());
                })
        }
    }
}
