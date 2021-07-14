mod album_cmd;
mod artist_cmd;
mod playlist_cmd;
mod track_cmd;

pub use super::Controller;
use crate::{
    command::{AlbumCmd, ArtistCmd, Cmd, PlaylistCmd, TrackCmd},
    playlist::Playlist,
    read_number, search, SpotifyResult,
};
use rspotify::model::{album::SimplifiedAlbum, artist::FullArtist, track::FullTrack};

enum SearchResult {
    Track(FullTrack),
    Album(SimplifiedAlbum),
    Artist(FullArtist),
    Playlist(Playlist),
}

impl SearchResult {
    fn println(&self, no: usize) {
        match self {
            Self::Track(t) => {
                println!(
                    "#{no:2}: {kind:8} | {name} by {artists}",
                    no = no,
                    kind = "track",
                    name = &t.name,
                    artists = crate::join_artists(&t.artists)
                );
            }
            Self::Album(a) => {
                println!(
                    "#{no:2}: {kind:8} | {name} by {artists}",
                    no = no,
                    kind = "album",
                    name = &a.name,
                    artists = crate::join_artists(&a.artists)
                );
            }
            Self::Artist(a) => {
                println!(
                    "#{no:2}: {kind:8} | {name}",
                    no = no,
                    kind = "artist",
                    name = &a.name
                );
            }
            Self::Playlist(p) => {
                println!(
                    "#{no:2}: {kind:8} | {name} from {owner}",
                    no = no,
                    kind = "playlist",
                    name = p.name(),
                    owner = p
                        .owner()
                        .display_name
                        .as_ref()
                        .map(|s| &s[..])
                        .unwrap_or("unknown")
                );
            }
        };
    }
}

impl Controller {
    pub fn search_track(&mut self, arg: Option<&str>) -> SpotifyResult {
        let query = match arg {
            Some(a) => search::track_query(a),
            None => {
                self.show_usage(Cmd::SearchTrack);
                return Ok(());
            }
        };

        let tracks = search::tracks(&self.client, &query, 20)?;

        if tracks.is_empty() {
            println!("no result for {}", &query);
            return Ok(());
        }

        self.track_shell(tracks)
    }

    pub fn search_artist(&mut self, arg: Option<&str>) -> SpotifyResult {
        let query = match arg {
            Some(a) => a,
            None => {
                self.show_usage(Cmd::SearchArtist);
                return Ok(());
            }
        };

        let artists = search::artists(&self.client, query, 20)?;

        if artists.is_empty() {
            println!("no result for {}", query);
            return Ok(());
        }

        self.artist_shell(artists)
    }

    pub fn search_album(&mut self, arg: Option<&str>) -> SpotifyResult {
        let query = match arg {
            Some(a) => search::album_query(a),
            None => {
                self.show_usage(Cmd::SearchAlbum);
                return Ok(());
            }
        };

        let albums = search::albums(&self.client, &query, 20)?;

        if albums.is_empty() {
            println!("no result for {}", &query);
            return Ok(());
        }

        self.album_shell(albums)
    }

    pub fn search_playlist(&mut self, arg: Option<&str>) -> SpotifyResult {
        let query = match arg {
            Some(a) => a,
            None => {
                self.show_usage(Cmd::SearchAlbum);
                return Ok(());
            }
        };

        let pls = search::playlists(&self.client, query, 20)?;

        if pls.is_empty() {
            println!("no result for {}", query);
            return Ok(());
        }

        self.playlist_shell(pls)
    }

    pub fn search(&mut self, arg: Option<&str>) -> SpotifyResult {
        let query = match arg {
            Some(a) => a,
            None => {
                self.show_usage(Cmd::Search);
                return Ok(());
            }
        };

        let tracks = search::tracks(&self.client, query, 5)?;
        let artists = search::artists(&self.client, query, 5)?;
        let albums = search::albums(&self.client, query, 5)?;
        let pls = search::playlists(&self.client, query, 5)?;

        let mut results: Vec<SearchResult> = vec![];
        results.extend(tracks.into_iter().map(SearchResult::Track));
        results.extend(artists.into_iter().map(SearchResult::Artist));
        results.extend(albums.into_iter().map(SearchResult::Album));
        results.extend(pls.into_iter().map(SearchResult::Playlist));

        if results.is_empty() {
            println!("no result for {}", query);
            return Ok(());
        }

        for (i, r) in results.iter().enumerate() {
            r.println(i);
        }

        if let Some(n) = read_number(0, results.len()) {
            match results.get(n).unwrap() {
                SearchResult::Track(t) => self.play_track(t),
                SearchResult::Artist(a) => self.play_artist(a),
                SearchResult::Album(a) => self.play_album(a),
                SearchResult::Playlist(p) => self.play_playlist(p),
            }
        } else {
            println!("cancelled");
            Ok(())
        }
    }

