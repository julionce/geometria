use std::io::{Read, Seek, SeekFrom};

use super::deserialize::Deserialize;
use super::deserializer::Deserializer;
use super::typecode::{self, Typecode};
use super::version::Version as FileVersion;

#[derive(Copy, Clone, Default)]
pub struct Begin {
    pub typecode: Typecode,
    pub value: i64,
    pub initial_position: u64,
}

impl Begin {
    // TODO: mark as private
    pub fn size_of_length(version: FileVersion) -> u8 {
        match version {
            FileVersion::V1 | FileVersion::V2 | FileVersion::V3 | FileVersion::V4 => 4u8,
            _ => 8u8,
        }
    }

    fn is_unsigned(self) -> bool {
        0 == (typecode::SHORT & self.typecode)
            || typecode::RGB == self.typecode
            || typecode::RGBDISPLAY == self.typecode
            || typecode::PROPERTIES_OPENNURBS_VERSION == self.typecode
            || typecode::OBJECT_RECORD_TYPE == self.typecode
    }
}

impl Deserialize for Begin {
    type Error = String;

    fn deserialize<D>(deserializer: &mut D) -> Result<Self, Self::Error>
    where
        D: Deserializer,
    {
        let mut chunk_begin = Begin {
            typecode: u32::deserialize(deserializer)?,
            value: 0i64,
            initial_position: 0u64,
        };
        if 8 == Begin::size_of_length(deserializer.version()) {
            chunk_begin.value = i64::deserialize(deserializer)?;
        } else if chunk_begin.is_unsigned() {
            chunk_begin.value = u32::deserialize(deserializer)? as i64;
        } else {
            chunk_begin.value = i32::deserialize(deserializer)? as i64;
        }
        match deserializer.stream_position() {
            Ok(position) => chunk_begin.initial_position = position,
            Err(e) => return Err(format!("{}", e)),
        }
        deserializer.set_chunk_begin(chunk_begin);
        Ok(chunk_begin)
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Value(i64);

impl Value {
    fn size(version: FileVersion) -> u8 {
        match version {
            FileVersion::V1 | FileVersion::V2 | FileVersion::V3 | FileVersion::V4 => 4u8,
            _ => 8u8,
        }
    }

    fn is_unsigned(typecode: Typecode) -> bool {
        0 == (typecode::SHORT & typecode)
            || typecode::RGB == typecode
            || typecode::RGBDISPLAY == typecode
            || typecode::PROPERTIES_OPENNURBS_VERSION == typecode
            || typecode::OBJECT_RECORD_TYPE == typecode
    }
}

impl From<Value> for i64 {
    fn from(value: Value) -> Self {
        value.0
    }
}

impl Deserialize for Value {
    type Error = String;

    fn deserialize<D>(deserializer: &mut D) -> Result<Self, Self::Error>
    where
        D: Deserializer,
    {
        if 8 == Self::size(deserializer.version()) {
            Ok(Self(i64::deserialize(deserializer)?))
        } else if Self::is_unsigned(deserializer.chunk_begin().typecode) {
            Ok(Self(u32::deserialize(deserializer)? as i64))
        } else {
            Ok(Self(i32::deserialize(deserializer)? as i64))
        }
    }
}

pub struct Version {
    inner: u8,
}

impl Version {
    pub fn minor(&self) -> u8 {
        self.inner & 0x0F
    }

    pub fn major(&self) -> u8 {
        self.inner >> 4
    }
}

impl Deserialize for Version {
    type Error = String;

    fn deserialize<D>(deserializer: &mut D) -> Result<Self, Self::Error>
    where
        D: Deserializer,
    {
        Ok(Self {
            inner: u8::deserialize(deserializer)?,
        })
    }
}

pub struct Chunk<'a, T>
where
    T: Read + Seek,
{
    stream: &'a mut T,
    offset: u64,
    length: u64,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ChunkError {
    EmptyChunk,
    OutOfBounds,
    InvalidInput,
}

impl From<ChunkError> for std::io::Error {
    fn from(chunk_error: ChunkError) -> Self {
        match chunk_error {
            ChunkError::EmptyChunk => std::io::Error::new(
                std::io::ErrorKind::Other,
                "chunk with null length is not allowed",
            ),
            ChunkError::OutOfBounds => std::io::Error::new(
                std::io::ErrorKind::Other,
                "the current stream position is out of bounds",
            ),
            ChunkError::InvalidInput => std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid seek to a negative or overflowing position",
            ),
        }
    }
}

impl PartialEq<std::io::Error> for ChunkError {
    fn eq(&self, other: &std::io::Error) -> bool {
        let converted_error = std::io::Error::from(*self);
        converted_error.kind() == other.kind() && converted_error.to_string() == other.to_string()
    }
}

impl<'a, T> Chunk<'a, T>
where
    T: Read + Seek,
{
    pub fn new(stream: &'a mut T, offset: u64, length: u64) -> Result<Self, ChunkError> {
        if 0 == length {
            Err(ChunkError::EmptyChunk)
        } else {
            Ok(Self {
                stream,
                offset,
                length,
            })
        }
    }

    pub fn start_position(&self) -> u64 {
        self.offset
    }

    pub fn end_position(&self) -> u64 {
        self.offset + (self.length - 1)
    }

    pub fn length(&self) -> u64 {
        self.length
    }

    fn current_position(&mut self) -> std::io::Result<u64> {
        let stream_position = self.stream.stream_position()?;
        if stream_position < self.start_position() || stream_position > self.end_position() {
            Err(std::io::Error::from(ChunkError::OutOfBounds))
        } else {
            Ok(stream_position)
        }
    }

    fn remainder_length(&mut self) -> std::io::Result<u64> {
        Ok(self.offset + self.length - self.current_position()?)
    }

    fn comsumed_length(&mut self) -> std::io::Result<u64> {
        Ok(self.current_position()? - self.start_position())
    }
}

impl<'a, T> Read for Chunk<'a, T>
where
    T: Read + Seek,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let length = std::cmp::min(self.remainder_length()? as usize, buf.len());
        self.stream.read(&mut buf[0..length])
    }
}

