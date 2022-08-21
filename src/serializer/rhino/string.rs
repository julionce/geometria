use std::io::Read;

use super::{chunk, deserialize::Deserialize, deserializer::Deserializer};

pub struct StringWithLength(pub String);

impl Deserialize for StringWithLength {
    type Error = String;

    fn deserialize<D>(deserializer: &mut D) -> Result<Self, Self::Error>
    where
        D: Deserializer,
    {
        let length = u32::deserialize(deserializer)?;
        let mut string = String::new();
        match deserializer.take(length as u64).read_to_string(&mut string) {
            Ok(size) => {
                if size as u64 == length as u64 {
                    Ok(StringWithLength(string))
                } else {
                    Err("Invalid length".to_string())
                }
            }
            Err(e) => Err(format!("{}", e)),
        }
    }
}

impl From<StringWithLength> for String {
    fn from(value: StringWithLength) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::serializer::rhino::chunk::Begin;
    use crate::serializer::rhino::deserialize::Deserialize;
    use crate::serializer::rhino::reader::Reader;
    use crate::serializer::rhino::string::StringWithChunkValue;
    use crate::serializer::rhino::typecode;
    use crate::serializer::rhino::version::Version as FileVersion;

    use super::StringWithLength;

    #[test]
    fn deserialize_string_with_length() {
        let string = "The string".to_string();
        let size: u32 = string.len() as u32;
        let mut data: Vec<u8> = vec![];
        data.extend(size.to_le_bytes().iter().clone());
        data.extend(string.as_bytes().iter().clone());

        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: FileVersion::V1,
            chunk_begin: Begin::default(),
        };

        let string_with_length = StringWithLength::deserialize(&mut deserializer).unwrap();
        assert_eq!(string, String::from(string_with_length));
    }

    #[test]
    fn deserialize_string_with_length_and_invalid_length() {
        let string = "The string".to_string();
        let size: u32 = (string.len() + 1) as u32;
        let mut data: Vec<u8> = vec![];
        data.extend(size.to_le_bytes().iter().clone());
        data.extend(string.as_bytes().iter().clone());

        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: FileVersion::V1,
            chunk_begin: Begin::default(),
        };

        assert!(StringWithLength::deserialize(&mut deserializer).is_err());
    }
}
