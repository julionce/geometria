use geometria_derive::RhinoDeserialize;

use super::{
    comment::Comment, deserialize::Deserialize, deserializer::Deserializer, header::Header,
    properties::Properties, settings::Settings, start_section::StartSection, version::Version,
};

#[derive(RhinoDeserialize)]
pub struct Archive {
    pub header: Header,
    pub version: Version,
    pub comment: Comment,
    pub start_section: StartSection,
    pub properties: Properties,
    pub settings: Settings,
}