impl<'a, T> Seek for Chunk<'a, T>
where
    T: Read + Seek,
{
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let final_position: u64 = match pos {
            SeekFrom::Start(value) => Ok(self.start_position() + value),
            SeekFrom::End(value) => {
                if 0 <= value {
                    Ok(self.end_position() + (value) as u64)
                } else if (value.abs() as u64) < self.length {
                    Ok(self.end_position() - (value.abs() as u64))
                } else {
                    Err(std::io::Error::from(ChunkError::InvalidInput))
                }
            }
            SeekFrom::Current(value) => {
                let current_position = self.current_position()?;
                if 0 < value {
                    Ok(current_position + (value as u64))
                } else if (value.abs() as u64) <= self.comsumed_length()? {
                    Ok(current_position - (value.abs() as u64))
                } else {
                    Err(std::io::Error::from(ChunkError::InvalidInput))
                }
            }
        }?;
        match self.stream.seek(SeekFrom::Start(final_position)) {
            Ok(value) => {
                if value == final_position {
                    Ok(final_position - self.start_position())
                } else {
                    Err(std::io::Error::from(ChunkError::OutOfBounds))
                }
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read, Seek, SeekFrom};

    use crate::serializer::rhino::chunk::ChunkError;
    use crate::serializer::rhino::typecode::{self};
    use crate::serializer::rhino::version::Version as FileVersion;
    use crate::serializer::rhino::{deserialize::Deserialize, reader::Reader};

    use super::{Begin, Chunk, Value, Version};

    #[test]
    fn deserialize_version() {
        let major_version = 1u8;
        let minor_version = 2u8;
        let data = [major_version << 4 | (minor_version & 0x0F); 1];

        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: FileVersion::V1,
            chunk_begin: Begin::default(),
        };

        let version = Version::deserialize(&mut deserializer).unwrap();
        assert_eq!(major_version, version.major());
        assert_eq!(minor_version, version.minor());
    }

    #[test]
    fn value_size() {
        assert_eq!(4, Value::size(FileVersion::V1));
        assert_eq!(4, Value::size(FileVersion::V2));
        assert_eq!(4, Value::size(FileVersion::V3));
        assert_eq!(4, Value::size(FileVersion::V4));
        assert_eq!(8, Value::size(FileVersion::V50));
        assert_eq!(8, Value::size(FileVersion::V60));
        assert_eq!(8, Value::size(FileVersion::V70));
    }

    #[test]
    fn value_is_unsigned() {
        assert!(Value::is_unsigned(typecode::RGB));
        assert!(Value::is_unsigned(typecode::RGBDISPLAY));
        assert!(Value::is_unsigned(typecode::PROPERTIES_OPENNURBS_VERSION));
        assert!(Value::is_unsigned(typecode::OBJECT_RECORD_TYPE));
        assert!(Value::is_unsigned(!typecode::SHORT));
        assert!(Value::is_unsigned(0));
        assert!(!Value::is_unsigned(typecode::SHORT));
    }

    #[test]
    fn deserialize_value_0_size_8() {
        let data = 0i64.to_le_bytes();
        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: FileVersion::V50,
            chunk_begin: Begin {
                typecode: 0,
                value: 0,
                initial_position: 0,
            },
        };
        assert_eq!(
            Value::deserialize(&mut deserializer).ok(),
            Some(Value(0i64))
        );
    }

    #[test]
    fn deserialize_value_max_size_8() {
        let data = i64::MAX.to_le_bytes();
        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: FileVersion::V50,
            chunk_begin: Begin {
                typecode: 0,
                value: 0,
                initial_position: 0,
            },
        };
        assert_eq!(
            Value::deserialize(&mut deserializer).ok(),
            Some(Value(i64::MAX))
        );
    }

    #[test]
    fn deserialize_value_min_size_8() {
        let data = i64::MIN.to_le_bytes();
        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: FileVersion::V50,
            chunk_begin: Begin {
                typecode: 0,
                value: 0,
                initial_position: 0,
            },
        };
        assert_eq!(
            Value::deserialize(&mut deserializer).ok(),
            Some(Value(i64::MIN))
        );
    }

    #[test]
    fn deserialize_value_0_size_4_unsigned() {
        let data = 0u32.to_le_bytes();
        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: FileVersion::V1,
            chunk_begin: Begin {
                typecode: typecode::RGB,
                value: 0,
                initial_position: 0,
            },
        };
        assert_eq!(
            Value::deserialize(&mut deserializer).ok(),
            Some(Value(0i64))
        );
    }

    #[test]
    fn deserialize_value_min_size_4_unsigned() {
        let data = u32::MIN.to_le_bytes();
        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: FileVersion::V1,
            chunk_begin: Begin {
                typecode: typecode::RGB,
                value: 0,
                initial_position: 0,
            },
        };
        assert_eq!(
            Value::deserialize(&mut deserializer).ok(),
            Some(Value(u32::MIN as i64))
        );
    }

    #[test]
    fn deserialize_value_max_size_4_unsigned() {
        let data = u32::MAX.to_le_bytes();
        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: FileVersion::V1,
            chunk_begin: Begin {
                typecode: typecode::RGB,
                value: 0,
                initial_position: 0,
            },
        };
        assert_eq!(
            Value::deserialize(&mut deserializer).ok(),
            Some(Value(u32::MAX as i64))
        );
    }

    #[test]
    fn deserialize_value_0_size_4_signed() {
        let data = 0i32.to_le_bytes();
        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: FileVersion::V1,
            chunk_begin: Begin {
                typecode: typecode::SHORT,
                value: 0,
                initial_position: 0,
            },
        };
        assert_eq!(
            Value::deserialize(&mut deserializer).ok(),
            Some(Value(0i64))
        );
    }

    #[test]
    fn deserialize_value_min_size_4_signed() {
        let data = i32::MIN.to_le_bytes();
        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: FileVersion::V1,
            chunk_begin: Begin {
                typecode: typecode::SHORT,
                value: 0,
                initial_position: 0,
            },
        };
        assert_eq!(
            Value::deserialize(&mut deserializer).ok(),
            Some(Value(i32::MIN as i64))
        );
    }

    #[test]
    fn deserialize_value_max_size_4_signed() {
        let data = i32::MAX.to_le_bytes();
        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: FileVersion::V1,
            chunk_begin: Begin {
                typecode: typecode::SHORT,
                value: 0,
                initial_position: 0,
            },
        };
        assert_eq!(
            Value::deserialize(&mut deserializer).ok(),
            Some(Value(i32::MAX as i64))
        );
    }

    #[test]
    fn new_not_empty_chunk() {
        let data = [0; 1];
        let mut stream = Cursor::new(data);
        let chunk = Chunk::new(&mut stream, 0, 1);
        assert!(chunk.is_ok());
        let result = chunk.ok().unwrap();
        assert_eq!(result.offset, 0);
        assert_eq!(result.length, 1);
    }

    #[test]
    fn new_empty_chunk() {
        let data = [0; 1];
        let mut stream = Cursor::new(data);
        let chunk = Chunk::new(&mut stream, 0, 0);
        assert_eq!(chunk.err(), Some(ChunkError::EmptyChunk));
    }

    #[test]
    fn chunk_start_position() {
        let data = [0; 10];
        let mut stream = Cursor::new(data);
        let chunk = Chunk::new(&mut stream, 1, 1).unwrap();
        assert_eq!(1, chunk.start_position());
    }

    #[test]
    fn chunk_end_position() {
        let data = [0; 10];
        let mut stream = Cursor::new(data);
        let chunk = Chunk::new(&mut stream, 1, 2).unwrap();
        assert_eq!(2, chunk.end_position());
    }

    #[test]
    fn chunk_length() {
        let data = [0; 10];
        let mut stream = Cursor::new(data);
        let chunk = Chunk::new(&mut stream, 1, 2).unwrap();
        assert_eq!(2, chunk.length());
    }

    #[test]
    fn chunk_current_position() {
        let data = [0; 11];
        let mut stream = Cursor::new(data);
        let offset = 1u64;
        let length = 9u64;

        stream.set_position(offset - 1);
        {
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            let result = chunk.current_position();
            assert!(result.is_err());
            assert_eq!(ChunkError::OutOfBounds, result.err().unwrap());
        }

        stream.set_position(offset);
        {
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            assert_eq!(Some(1), chunk.current_position().ok());
        }

        stream.set_position(offset + length - 1);
        {
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            assert_eq!(Some(9), chunk.current_position().ok());
        }

        stream.set_position(offset + length);
        {
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            let result = chunk.current_position();
            assert!(result.is_err());
            assert_eq!(ChunkError::OutOfBounds, result.err().unwrap());
        }
    }

    #[test]
    fn chunk_remainder_length() {
        let data = [0; 11];
        let mut stream = Cursor::new(data);
        let offset = 1u64;
        let length = 9u64;

        stream.set_position(offset - 1);
        {
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            let result = chunk.remainder_length();
            assert!(result.is_err());
            assert_eq!(ChunkError::OutOfBounds, result.err().unwrap());
        }

        stream.set_position(offset);
        {
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            assert_eq!(Some(length), chunk.remainder_length().ok());
        }

        stream.set_position(offset + length - 1);
        {
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            assert_eq!(Some(1), chunk.remainder_length().ok());
        }

        stream.set_position(offset + length);
        {
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            let result = chunk.remainder_length();
            assert!(result.is_err());
            assert_eq!(ChunkError::OutOfBounds, result.err().unwrap());
        }
    }

    #[test]
    fn consumed_remainder_length() {
        let data = [0; 11];
        let mut stream = Cursor::new(data);
        let offset = 1u64;
        let length = 9u64;

        stream.set_position(offset - 1);
        {
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            let result = chunk.comsumed_length();
            assert!(result.is_err());
            assert_eq!(ChunkError::OutOfBounds, result.err().unwrap());
        }

        stream.set_position(offset);
        {
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            assert_eq!(Some(0), chunk.comsumed_length().ok());
        }

        stream.set_position(offset + length - 1);
        {
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            assert_eq!(Some(length - 1), chunk.comsumed_length().ok());
        }

        stream.set_position(offset + length);
        {
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            let result = chunk.comsumed_length();
            assert!(result.is_err());
            assert_eq!(ChunkError::OutOfBounds, result.err().unwrap());
        }
    }

    #[test]
    fn seek_chunk_from_start() {
        let data = [0; 11];
        let mut stream = Cursor::new(data);
        let offset = 1u64;
        let length = 9u64;

        {
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            assert_eq!(Some(0), chunk.seek(SeekFrom::Start(0)).ok());
        }
        assert_eq!(offset, stream.position());

        {
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            assert_eq!(Some(length), chunk.seek(SeekFrom::Start(length)).ok());
        }
        assert_eq!(offset + length, stream.position());
    }

    #[test]
    fn seek_chunk_from_end() {
        let data = [0; 11];
        let mut stream = Cursor::new(data);
        let offset = 1u64;
        let length = 9u64;

        {
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            assert_eq!(Some(length), chunk.seek(SeekFrom::End(1)).ok());
        }
        assert_eq!(offset + length, stream.position());

        {
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            assert_eq!(Some(length - 1), chunk.seek(SeekFrom::End(0)).ok());
        }
        assert_eq!(offset + length - 1, stream.position());

        {
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            assert_eq!(Some(0), chunk.seek(SeekFrom::End(1 - (length as i64))).ok());
        }
        assert_eq!(offset, stream.position());

        {
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            assert_eq!(ChunkError::InvalidInput, chunk.seek(SeekFrom::End(-(length as i64))).err().unwrap());
        }
        assert_eq!(offset, stream.position());
    }

    #[test]
    fn seek_chunk_from_current() {
        let data = [0; 11];
        let mut stream = Cursor::new(data);
        let offset = 1u64;
        let length = 9u64;

        stream.set_position(offset - 1);
        {
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            assert_eq!(ChunkError::OutOfBounds, chunk.seek(SeekFrom::Current(0)).err().unwrap());
            assert_eq!(ChunkError::OutOfBounds, chunk.seek(SeekFrom::Current(1)).err().unwrap());
        }
        assert_eq!(offset - 1, stream.position());

        stream.set_position(offset);
        {
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            assert_eq!(Some(0), chunk.seek(SeekFrom::Current(0)).ok());
        }
        assert_eq!(offset, stream.position());

        stream.set_position(offset);
        {
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            assert_eq!(ChunkError::InvalidInput, chunk.seek(SeekFrom::Current(-1)).err().unwrap());
        }
        assert_eq!(offset, stream.position());

        stream.set_position(offset);
        {
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            assert_eq!(Some(length), chunk.seek(SeekFrom::Current(length as i64)).ok());
        }
        assert_eq!(offset + length, stream.position());

        stream.set_position(offset + 1);
        {
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            assert_eq!(Some(0), chunk.seek(SeekFrom::Current(-1)).ok());
        }
        assert_eq!(offset, stream.position());
    }

    #[test]
    fn read_chunk() {
        let data: Vec<u8> = (0..11).collect();
        let mut stream = Cursor::new(data);
        let offset = 1u64;
        let length = 9u64;

        {
            let mut buf = [0; 10];
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            assert_eq!(ChunkError::OutOfBounds, chunk.read(&mut buf).err().unwrap());
        }

        {
            let mut buf = [0; 10];
            let mut chunk = Chunk::new(&mut stream, offset, length).unwrap();
            chunk.seek(SeekFrom::Start(0));
            assert_eq!(Some(length as usize), chunk.read(&mut buf).ok());
            let mut expected = (1..=9).collect::<Vec<u8>>();
            expected.push(0);
            assert_eq!(buf, expected[..]);
        }
        assert_eq!(offset + length, stream.position());
    }
}
