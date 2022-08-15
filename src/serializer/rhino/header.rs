use std::fmt::Display;

use super::deserialize::Deserialize;
use super::deserializer::Deserializer;

pub struct Header;

#[derive(Debug, PartialEq)]
pub enum HeaderError {
    InvalidHeader,
    IoError(std::io::ErrorKind),
}

impl Display for HeaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidHeader => write!(f, "invalid header"),
            Self::IoError(e) => write!(f, "{}", e),
        }
    }
}

const FILE_BEGIN: &[u8] = "3D Geometry File Format ".as_bytes();

impl Deserialize for Header {
    type Error = HeaderError;

    fn deserialize<D>(deserializer: &mut D) -> Result<Self, Self::Error>
    where
        D: Deserializer,
    {
        let mut buffer = [0; FILE_BEGIN.len()];
        match deserializer.read_exact(&mut buffer) {
            Ok(()) => match FILE_BEGIN == buffer {
                true => Ok(Header {}),
                false => Err(HeaderError::InvalidHeader),
            },
            Err(e) => Err(HeaderError::IoError(e.kind())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::serializer::rhino::{chunk, reader::Reader, version::Version};

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

        assert_eq!(
            Header::deserialize(&mut deserializer).err(),
            Some(HeaderError::InvalidHeader)
        );
    }

    #[test]
    fn deserialize_io_error() {
        let data = "3D Geometry File Format".as_bytes();

        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: Version::V1,
            chunk_begin: chunk::Begin::default(),
        };

        assert_eq!(
            Header::deserialize(&mut deserializer).err(),
            Some(HeaderError::IoError(std::io::ErrorKind::UnexpectedEof))
        );
    }
}
