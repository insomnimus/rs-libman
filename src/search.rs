pub fn search_tracks(client: &Spotify, query: &str) -> Result<Vec<FullTrack>> {
	let page = client.search(query,
	&SearchType::Track,
	20,
	0,
	None,
	None)?;
	Ok(if let SearchResult::Tracks(p) = page{
		p.items
	}else{
		vec![]
	})
}

pub fn search_artists(client: &Spotify, query: &str) -> Result<Vec<FullArtist>> {
	let page = client.search(
	query,
	&SearchType::Artist,
	20,
	0,
	None,
	None,
	)?;
	
	Ok(if let SearchResult::Artists(p) = page{
		p.items
	}else{
		vec![]
	})
}

pub fn search_albums(client: &Spotify, query: &str) -> Result<Vec<SimplifiedAlbum>> {
	let page = client.search(
	query,
	&SearchType::Album,
	20,
	0,
	None,
	None,
	)?;
	
	Ok(if let SearchResult::Albums(p) = page{
		p.items
	}else{
		vec![]
	})
}

pub fn search_playlists(client: &Spotify, query: &str) -> Result<Vec<SimplifiedPlaylist>> {
	let page = client.search(
	query,
	&SearchType::Playlist,
	20,
	0,
	None,
	None,
	)?;
	
	Ok(if let SearchResult::Playlists(p) = page{
		p.items
	}else{
		vec![]
	})
}
