use super::Controller;
use crate::{command::Cmd, read_number, search, SpotifyResult};
use rspotify::model::{
    album::SimplifiedAlbum, artist::FullArtist, playlist::SimplifiedPlaylist, track::FullTrack,
};

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

        for (i, t) in tracks.iter().enumerate() {
            println!(
                "#{no:2} | {name} by {artist}",
                no = i,
                name = &t.name,
                artist = crate::join_artists(&t.artists)
            );
        }

        if let Some(n) = read_number(0, tracks.len()) {
            self.play_track(&tracks[n])
        } else {
            println!("cancelled");
            Ok(())
        }
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

        for (i, a) in artists.iter().enumerate() {
            println!("#{no:2} | {name}", no = i, name = &a.name);
        }

        if let Some(n) = read_number(0, artists.len()) {
            self.play_artist(&artists[n])
        } else {
            println!("cancelled");
            Ok(())
        }
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

        for (i, a) in albums.iter().enumerate() {
            println!(
                "#{no:2} | {name} by {artist}",
                no = i,
                name = &a.name,
                artist = crate::join_artists(&a.artists)
            );
        }

        if let Some(n) = read_number(0, albums.len()) {
            self.play_album(&albums[n])
        } else {
            println!("cancelled");
            Ok(())
        }
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

        for (i, p) in pls.iter().enumerate() {
            println!(
                "#{no:2} | {name} from {owner}",
                no = i,
                name = &p.name,
                owner = &p
                    .owner
                    .display_name
                    .as_ref()
                    .map(|s| &s[..])
                    .unwrap_or("unknown")
            );
        }

        if let Some(n) = read_number(0, pls.len()) {
            self.play_playlist(&pls[n])
        } else {
            println!("cancelled");
            Ok(())
        }
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
}

enum SearchResult {
    Track(FullTrack),
    Album(SimplifiedAlbum),
    Artist(FullArtist),
    Playlist(SimplifiedPlaylist),
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
                    name = &p.name,
                    owner = p
                        .owner
                        .display_name
                        .as_ref()
                        .map(|s| &s[..])
                        .unwrap_or("unknown")
                );
            }
        };
    }
}