    pub fn track_shell(&mut self, tracks: Vec<FullTrack>) -> SpotifyResult {
        // show the tracks
        for (i, t) in tracks.iter().enumerate() {
            println!(
                "#{no:2} | {name} by {artist}",
                no = i,
                name = &t.name,
                artist = crate::join_artists(&t.artists)
            );
        }

        println!("type help for a list of available actions");
        loop {
            let input = crate::prompt("command:");
            if input.is_empty() {
                println!("cancelled");
                return Ok(());
            }
            let (cmd, arg) = crate::split_command(&input);
            if arg.is_none() && crate::is_digits(cmd) {
                let n = cmd.parse::<usize>().unwrap();
                if n < tracks.len() {
                    return self.play_track(&tracks[n]);
                } else {
                    println!("please enter a number between 0 and {}", tracks.len());
                }
            } else {
                let c = match self.track_handlers.iter().find(|h| h.is_match(cmd)) {
                    Some(h) => h.cmd,
                    None => {
                        println!("{} is not a known command\ntype `help` for a list of available actions", cmd);
                        continue;
                    }
                };

                let should_return = match c {
                    TrackCmd::Play => self.track_cmd_play(&tracks, arg)?,
                    TrackCmd::Help => {
                        self.track_cmd_help(arg);
                        false
                    }
                    TrackCmd::Queue => self.track_cmd_queue(&tracks, arg)?,
                    TrackCmd::Save => self.track_cmd_save(&tracks, arg)?,
                    TrackCmd::Like => self.track_cmd_like(&tracks, arg)?,
                };

                if should_return {
                    return Ok(());
                }
            }
        }
    }

    fn artist_shell(&mut self, artists: Vec<FullArtist>) -> SpotifyResult {
        // show artists
        for (i, a) in artists.iter().enumerate() {
            println!("#{no:2} | {name}", no = i, name = &a.name);
        }

        println!("type help for a list of available actions");
        loop {
            let input = crate::prompt("command:");
            if input.is_empty() {
                println!("cancelled");
                return Ok(());
            }
            let (cmd, arg) = crate::split_command(&input);
            if arg.is_none() && crate::is_digits(cmd) {
                let n = cmd.parse::<usize>().unwrap();
                if n < artists.len() {
                    return self.play_artist(&artists[n]);
                } else {
                    println!("please enter a number between 0 and {}", artists.len());
                }
            } else {
                let c = match self.artist_handlers.iter().find(|h| h.is_match(cmd)) {
                    Some(h) => h.cmd,
                    None => {
                        println!("{} is not a known command\ntype `help` for a list of available actions", cmd);
                        continue;
                    }
                };

                let should_return = match c {
                    ArtistCmd::Play => self.artist_cmd_play(&artists, arg)?,
                    ArtistCmd::Help => {
                        self.artist_cmd_help(arg);
                        false
                    }
                    ArtistCmd::Follow => self.artist_cmd_follow(&artists, arg)?,
                };

                if should_return {
                    return Ok(());
                }
            }
        }
    }

    fn album_shell(&mut self, albums: Vec<SimplifiedAlbum>) -> SpotifyResult {
        // show albums
        for (i, a) in albums.iter().enumerate() {
            println!(
                "#{no:2} | {name} by {artist}",
                no = i,
                name = &a.name,
                artist = crate::join_artists(&a.artists)
            );
        }

        println!("type help for a list of available actions");
        loop {
            let input = crate::prompt("command:");
            if input.is_empty() {
                println!("cancelled");
                return Ok(());
            }
            let (cmd, arg) = crate::split_command(&input);
            if arg.is_none() && crate::is_digits(cmd) {
                let n = cmd.parse::<usize>().unwrap();
                if n < albums.len() {
                    return self.play_album(&albums[n]);
                } else {
                    println!("please enter a number between 0 and {}", albums.len());
                }
            } else {
                let c = match self.album_handlers.iter().find(|h| h.is_match(cmd)) {
                    Some(h) => h.cmd,
                    None => {
                        println!("{} is not a known command\ntype `help` for a list of available actions", cmd);
                        continue;
                    }
                };

                let should_return = match c {
                    AlbumCmd::Play => self.album_cmd_play(&albums, arg)?,
                    AlbumCmd::Help => {
                        self.album_cmd_help(arg);
                        false
                    }
                    AlbumCmd::Queue => self.album_cmd_queue(&albums, arg)?,
                    AlbumCmd::Save => self.album_cmd_save(&albums, arg)?,
                };

                if should_return {
                    return Ok(());
                }
            }
        }
    }

    fn playlist_shell(&mut self, pls: Vec<Playlist>) -> SpotifyResult {
        // show playlistts
        for (i, p) in pls.iter().enumerate() {
            println!(
                "#{no:2} | {name} from {owner}",
                no = i,
                name = p.name(),
                owner = p
                    .owner()
                    .display_name
                    .as_ref()
                    .map(|s| &s[..])
                    .unwrap_or("unknown")
            );
        }

        println!("type help for a list of available actions");
        loop {
            let input = crate::prompt("command:");
            if input.is_empty() {
                println!("cancelled");
                return Ok(());
            }
            let (cmd, arg) = crate::split_command(&input);
            if arg.is_none() && crate::is_digits(cmd) {
                let n = cmd.parse::<usize>().unwrap();
                if n < pls.len() {
                    return self.play_playlist(&pls[n]);
                } else {
                    println!("please enter a number between 0 and {}", pls.len());
                }
            } else {
                let c = match self.playlist_handlers.iter().find(|h| h.is_match(cmd)) {
                    Some(h) => h.cmd,
                    None => {
                        println!("{} is not a known command\ntype `help` for a list of available actions", cmd);
                        continue;
                    }
                };

                let should_return = match c {
                    PlaylistCmd::Play => self.playlist_cmd_play(&pls, arg)?,
                    PlaylistCmd::Help => {
                        self.playlist_cmd_help(arg);
                        false
                    }
                    PlaylistCmd::Follow => self.playlist_cmd_follow(&pls, arg)?,
                };

                if should_return {
                    return Ok(());
                }
            }
        }
    }
}
