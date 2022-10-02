#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_static_resources;

use rocket::State;

use rocket_include_static_resources::{EtagIfNoneMatch, StaticContextManager, StaticResponse};

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

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(static_resources_initializer!(
            "looptober-jaycie-2022-10-01" => "music/looptober-2022-10-01.mp3",
            "readme" => "README.md",
        ))
        .mount("/", routes![looptober_jaycie_2022_10_01])
}
