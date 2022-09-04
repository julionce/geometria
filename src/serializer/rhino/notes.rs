use super::{
    chunk,
    deserialize::Deserialize,
    deserializer::Deserializer,
    string::{StringWithLength, WStringWithLength},
    version::Version,
};

#[derive(Default)]
pub struct Notes {
    data: String,
    visible: bool,
    html_encoded: bool,
    window_left: i32,
    window_top: i32,
    window_right: i32,
    window_bottom: i32,
}

impl<D> Deserialize<'_, D> for Notes
where
    D: Deserializer,
{
    type Error = String;

    fn deserialize(deserializer: &mut D) -> Result<Self, Self::Error> {
        let mut notes = Notes::default();
        if Version::V1 == deserializer.version() {
            notes.visible = i32::deserialize(deserializer)? != 0i32;
            notes.window_left = i32::deserialize(deserializer)?;
            notes.window_top = i32::deserialize(deserializer)?;
            notes.window_right = i32::deserialize(deserializer)?;
            notes.window_bottom = i32::deserialize(deserializer)?;
            notes.data = StringWithLength::deserialize(deserializer)?.into();
        } else {
            let chunk_version = chunk::Version::deserialize(deserializer)?;
            if 1u8 == chunk_version.major() {
                notes.html_encoded = i32::deserialize(deserializer)? != 0i32;
                notes.data = WStringWithLength::deserialize(deserializer)?.into();
                notes.visible = i32::deserialize(deserializer)? != 0i32;
                notes.window_left = i32::deserialize(deserializer)?;
                notes.window_top = i32::deserialize(deserializer)?;
                notes.window_right = i32::deserialize(deserializer)?;
                notes.window_bottom = i32::deserialize(deserializer)?;
            }
        }
        Ok(notes)
    }
}
