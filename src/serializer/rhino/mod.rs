mod typecode;

use std::{convert::TryFrom, io::Read, io::Seek, io::SeekFrom, mem};

const FILE_BEGIN: &[u8] = "3D Geometry File Format ".as_bytes();

struct Header;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Version {
    V1,
    V2,
    V3,
    V4,
    V50,
    V60,
    V70,
}

impl TryFrom<u8> for Version {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Version::V1),
            2 => Ok(Version::V2),
            3 => Ok(Version::V3),
            4 => Ok(Version::V4),
            50 => Ok(Version::V50),
            60 => Ok(Version::V60),
            70 => Ok(Version::V70),
            _ => Err(format!("3dm file error: invalid version {}", value)),
        }
    }
}

#[derive(Copy, Clone, Default)]
struct ChunkBegin {
    typecode: u32,
    value: i64,
}

struct Chunk<T> {
    begin: ChunkBegin,
    data: T,
}

struct ChunkString(String);

impl ChunkBegin {
    fn size_of_length(version: Version) -> u8 {
        match version {
            Version::V1 | Version::V2 | Version::V3 | Version::V4 => 4u8,
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

struct Comment(String);

trait Deserializer
where
    Self: Sized + Read + Seek,
{
    fn deserialize_bytes(&mut self, buf: &mut [u8]) -> Result<(), String>;
    fn deserialize_i32(&mut self) -> Result<i32, String>;
    fn deserialize_u32(&mut self) -> Result<u32, String>;
    fn deserialize_i64(&mut self) -> Result<i64, String>;

    fn version(&self) -> Version;
    fn set_version(&mut self, version: Version);

    fn chunk_begin(&self) -> ChunkBegin;
    fn set_chunk_begin(&mut self, chunk_begin: ChunkBegin);
}

struct ReadDeserializer<'a, T>
where
    T: Read + Seek,
{
    stream: &'a mut T,
    version: Version,
    chunk_begin: ChunkBegin,
}

impl<T> Read for ReadDeserializer<'_, T>
where
    T: Read + Seek,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(buf)
    }
}

impl<T> Seek for ReadDeserializer<'_, T>
where
    T: Read + Seek,
{
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.stream.seek(pos)
    }
}

impl<T> Deserializer for ReadDeserializer<'_, T>
where
    T: Read + Seek,
{
    fn deserialize_bytes(&mut self, buf: &mut [u8]) -> Result<(), String> {
        match self.read_exact(buf) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("{}", e)),
        }
    }

    fn deserialize_i32(&mut self) -> Result<i32, String> {
        let mut buffer = [0; mem::size_of::<i32>()];
        match self.read_exact(&mut buffer) {
            Ok(()) => Ok(i32::from_le_bytes(buffer)),
            Err(e) => Err(format!("{}", e)),
        }
    }

    fn deserialize_u32(&mut self) -> Result<u32, String> {
        let mut buffer = [0; mem::size_of::<u32>()];
        match self.read_exact(&mut buffer) {
            Ok(()) => Ok(u32::from_le_bytes(buffer)),
            Err(e) => Err(format!("{}", e)),
        }
    }

    fn deserialize_i64(&mut self) -> Result<i64, String> {
        let mut buffer = [0; mem::size_of::<i64>()];
        match self.read_exact(&mut buffer) {
            Ok(()) => Ok(i64::from_le_bytes(buffer)),
            Err(e) => Err(format!("{}", e)),
        }
    }

    fn version(&self) -> Version {
        return self.version;
    }

    fn set_version(&mut self, version: Version) {
        self.version = version;
    }

    fn chunk_begin(&self) -> ChunkBegin {
        return self.chunk_begin;
    }

    fn set_chunk_begin(&mut self, chunk_begin: ChunkBegin) {
        self.chunk_begin = chunk_begin;
    }
}

trait Deserialize
where
    Self: Sized,
{
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, String>
    where
        D: Deserializer;
}

trait DeserializeChunk
where
    Self: Sized,
{
    fn deserialize<D>(deserializer: &mut D, chunk_begin: ChunkBegin) -> Result<Self, String>
    where
        D: Deserializer;
}

