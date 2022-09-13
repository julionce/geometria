use geometria_derive::Deserialize;

use super::{
    bool::BoolFromI32,
    chunk,
    deserialize::Deserialize,
    deserializer::Deserializer,
    string::{StringWithLength, WStringWithLength},
    version::Version,
};

#[derive(Default, Deserialize)]
pub struct NotesV1 {
    pub visible: i32,
    pub window_left: i32,
    pub window_top: i32,
    pub window_right: i32,
    pub window_bottom: i32,
    #[underlying_type(StringWithLength)]
    pub data: String,
}

#[derive(Default, Deserialize)]
#[big_chunk_version(major == 1)]
pub struct NotesV2 {
    #[underlying_type(BoolFromI32)]
    pub html_encoded: bool,
    #[underlying_type(WStringWithLength)]
    pub data: String,
    #[underlying_type(BoolFromI32)]
    pub visible: bool,
    pub window_left: i32,
    pub window_top: i32,
    pub window_right: i32,
    pub window_bottom: i32,
}

pub enum Notes {
    V1(NotesV1),
    V2(NotesV2),
}

impl Default for Notes {
    fn default() -> Self {
        Self::V1(NotesV1::default())
    }
}

impl<D> Deserialize<'_, D> for Notes
where
    D: Deserializer,
{
    type Error = String;

    fn deserialize(deserializer: &mut D) -> Result<Self, Self::Error> {
        let notes;
        if Version::V1 == deserializer.version() {
            notes = Notes::V1(NotesV1::deserialize(deserializer)?);
        } else {
            notes = Notes::V2(NotesV2::deserialize(deserializer)?);
        }
        Ok(notes)
    }
}
