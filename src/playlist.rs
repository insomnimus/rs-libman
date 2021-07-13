use rspotify::blocking::client::Spotify;
use rspotify::model::{
    playlist::{FullPlaylist, SimplifiedPlaylist},
    user::PublicUser,
};

#[derive(Debug, Clone)]
pub enum Playlist {
    Simple(SimplifiedPlaylist),
    Full(FullPlaylist),
}

impl From<FullPlaylist> for Playlist {
    fn from(p: FullPlaylist) -> Self {
        Self::Full(p)
    }
}

impl From<SimplifiedPlaylist> for Playlist {
    fn from(p: SimplifiedPlaylist) -> Self {
        Self::Simple(p)
    }
}

impl Playlist {
    pub fn name_eq(&self, s: &str) -> bool {
        match self {
            Self::Simple(p) => crate::equalfold(&p.name, s),
            Self::Full(p) => crate::equalfold(&p.name, s),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Simple(p) => &p.name[..],
            Self::Full(p) => &p.name[..],
        }
    }

    pub fn uri(&self) -> &str {
        match self {
            Self::Simple(p) => &p.uri[..],
            Self::Full(p) => &p.uri[..],
        }
    }

    pub fn id(&self) -> &str {
        match self {
            Self::Simple(p) => &p.id[..],
            Self::Full(p) => &p.id[..],
        }
    }

    pub fn owner(&self) -> &PublicUser {
        match self {
            Self::Simple(p) => &p.owner,
            Self::Full(p) => &p.owner,
        }
    }

    pub fn set_name(&mut self, name: &str) {
        match self {
            Self::Simple(p) => {
                p.name.clear();
                p.name.push_str(name);
            }
            Self::Full(p) => {
                p.name.clear();
                p.name.push_str(name);
            }
        }
    }

    pub fn make_full(&mut self, client: &Spotify, user_id: &str) -> crate::SpotifyResult {
        if let Self::Simple(p) = self {
            let mut id = p.id.to_string();
            *self = client
                .user_playlist(user_id, Some(id.as_mut_str()), None, None)?
                .into();
        }
        Ok(())
    }

    pub fn is_simple(&self) -> bool {
        matches!(self, Self::Simple(_))
    }
}
