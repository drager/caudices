use quicksilver::{graphics::Color, lifecycle::Asset, load_file};
use serde::de::{self, Deserialize, Deserializer, Unexpected};

pub fn de_color<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    let color_string = String::deserialize(deserializer)?.to_lowercase();

    match color_string.as_ref() {
        "red" => Ok(Color::RED),
        "green" => Ok(Color::GREEN),
        "blue" => Ok(Color::BLUE),
        "orange" => Ok(Color::ORANGE),
        "purple" => Ok(Color::PURPLE),
        "indigo" => Ok(Color::INDIGO),
        _ => Err(de::Error::invalid_value(
            Unexpected::Str("Color not found"),
            &"0",
        )),
    }
}

pub fn load_json_from_file(filename: &'static str) -> Asset<Vec<u8>> {
    Asset::new(load_file(filename))
}
