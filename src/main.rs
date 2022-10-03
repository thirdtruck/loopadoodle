#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_static_resources;

use rocket::State;
use rocket_include_static_resources::{EtagIfNoneMatch, StaticContextManager, StaticResponse};

mod dropbox;

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

/*
#[get("/oembed/~/<username>/<filename>")]
fn oembed(username: &str, filename: &str) -> Json<OEmbed> {
    Json(OEmbed::new())
}
*/

#[get("/from_dropbox.mp3")]
fn from_dropbox(state: &State<Option<MusicFile>>) -> MusicFile {
    if let Some(raw) = state.inner() {
        let raw = raw.0.clone();
        MusicFile(raw)
    } else {
        MusicFile(vec![])
    }
}

#[derive(Responder)]
#[response(status = 200)]
pub struct MusicFile(Vec<u8>);

#[launch]
fn rocket() -> _ {
    let folder_path = "/Looptober/2022/".to_string();

    let downloaded_file = dropbox::fetch_music_files(&folder_path);

    rocket::build()
        .attach(static_resources_initializer!(
            "looptober-jaycie-2022-10-01" => "music/looptober-2022-10-01.mp3",
            "readme" => "README.md",
        ))
        .mount("/", routes![
               // oembed,
               from_dropbox,
               looptober_jaycie_2022_10_01,
        ])
        .manage(downloaded_file)
}
