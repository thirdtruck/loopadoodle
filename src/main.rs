const BUILD_NUMBER: usize = 1;

mod dropbox;

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_static_resources;

use rocket::State;
use rocket::http::Header;
use rocket_include_static_resources::{EtagIfNoneMatch, StaticContextManager, StaticResponse};

use dropbox::MusicFile;

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

#[get("/~jaycie/<year>/<month>/<day>")]
fn music_for_user(state: &State<Vec<MusicFile>>, year: usize, month: usize, day: usize) -> MusicFile {
    println!("Loading music file for date: {}/{}/{}", year, month, day);

    let filename = format!("looptober-jaycie-{}-{}-{}.mp3", year, month, day);

    MusicFile::new(vec![], &filename)
}

#[get("/from_dropbox/<index>")]
fn from_dropbox(state: &State<Vec<MusicFile>>, index: usize) -> MusicFile {
    if let Some(raw) = state.get(index) {
        let raw = raw.body.clone();
        let filename = format!("looptober-{}.mp3", index);
        MusicFile::new(raw, &filename)
    } else {
        MusicFile::new(vec![], "no-such-file.mp3")
    }
}

#[launch]
fn rocket() -> _ {
    println!("Initializing {} build number {}", env!("CARGO_PKG_NAME"), BUILD_NUMBER);

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
