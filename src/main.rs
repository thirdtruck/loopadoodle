const BUILD_NUMBER: usize = 4;

mod dropbox;

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_static_resources;

use rocket::State;
use rocket::http::Header;
use rocket_include_static_resources::{EtagIfNoneMatch, StaticContextManager, StaticResponse};

use dropbox::{MusicFile, MusicAlbum};

static_response_handler! {
    "/~jaycie/looptober-jaycie-2022-10-01.mp3" => looptober_jaycie_2022_10_01 => "looptober-jaycie-2022-10-01",
}

#[derive(Clone, Debug, Responder)]
#[response(status = 200)]
pub struct MusicFileResponse {
    body: Vec<u8>,
    content_disposition: Header<'static>,
}

impl MusicFileResponse {
    pub fn new(music_file: &MusicFile) -> Self {
        let content_disposition = format!("inline; filename=\"{}\"", music_file.filename);

        Self {
            body: music_file.body.clone(),
            content_disposition: Header::new("Content-Disposition", content_disposition),
        }
    }
}

#[derive(Responder)]
enum RefreshOutcome {
    #[response(status = 200)]
    Success(String),
    #[response(status = 500)]
    Error(String),
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

#[post("/~jaycie/refresh-music")]
fn refresh_music(state: &State<MusicAlbum>) -> RefreshOutcome {
    let username = "jaycie".to_string();

    RefreshOutcome::Success(format!("Successfully refreshed music files for {}.", username))
}

#[get("/~jaycie/<year>/<month>/<day>")]
fn music_for_user(state: &State<MusicAlbum>, year: usize, month: usize, day: usize) -> Option<MusicFileResponse> {
    let filename = format!("looptober-jaycie-{:04}-{:02}-{:02}.mp3", year, month, day);

    if let Some(music_file) = state.tracks.get(&filename) {
        Some(MusicFileResponse::new(&music_file))
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
               refresh_music,
               looptober_jaycie_2022_10_01,
        ])
        .manage(downloaded_files)
        .manage(files_by_name)
}
