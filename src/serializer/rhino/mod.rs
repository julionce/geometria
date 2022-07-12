use std::{io::Read, mem};

const FILE_BEGIN: &[u8] = "3D Geometry File Format ".as_bytes();

struct Header;

trait Deserializer
where
    Self: Sized,
{
    fn deserialize_bytes(&mut self, buf: &mut [u8]) -> Result<(), String>;
    fn deserialize_u32(&mut self) -> Result<u32, String>;
}

struct ReadDeserializer<'a> {
    reader: &'a mut (dyn Read + 'a),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, io::BufReader};

    #[test]
    fn serialize_3dm() {
        let file = File::open("src/serializer/rhino/test_file/v1/v1_three_points.3dm").unwrap();
        let mut deserializer = ReadDeserializer {
            reader: &mut BufReader::new(file),
        };
        match Header::deserialize(&mut deserializer) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
