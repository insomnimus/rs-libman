#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cmd {
	// search commands
	Search,
	SearchTrack,
	SearchArtist,
	SearchAlbum,
	SearchPlaylist,

	// play-first commands (similar to "i'm feeling lucky")
	PlayTrack,
	PlayAlbum,
	PlayPlaylist,
	PlayArtist,

	// player commands
	SetVolume,
	Shuffle,
	Repeat,
	Next,
	Prev,

	// library commands
	SavePlaying,
	RemovePlaying,
	CreatePlaylist,
	EditPlaylist,
	DeletePlaylist,

	// misc
	PlayUserPlaylist,
	SetDevice,
	Show,
}
