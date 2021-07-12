use super::Controller;
use crate::{command::Cmd, read_number, search, SpotifyResult};

impl Controller {
    pub fn search_track(&mut self, arg: Option<&str>) -> SpotifyResult {
        let query = match arg {
            Some(a) => search::track_query(a),
            None => {
                self.show_usage(Cmd::SearchTrack);
                return Ok(());
            }
        };

        let tracks = search::tracks(&self.client, &query)?;

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

        let artists = search::artists(&self.client, query)?;

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

        let albums = search::albums(&self.client, &query)?;

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

        let pls = search::playlists(&self.client, query)?;

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

    pub fn search(&mut self, _arg: Option<&str>) -> SpotifyResult {
        todo!()
    }
}
