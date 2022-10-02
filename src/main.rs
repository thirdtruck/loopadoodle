#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_static_resources;

use rocket::State;
use rocket::serde::{Serialize, json::Json};

use rocket_include_static_resources::{EtagIfNoneMatch, StaticContextManager, StaticResponse};

use simple_xml_builder::XMLElement;

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

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(static_resources_initializer!(
            "looptober-jaycie-2022-10-01" => "music/looptober-2022-10-01.mp3",
            "readme" => "README.md",
        ))
        .mount("/", routes![oembed, looptober_jaycie_2022_10_01])
}
