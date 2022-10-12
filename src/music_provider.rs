use std::fmt;
use std::collections::HashMap;

pub trait MusicProvider {
    fn fetch_music_files(&self) -> (MusicAlbum, Vec<MusicFile>);
}

#[derive(Clone)]
pub struct MusicFile {
    pub filename: String,
    pub body: Vec<u8>,
}

impl MusicFile {
    pub fn new(body: Vec<u8>, filename: &str) -> Self {
        Self {
            body,
            filename: filename.to_string(),
        }
    }
}

impl fmt::Debug for MusicFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MusicFile")
            .field("filename", &self.filename)
            .field("body (size)", &self.body.len())
            .finish()
    }
}

#[derive(Debug)]
pub struct MusicAlbum {
    pub tracks: HashMap<String, MusicFile>,
}
