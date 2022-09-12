use geometria_derive::Deserialize;

use super::{
    comment::Comment, deserialize::Deserialize, deserializer::Deserializer, header::Header,
    properties::Properties, start_section::StartSection, version::Version,
};

#[derive(Deserialize)]
pub struct Archive {
    pub header: Header,
    pub version: Version,
    pub comment: Comment,
    pub start_section: StartSection,
    pub properties: Properties,
}
