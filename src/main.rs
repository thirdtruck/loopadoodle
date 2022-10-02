#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_static_resources;

use rocket::State;
use rocket::serde::{Serialize, json::Json};

use rocket_include_static_resources::{EtagIfNoneMatch, StaticContextManager, StaticResponse};

use simple_xml_builder::XMLElement;

use dropbox_sdk::{files, UserAuthClient};
use dropbox_sdk::default_client::UserAuthDefaultClient;

use std::collections::VecDeque;
use std::fs::File;
use std::io::{Write};

use regex::Regex;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct OEmbed {
    #[serde(rename(serialize = "type"))] embed_type: String,
    title: String,
    html: String,
}

impl OEmbed {
    pub fn new() -> Self {
        let example_file = "https://music.looptober.com/~jaycie/looptober-jaycie-2022-10-01.mp3".to_string();

        let mut embed = XMLElement::new("embed");
        embed.add_attribute("src", example_file);
        embed.add_attribute("type", "audio/mpeg");

        let mut object = XMLElement::new("object");
        object.add_child(embed);

        let mut buf = Vec::new();

        object.write(&mut buf).unwrap();

        let html = std::str::from_utf8(&buf).unwrap().to_string();

        Self {
            embed_type: "audio".to_string(),
            title: "Example Embed!".to_string(),
            html,
        }
    }
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

static_response_handler! {
    "/~jaycie/looptober-jaycie-2022-10-01.mp3" => looptober_jaycie_2022_10_01 => "looptober-jaycie-2022-10-01",
}

#[get("/")]
fn index(
    static_resources: &State<StaticContextManager>,
    etag_if_none_match: EtagIfNoneMatch,
) -> StaticResponse {
    static_resources.build(&etag_if_none_match, "readme")
}

#[get("/oembed/~/<username>/<filename>")]
fn oembed(username: &str, filename: &str) -> Json<OEmbed> {
    Json(OEmbed::new())
}

fn download_music_file<'a, T: UserAuthClient>(client: &'a T, metadata: &files::FileMetadata) {
    let metadata = metadata.clone();
    let filename = metadata.name;
    let filepath = metadata.path_display.unwrap_or(filename);

    println!("Downloading: {}", filepath);

    let download_arg = files::DownloadArg::new(filepath.to_string());
    let result = files::download(client, &download_arg, None, Some(metadata.size));
    let result = result.unwrap().unwrap();
    let mut body = result.body.unwrap();

    let mut buffer = Vec::new();
    body.read_to_end(&mut buffer).unwrap();

    let path = "/tmp/latest-download.mp3";
    let mut file = File::create(path).unwrap();
    file.write_all(&buffer).unwrap();
}

#[launch]
fn rocket() -> _ {
    let auth = dropbox_sdk::oauth2::get_auth_from_env_or_prompt();
    let client = UserAuthDefaultClient::new(auth);

    let folder_path = "/Looptober/2022/".to_string();

    let music_files = list_music_files(&client, &folder_path);

    for file in music_files {
        match file {
            files::Metadata::File(entry) => {
                download_music_file(&client, &entry);
            }
            _ => {
                println!("Unexpected metadata: {:?}", file);
            }
        }
    }

    rocket::build()
        .attach(static_resources_initializer!(
            "looptober-jaycie-2022-10-01" => "music/looptober-2022-10-01.mp3",
            "readme" => "README.md",
        ))
        .mount("/", routes![oembed, looptober_jaycie_2022_10_01])
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
