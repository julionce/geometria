use super::deserialize::Deserialize;
use super::deserializer::Deserializer;

pub struct Header;

const FILE_BEGIN: &[u8] = "3D Geometry File Format ".as_bytes();

impl<D> Deserialize<'_, D> for Header
where
    D: Deserializer,
{
    type Error = String;

    fn deserialize(deserializer: &mut D) -> Result<Self, Self::Error> {
        let mut buffer = [0; FILE_BEGIN.len()];
        match deserializer.read_exact(&mut buffer) {
            Ok(()) => match FILE_BEGIN == buffer {
                true => Ok(Header {}),
                false => Err("invalid header".to_string()),
            },
            Err(e) => Err(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::rhino::{chunk, reader::Reader, version::Version};

    use super::*;

    #[test]
    fn deserialize_ok() {
        let data = "3D Geometry File Format ".as_bytes();
        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: Version::V1,
            chunk_begin: chunk::Begin::default(),
        };

        assert!(Header::deserialize(&mut deserializer).is_ok());
    }

    #[test]
    fn deserialize_invalid_header() {
        let data = "4D Geometry File Format ".as_bytes();

        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: Version::V1,
            chunk_begin: chunk::Begin::default(),
        };
        assert!(Header::deserialize(&mut deserializer).is_err());
    }

    #[test]
    fn deserialize_io_error() {
        let data = "3D Geometry File Format".as_bytes();

        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: Version::V1,
            chunk_begin: chunk::Begin::default(),
        };
        assert!(Header::deserialize(&mut deserializer).is_err());
    }
}
