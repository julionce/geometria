use std::{convert::TryFrom, io::Read, mem};

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

trait Deserializer
where
    Self: Sized,
{
    fn deserialize_bytes(&mut self, buf: &mut [u8]) -> Result<(), String>;
    fn deserialize_u32(&mut self) -> Result<u32, String>;

    fn version(&self) -> Version;
    fn set_version(&mut self, version: Version);
}

struct ReadDeserializer<'a> {
    reader: &'a mut (dyn Read + 'a),
    version: Version,
}

impl Deserializer for ReadDeserializer<'_> {
    fn deserialize_bytes(&mut self, buf: &mut [u8]) -> Result<(), String> {
        match self.reader.read_exact(buf) {
            Ok(()) => Ok(()),
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

    fn version(&self) -> Version {
        return self.version;
    }

    fn set_version(&mut self, version: Version) {
        self.version = version;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, io::BufReader};

    #[test]
    fn serialize_3dm() {
        let file = File::open("src/serializer/rhino/test_file/v1/v1_three_points.3dm").unwrap();
        let mut deserializer = ReadDeserializer {
            reader: &mut BufReader::new(file),
            version: Version::V1,
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
