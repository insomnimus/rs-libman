#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cmd {
    // search commands
    Search,
    SearchTrack,
    SearchArtist,
    SearchAlbum,
    SearchPlaylist,

    // play-first commands (similar to "i'm feeling lucky")
    PlayFirstTrack,
    PlayFirstAlbum,
    PlayFirstPlaylist,
    PlayFirstArtist,

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
    Help,
    PlayUserPlaylist,
    SetDevice,
    Show,
    Prompt,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrackCmd {
    Play,
    Queue,
    Save,
    Like,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlbumCmd {
    Play,
    Queue,
    Save,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArtistCmd {
    Play,
    Queue,
    Follow,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaylistCmd {
    Play,
    Queue,
    Follow,
    Help,
}
