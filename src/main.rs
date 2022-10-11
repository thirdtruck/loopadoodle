const BUILD_NUMBER: usize = 5;
const FOLDER_PATH: &str = "/Looptober/2022/";

mod dropbox;

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_static_resources;

use std::sync::{Arc, RwLock};

use rocket::State;
use rocket::http::Header;
use rocket_include_static_resources::{EtagIfNoneMatch, StaticContextManager, StaticResponse};

use dropbox::{MusicFile, MusicAlbum};

type EditableMusicAlbum = Arc<RwLock<MusicAlbum>>;

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
fn refresh_music(state: &State<EditableMusicAlbum>) -> RefreshOutcome {
    let username = "jaycie".to_string();

    let lock = Arc::clone(state.inner());
    let locked_album = lock.write();

    if let Ok(mut album) = locked_album {
        let (new_album, _) = dropbox::fetch_music_files(FOLDER_PATH);
        *album = new_album;
        RefreshOutcome::Success(format!("Successfully refreshed music files for {}.", username))
    } else {
        RefreshOutcome::Error("Unable to update album".to_string())
    }
}

#[get("/~jaycie/<year>/<month>/<day>")]
fn music_for_user(state: &State<EditableMusicAlbum>, year: usize, month: usize, day: usize) -> Option<MusicFileResponse> {
    let filename = format!("looptober-jaycie-{:04}-{:02}-{:02}.mp3", year, month, day);

    let lock = Arc::clone(state.inner());
    let locked_album = lock.read();

    if let Ok(album) = locked_album {
        if let Some(music_file) = album.tracks.get(&filename) {
            Some(MusicFileResponse::new(&music_file))
        } else {
            None
        }
    } else {
        None
    }
}

#[get("/~jaycie/<year>/<month>/<day>/<format>")]
fn music_for_user_by_format(state: &State<EditableMusicAlbum>, year: usize, month: usize, day: usize, format: String) -> Option<MusicFileResponse> {
    let filename = format!("looptober-jaycie-{:04}-{:02}-{:02}.{}", year, month, day, format);

    let lock = Arc::clone(state.inner());
    let locked_album = lock.read();

    if let Ok(album) = locked_album {
        if let Some(music_file) = album.tracks.get(&filename) {
            Some(MusicFileResponse::new(&music_file))
        } else {
            None
        }
    } else {
        None
    }
}

#[launch]
fn rocket() -> _ {
    println!("Initializing {} build number {}", env!("CARGO_PKG_NAME"), BUILD_NUMBER);

    let (music_album, downloaded_files) = dropbox::fetch_music_files(FOLDER_PATH);
    let editable_album = Arc::new(RwLock::new(music_album));

    rocket::build()
        .attach(static_resources_initializer!(
            "looptober-jaycie-2022-10-01" => "music/looptober-2022-10-01.mp3",
            "readme" => "README.md",
        ))
        .mount("/", routes![
               // oembed,
               music_for_user,
               music_for_user_by_format,
               refresh_music,
               looptober_jaycie_2022_10_01,
        ])
        .manage(downloaded_files)
        .manage(editable_album)
}
