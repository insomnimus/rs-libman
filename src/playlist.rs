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
}
