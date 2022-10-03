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
