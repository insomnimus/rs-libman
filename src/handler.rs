use crate::command::Cmd;
use std::borrow::Cow;

pub struct Handler {
    pub name: Cow<'static, str>,
    pub cmd: Cmd,
    pub description: Cow<'static, str>,
    pub help: Cow<'static, str>,
    pub usage: Cow<'static, str>,
    pub aliases: Vec<Cow<'static, str>>,
}

impl Handler {
    pub fn is_match(&self, s: &str) -> bool {
        crate::equalfold(&self.name, s) || self.aliases.iter().any(|a| crate::equalfold(a, s))
    }

    pub fn show_help(&self) {
        if self.aliases.is_empty() {
            println!(
                "# {name}
usage:
  {usage}

{help}",
                name = &self.name,
                usage = &self.usage,
                help = &self.help
            );
        } else {
            println!(
                "# {name}
aliases: {aliases}
usage:
  {usage}

{help}",
                name = &self.name,
                aliases = self.aliases.join(", "),
                usage = &self.usage,
                help = &self.help
            );
        }
    }

    pub fn show_usage(&self) {
        println!("usage:\n  {}", &self.usage);
    }

    pub fn show_short_help(&self) {
        if self.aliases.is_empty() {
            println!("{cmd}", cmd = &self.name);
        } else {
            println!(
                "{cmd} [aliases: {aliases}]",
                cmd = &self.name,
                aliases = self.aliases.join(", ")
            );
        }
        println!("	{}", &self.description);
    }
}

pub fn default_handlers() -> Vec<Handler> {
    use Cmd::*;
    fn new(
        cmd: Cmd,
        name: &'static str,
        description: &'static str,
        usage: &'static str,
        aliases: &[&'static str],
        help: &'static str,
    ) -> Handler {
        Handler {
            name: name.into(),
            description: description.into(),
            help: help.into(),
            usage: usage.into(),
            aliases: aliases
                .iter()
                .map(|s| -> Cow<'static, str> { (*s).into() })
                .collect(),
            cmd,
        }
    }

    vec![
	// search commands
	new(
	Search,
	"search",
	"Search for tracks, artists, albums and playlists.",
	"search <keyword>",
	&["s"],
	"Searchs for the given keyword to be played.",
	),
	new(
	SearchTrack,
	"search-track",
	"Search for a track.",
	"search-track <track>",
	&["stra"],
	"Search for a track.\nThe search term can be in the form `track::artist` for a more precise search.
`track by artist` is also allowed.",
	),
	new(
	SearchAlbum,
	"search-album",
	"Search for an album.",
	"search-album <album>",
	&["salb"],
	"Search for an album.\nThe search term can be in the form `album::artist` or `album by artist` for more precse results.",
	),
	new(
	SearchArtist,
	"search-artist",
	"Search for an artist.",
	"search-artist <artist>",
	&["sart"],
	"Search for an artist.",
	),
	new(
	SearchPlaylist,
	"search-playlist",
	"Search for a playlist.",
	"search-playlist <playlist>",
	&["spla"],
	"Search for a public playlist.",
	),
	// play-first commands
	new(
	PlayFirstTrack,
	"play-track",
	"Search for a track and play the first result.",
	"play-track <track>",
	&["ptra"],
	"Search for a track and play the first result.\nThe search term can be in the form `track::artist` or `track by artist` for more precise results.",
	),
	new(
	PlayFirstAlbum,
	"play-album",
	"Search for an album and play the first result.",
	"play-album <album>",
	&["palb"],
	"Search for an album and play the first result.\nThe search term can be in the form `album::artist` or `album by artist` for more precise results.",
	),
	new(
	PlayFirstArtist,
	"play-artist",
	"Search for an artist and play the first result.",
	"play-artist <artist>",
	&["part"],
	"Search for an artist and play the first result.",
	),
	new(
	PlayFirstPlaylist,
	"play-playlist",
	"Search for a playlist and play the first result.",
	"play-playlist <playlist>",
	&["ppla"],
	"Search for a public playlist and play the first result.",
	),
	// player commands
	new(
	SetVolume,
	"volume",
	"Change the volume.",
	"volume <percentage>",
	&["vol"],
	"Change the volume. You can also use `+N` and `-N` to increase or decrease the volume.",
	),
	new(
	Shuffle,
	"shuffle",
	"Turn shuffle on or off.",
	"shuffle [on|off]",
	&["sh"],
	"Without any argument, toggles shuffle on or off otherwise ests shuffle n or off.",
	),
	new(
	Repeat,
	"repeat",
	"Changes the repeat state.",
	"repeat [off|track|context]",
	&["rep"],
	"Changes the repeat state.",
	),
	new(
	Next,
	"next",
	"Plays the next track.",
	"next",
	&[">"],
	"Plays the next track.",
	),
	new(
	Prev,
	"prev",
	"Plays the previous track.",
	"prev",
	&["<"],
	"Plays the previous track.",
	),
	// library commands
	new(
	SavePlaying,
	"save-playing",
	"Save the currently playing track to a playlist.",
	"save-playing [playlist]",
	&["save", "add"],
	"Save the currently playing track to one of your playlists.\nWithout any argument, prompts you for a playlist, else adds to the given playlist.",
	),
	new(
	RemovePlaying,
	"remove-playing",
	"Remove the currently playing track from a playlist.",
	"remove-playing [playlist]",
	&["rm"],
	"Remove the currently playing track from a playlist.\nIf no playlist name is given, the last played playlist will be assumed.",
	),
	new(
	CreatePlaylist,
	"create-playlist",
	"Create a new playlist.",
	"create-playlist [name]",
	&["create"],
	"Create a new playlist.\nYou will be prompted for the playlist details.",
	),
	new(
	EditPlaylist,
	"edit-playlist",
	"Edit a playlists details.",
	"edit-playlist [playlist]",
	&["edit"],
	"Edit one of your playlists' details.\nWithout a playlist name, you will be prompted to choose one.",
	),
	new(
	DeletePlaylist,
	"delete-playlist",
	"Delete a playlist.",
	"delete-playlist [playlist]",
	&["delete"],
	"Delete or unfollow one of your playlists.",
	),
	// misc commands
	new(
	Help,
	"help",
	"Display help about a topic.",
	"help [topic]",
	&[],
	"Without any argument, displays help topics.\nWith an argument, shows help about the given topic.",
	),
	new(
	PlayUserPlaylist,
	"play",
	"Play one of your playlists.",
	"play [playlist]",
	&["pl"],
	"Play one of your playlists"
	),
	new(
	SetDevice,
	"device",
	"Choose a playback device.",
	"device [name]",
	&["dev"],
	"Choose a playback device.",
	),
	new(
	Show,
	"show",
	"Display various items.",
	"show [library|playing|playlist]",
	&["sw"],
	"Show various items.\n
	lib|library: Show a list of your playlists.
	playing (or empty): Show currently playing track.
	playlist: Show one of your playlists by name.",
	),
	]
}
