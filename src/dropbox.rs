use crate::prelude::*;

use dropbox_sdk::{files, UserAuthClient};
use dropbox_sdk::default_client::UserAuthDefaultClient;
use regex::Regex;
use std::collections::VecDeque;
use std::collections::HashMap;

pub struct DropboxHosted {
    folder_path: String,
    client: UserAuthDefaultClient,
}

impl DropboxHosted {
    pub fn new(folder_path: &str) -> Self {
        let auth = dropbox_sdk::oauth2::get_auth_from_env_or_prompt();
        let client = UserAuthDefaultClient::new(auth);

        Self {
            folder_path: folder_path.to_string(),
            client,
        }
    }

    fn collect_music_files(&self) -> (MusicAlbum, Vec<MusicFile>) {
        let music_files = self.list_music_files();

        let mut downloaded_files = vec![];
        let mut files_by_name = HashMap::new();

        for file in music_files {
            match file {
                files::Metadata::File(entry) => {
                    let filename = entry.name.clone();
                    let raw = self.download_music_file(&entry);

                    let music_file = MusicFile::new(raw, &filename);

                    files_by_name.insert(filename, music_file.clone());
                    downloaded_files.push(music_file.clone());
                }
                _ => {
                    println!("Unexpected metadata: {:?}", file);
                }
            }
        }

        let album = MusicAlbum {
            tracks: files_by_name,
        };

        (album, downloaded_files)
    }

    fn list_music_files(&self) -> Vec<dropbox_sdk::files::Metadata> {
        let extension_regex = Regex::new(r".*\.(mp3|m4a|ogg)").unwrap();

        let folder_path = self.folder_path.clone();

        let listed_files = files::list_folder(
            &self.client,
            &files::ListFolderArg::new(folder_path).with_recursive(true),
        );

        let result = listed_files.unwrap().unwrap();
        let cursor = if result.has_more {
            Some(result.cursor)
        } else {
            None
        };

        let buffer: VecDeque<dropbox_sdk::files::Metadata> = result.entries.into();

        let directory = DirectoryIterator {
            client: &self.client,
            cursor,
            buffer,
        };

        let metadata: Vec<dropbox_sdk::files::Metadata> = directory.collect();

        metadata
            .iter()
            .filter(|file| {
                match file {
                    files::Metadata::File(entry) => {
                        let entry = entry.clone();
                        let filename = entry.path_display.unwrap_or(entry.name);
                        extension_regex.is_match(&filename)
                    },
                    _ => false,
                }
            })
            .map(|file| file.clone())
            .collect()
    }

    fn download_music_file(&self, metadata: &files::FileMetadata) -> Vec<u8> {
        let metadata = metadata.clone();
        let filename = metadata.name;
        let filepath = metadata.path_display.unwrap_or(filename);

        println!("Downloading: {}", filepath);

        let download_arg = files::DownloadArg::new(filepath.to_string());
        let result = files::download(&self.client, &download_arg, None, Some(metadata.size));
        let result = result.unwrap().unwrap();
        let mut body = result.body.unwrap();

        let mut buffer = Vec::new();
        body.read_to_end(&mut buffer).unwrap();

        buffer
    }
}

impl MusicProvider for DropboxHosted {
    fn fetch_music_files(&self) -> (MusicAlbum, Vec<MusicFile>) {
        self.collect_music_files()
    }
}

struct DirectoryIterator<'a, T: UserAuthClient> {
    client: &'a T,
    buffer: VecDeque<files::Metadata>,
    cursor: Option<String>,
}

impl<'a, T: UserAuthClient> Iterator for DirectoryIterator<'a, T> {
    type Item = files::Metadata;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.buffer.pop_front() {
            Some(entry)
        } else if let Some(cursor) = self.cursor.take() {
            let result = files::list_folder_continue(self.client, &files::ListFolderContinueArg::new(cursor)).unwrap().unwrap();
            self.buffer.extend(result.entries.into_iter());
            if result.has_more {
                self.cursor = Some(result.cursor);
            }
            self.buffer.pop_front().map(|entry| entry)
        } else {
            None
        }
    }
}
