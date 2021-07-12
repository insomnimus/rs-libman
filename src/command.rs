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
    PlayUserPlaylist,
    SetDevice,
    Show,
}
