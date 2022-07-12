use std::{convert::TryFrom, io::Read, mem};

const FILE_BEGIN: &[u8] = "3D Geometry File Format ".as_bytes();

struct Header;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
enum Version {
    #[default]
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

impl ChunkBegin {
    fn size_of_length(version: Version) -> u8 {
        match version {
            Version::V1 | Version::V2 | Version::V3 | Version::V4 => 4u8,
            _ => 8u8,
        }
    }

    fn is_unsigned(self) -> bool {
        false
    }
}

trait Deserializer
where
    Self: Sized,
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

struct ReadDeserializer<'a> {
    reader: &'a mut (dyn Read + 'a),
    version: Version,
    chunk_begin: ChunkBegin,
}

impl Deserializer for ReadDeserializer<'_> {
    fn deserialize_bytes(&mut self, buf: &mut [u8]) -> Result<(), String> {
        match self.reader.read_exact(buf) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("{}", e)),
        }
    }

    fn deserialize_i32(&mut self) -> Result<i32, String> {
        let mut buffer = [0; mem::size_of::<i32>()];
        match self.reader.read_exact(&mut buffer) {
            Ok(()) => Ok(i32::from_le_bytes(buffer)),
            Err(e) => Err(format!("{}", e)),
        }
    }

    fn deserialize_u32(&mut self) -> Result<u32, String> {
        let mut buffer = [0; mem::size_of::<u32>()];
        match self.reader.read_exact(&mut buffer) {
            Ok(()) => Ok(u32::from_le_bytes(buffer)),
            Err(e) => Err(format!("{}", e)),
        }
    }

    fn deserialize_i64(&mut self) -> Result<i64, String> {
        let mut buffer = [0; mem::size_of::<i64>()];
        match self.reader.read_exact(&mut buffer) {
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
        let mut chunk_begin = ChunkBegin::default();
        chunk_begin.typecode = deserializer.deserialize_u32().unwrap();
        if 8 == ChunkBegin::size_of_length(deserializer.version()) {
            chunk_begin.value = deserializer.deserialize_i64().unwrap();
        } else {
            if chunk_begin.is_unsigned() {
                chunk_begin.value = deserializer.deserialize_u32().unwrap() as i64;
            } else {
                chunk_begin.value = deserializer.deserialize_i32().unwrap() as i64;
            }
        }
        deserializer.set_chunk_begin(chunk_begin);
        Ok(chunk_begin)
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
            reader: &mut BufReader::new(file),
            version: Version::default(),
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
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
