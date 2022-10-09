const BUILD_NUMBER: usize = 2;

mod dropbox;

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_static_resources;

use std::collections::HashMap;

use rocket::State;
use rocket_include_static_resources::{EtagIfNoneMatch, StaticContextManager, StaticResponse};

use dropbox::MusicFile;

static_response_handler! {
    "/~jaycie/looptober-jaycie-2022-10-01.mp3" => looptober_jaycie_2022_10_01 => "looptober-jaycie-2022-10-01",
}

#[allow(dead_code)]
#[get("/")]
fn index(
    static_resources: &State<StaticContextManager>,
    etag_if_none_match: EtagIfNoneMatch,
) -> StaticResponse {
    static_resources.build(&etag_if_none_match, "readme")
}

/*
#[get("/oembed/~/<username>/<filename>")]
fn oembed(username: &str, filename: &str) -> Json<OEmbed> {
    Json(OEmbed::new())
}
*/

#[get("/~jaycie/<year>/<month>/<day>")]
fn music_for_user(state: &State<HashMap<String, MusicFile>>, year: usize, month: usize, day: usize) -> Option<MusicFile> {
    let filename = format!("looptober-jaycie-{:04}-{:02}-{:02}.mp3", year, month, day);

    if let Some(music) = state.get(&filename) {
        Some(music.clone())
    } else {
        None
    }
}

#[launch]
fn rocket() -> _ {
    println!("Initializing {} build number {}", env!("CARGO_PKG_NAME"), BUILD_NUMBER);

    let folder_path = "/Looptober/2022/".to_string();

    let (files_by_name, downloaded_files) = dropbox::fetch_music_files(&folder_path);

    rocket::build()
        .attach(static_resources_initializer!(
            "looptober-jaycie-2022-10-01" => "music/looptober-2022-10-01.mp3",
            "readme" => "README.md",
        ))
        .mount("/", routes![
               // oembed,
               music_for_user,
               looptober_jaycie_2022_10_01,
        ])
        .manage(downloaded_files)
        .manage(files_by_name)
}
