use super::deserialize::Deserialize;
use super::deserializer::Deserializer;

pub struct Header;

const FILE_BEGIN: &[u8] = "3D Geometry File Format ".as_bytes();

impl Deserialize for Header {
    type Error = String;

    fn deserialize<D>(deserializer: &mut D) -> Result<Self, Self::Error>
    where
        D: Deserializer,
    {
        let mut buffer = [0; FILE_BEGIN.len()];
        match deserializer.deserialize_bytes(&mut buffer) {
            Ok(()) => match FILE_BEGIN == buffer {
                true => Ok(Header {}),
                false => Err("3dm file error: invalid file begin".to_string()),
            },
            Err(e) => Err(e),
        }
    }
}
