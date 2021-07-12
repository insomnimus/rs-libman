use rspotify::{
    blocking::client::Spotify,
    model::{
        album::SimplifiedAlbum, artist::FullArtist, playlist::SimplifiedPlaylist,
        search::SearchResult, track::FullTrack,
    },
    senum::SearchType,
};

type Result<T> = ::std::result::Result<T, failure::Error>;

pub fn tracks(client: &Spotify, query: &str, limit: u32) -> Result<Vec<FullTrack>> {
    let page = client.search(query, SearchType::Track, limit, 0, None, None)?;
    Ok(if let SearchResult::Tracks(p) = page {
        p.items
    } else {
        vec![]
    })
}

pub fn artists(client: &Spotify, query: &str, limit: u32) -> Result<Vec<FullArtist>> {
    let page = client.search(query, SearchType::Artist, limit, 0, None, None)?;

    Ok(if let SearchResult::Artists(p) = page {
        p.items
    } else {
        vec![]
    })
}

pub fn albums(client: &Spotify, query: &str, limit: u32) -> Result<Vec<SimplifiedAlbum>> {
    let page = client.search(query, SearchType::Album, limit, 0, None, None)?;

    Ok(if let SearchResult::Albums(p) = page {
        p.items
    } else {
        vec![]
    })
}

pub fn playlists(client: &Spotify, query: &str, limit: u32) -> Result<Vec<SimplifiedPlaylist>> {
    let page = client.search(query, SearchType::Playlist, limit, 0, None, None)?;

    Ok(if let SearchResult::Playlists(p) = page {
        p.items
    } else {
        vec![]
    })
}

pub fn track_query(s: &str) -> String {
    if s.contains("::") {
        let mut split = s.splitn(2, "::");
        let track = split.next().unwrap_or_default().trim();
        if let Some(art) = split.next() {
            format!("track:{} artist:{}", track, art.trim())
        } else {
            format!("track:{}", track)
        }
    } else if s.contains(" by ") {
        let mut split = s.splitn(2, " by ");
        let track = split.next().unwrap_or_default().trim();
        if let Some(art) = split.next() {
            format!("track:{} artist:{}", track, art.trim())
        } else {
            format!("track:{}", track)
        }
    } else {
        s.to_string()
    }
}

pub fn album_query(s: &str) -> String {
    if s.contains("::") {
        let mut split = s.splitn(2, "::");
        let album = split.next().unwrap_or_default().trim();
        if let Some(art) = split.next() {
            format!("album:{} artist:{}", album, art.trim())
        } else {
            format!("album:{}", album)
        }
    } else if s.contains(" by ") {
        let mut split = s.splitn(2, " by ");
        let album = split.next().unwrap_or_default().trim();
        if let Some(art) = split.next() {
            format!("album:{} artist:{}", album, art.trim())
        } else {
            format!("album:{}", album)
        }
    } else {
        s.to_string()
    }
}
