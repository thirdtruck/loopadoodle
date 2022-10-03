use dropbox_sdk::{files, UserAuthClient};
use dropbox_sdk::default_client::UserAuthDefaultClient;
use regex::Regex;
use std::collections::VecDeque;

#[derive(Responder)]
#[response(status = 200)]
pub struct MusicFile(pub Vec<u8>);

pub fn fetch_music_files(folder_path: &str) -> Vec<MusicFile> {
    let auth = dropbox_sdk::oauth2::get_auth_from_env_or_prompt();
    let client = UserAuthDefaultClient::new(auth);

    let music_files = list_music_files(&client, folder_path);

    let mut downloaded_files = vec![];

    for file in music_files {
        match file {
            files::Metadata::File(entry) => {
                let raw = download_music_file(&client, &entry);
                downloaded_files.push(MusicFile(raw));
            }
            _ => {
                println!("Unexpected metadata: {:?}", file);
            }
        }
    }

    downloaded_files
}

fn list_music_files<'a, T: UserAuthClient>(client: &'a T, folder_path: &str) -> Vec<dropbox_sdk::files::Metadata> {
    let mp3_regex = Regex::new(r".*.mp3").unwrap();

    let listed_files = files::list_folder(
        client,
        &files::ListFolderArg::new(folder_path.into()).with_recursive(true),
    );

    let result = listed_files.unwrap().unwrap();
    let cursor = if result.has_more {
        Some(result.cursor)
    } else {
        None
    };

    let buffer: VecDeque<dropbox_sdk::files::Metadata> = result.entries.into();

    let directory = DirectoryIterator {
        client,
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
                    mp3_regex.is_match(&filename)
                },
                _ => false,
            }
        })
        .map(|file| file.clone())
        .collect()
}

fn download_music_file<'a, T: UserAuthClient>(client: &'a T, metadata: &files::FileMetadata) -> Vec<u8> {
    let metadata = metadata.clone();
    let filename = metadata.name;
    let filepath = metadata.path_display.unwrap_or(filename);

    println!("Downloading: {}", filepath);

    let download_arg = files::DownloadArg::new(filepath.to_string());
    let result = files::download(client, &download_arg, None, Some(metadata.size));
    let result = result.unwrap().unwrap();
    println!("metadata.size({}), result.content_length({:?})", metadata.size, result.content_length);
    let mut body = result.body.unwrap();

    let mut buffer = Vec::new();
    body.read_to_end(&mut buffer).unwrap();

    buffer
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