impl Deserialize for Header {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, String>
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

impl Deserialize for Version {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, String>
    where
        D: Deserializer,
    {
        const ERROR_MSG: &str = "3dm file error: unable to read file version";
        let mut buffer = [0; 8];
        match deserializer.deserialize_bytes(&mut buffer) {
            Ok(()) => {
                match buffer
                    .iter()
                    .skip_while(|x| **x == ' ' as u8)
                    .try_fold(0u8, |acc, x| match (*x as char).to_digit(10) {
                        Some(d) => Ok(acc * 10u8 + (d as u8)),
                        None => Err(""),
                    }) {
                    Ok(v) => match Version::try_from(v) {
                        Ok(version) => {
                            deserializer.set_version(version);
                            Ok(version)
                        }
                        Err(e) => Err(e),
                    },
                    Err(_) => Err(ERROR_MSG.to_string()),
                }
            }
            Err(_) => Err(ERROR_MSG.to_string()),
        }
    }
}

impl Deserialize for ChunkBegin {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, String>
    where
        D: Deserializer,
    {
        let mut chunk_begin = ChunkBegin {
            typecode: deserializer.deserialize_u32().unwrap(),
            value: 0i64,
        };
        if 8 == ChunkBegin::size_of_length(deserializer.version()) {
            chunk_begin.value = deserializer.deserialize_i64().unwrap();
        } else if chunk_begin.is_unsigned() {
            chunk_begin.value = deserializer.deserialize_u32().unwrap() as i64;
        } else {
            chunk_begin.value = deserializer.deserialize_i32().unwrap() as i64;
        }
        deserializer.set_chunk_begin(chunk_begin);
        Ok(chunk_begin)
    }
}

impl<T> Deserialize for Chunk<T>
where
    T: DeserializeChunk,
{
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, String>
    where
        D: Deserializer,
    {
        let begin = ChunkBegin::deserialize(deserializer).unwrap();
        let data = T::deserialize(deserializer, begin).unwrap();
        Ok(Chunk::<T> {
            begin: begin,
            data: data,
        })
    }
}

struct EmptyChunk;
struct BackwardEmptyChunk;
struct ForwardChunk;

impl DeserializeChunk for EmptyChunk {
    fn deserialize<D>(_deserializer: &mut D, _chunk_begin: ChunkBegin) -> Result<Self, String>
    where
        D: Deserializer,
    {
        Ok(EmptyChunk {})
    }
}

impl DeserializeChunk for BackwardEmptyChunk {
    fn deserialize<D>(deserializer: &mut D, chunk_begin: ChunkBegin) -> Result<Self, String>
    where
        D: Deserializer,
    {
        deserializer
            .seek(SeekFrom::Current(
                -(mem::size_of_val(&chunk_begin.typecode) as i64
                    + ChunkBegin::size_of_length(deserializer.version()) as i64),
            ))
            .unwrap();
        Ok(BackwardEmptyChunk {})
    }
}

impl DeserializeChunk for ForwardChunk {
    fn deserialize<D>(deserializer: &mut D, chunk_begin: ChunkBegin) -> Result<Self, String>
    where
        D: Deserializer,
    {
        deserializer
            .seek(SeekFrom::Current(chunk_begin.value))
            .unwrap();
        Ok(ForwardChunk {})
    }
}

impl DeserializeChunk for String {
    fn deserialize<D>(deserializer: &mut D, chunk_begin: ChunkBegin) -> Result<Self, String>
    where
        D: Deserializer,
    {
        let mut buf = String::default();
        deserializer
            .take(chunk_begin.value as u64)
            .read_to_string(&mut buf)
            .unwrap();
        Ok(buf)
    }
}

impl Deserialize for ChunkString {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, String>
    where
        D: Deserializer,
    {
        let chunk_begin = ChunkBegin::deserialize(deserializer).unwrap();
        let mut buf = String::default();
        deserializer
            .take(chunk_begin.value as u64)
            .read_to_string(&mut buf)
            .unwrap();
        Ok(ChunkString(buf))
    }
}

impl Deserialize for Comment {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, String>
    where
        D: Deserializer,
    {
        let comment = Chunk::<String>::deserialize(deserializer).unwrap().data;
        Ok(Comment(comment))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, io::BufReader};

    #[test]
    fn serialize_3dm() {
        let file = File::open("src/serializer/rhino/test_file/v1/v1_three_points.3dm").unwrap();
        let mut deserializer = ReadDeserializer {
            stream: &mut BufReader::new(file),
            version: Version::V1,
            chunk_begin: ChunkBegin::default(),
        };
        match Header::deserialize(&mut deserializer) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
        match Version::deserialize(&mut deserializer) {
            Ok(version) => {
                assert_eq!(Version::V1, version);
                assert_eq!(Version::V1, deserializer.version())
            }
            Err(_) => assert!(false),
        }
        match Comment::deserialize(&mut deserializer) {
            Ok(_) => {
                assert!(true)
            }
            Err(_) => assert!(false),
        }
    }
}
