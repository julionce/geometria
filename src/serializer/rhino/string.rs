use std::io::Read;

use super::{chunk, deserialize::Deserialize, deserializer::Deserializer};

impl<D> Deserialize<'_, D> for String
where
    D: Deserializer,
{
    type Error = String;

    fn deserialize(deserializer: &mut D) -> Result<Self, Self::Error> {
        let mut string = String::new();
        match deserializer.read_to_string(&mut string) {
            Ok(_) => Ok(string),
            Err(e) => {
                println!("{}", e);
                Err(format!("{}", e))
            }
        }
    }
}

pub struct StringWithLength(pub String);

impl<D> Deserialize<'_, D> for StringWithLength
where
    D: Deserializer,
{
    type Error = String;

    fn deserialize(deserializer: &mut D) -> Result<Self, Self::Error> {
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

pub struct StringWithChunkValue(pub String);

impl<D> Deserialize<'_, D> for StringWithChunkValue
where
    D: Deserializer,
{
    type Error = String;

    fn deserialize(deserializer: &mut D) -> Result<Self, Self::Error> {
        let chunk_value: i64 = chunk::Value::deserialize(deserializer)?.into();
        if chunk_value > 0 {
            let mut string = String::new();
            match deserializer
                .take(chunk_value as u64)
                .read_to_string(&mut string)
            {
                Ok(size) => {
                    if size as u64 == chunk_value as u64 {
                        Ok(StringWithChunkValue(string))
                    } else {
                        Err("Invalid length".to_string())
                    }
                }
                Err(e) => Err(format!("{}", e)),
            }
        } else {
            Err("Negative length".to_string())
        }
    }
}

impl From<StringWithChunkValue> for String {
    fn from(value: StringWithChunkValue) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::serializer::rhino::chunk::Begin;
    use crate::serializer::rhino::deserialize::Deserialize;
    use crate::serializer::rhino::reader::Reader;
    use crate::serializer::rhino::typecode;
    use crate::serializer::rhino::version::Version as FileVersion;

    use super::StringWithChunkValue;
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
    fn deserialize_string_with_invalid_length() {
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

    #[test]
    fn deserialize_string_with_chunk_value() {
        let string = "The string".to_string();
        let value: u32 = string.len() as u32;
        let mut data: Vec<u8> = vec![];
        data.extend(value.to_le_bytes().iter().clone());
        data.extend(string.as_bytes().iter().clone());

        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: FileVersion::V1,
            chunk_begin: Begin::default(),
        };

        let string_with_chunk_value = StringWithChunkValue::deserialize(&mut deserializer).unwrap();
        assert_eq!(string, String::from(string_with_chunk_value));
    }

    #[test]
    fn deserialize_string_with_invalid_chunk_value() {
        let string = "The string".to_string();
        let value: u32 = (string.len() + 1) as u32;
        let mut data: Vec<u8> = vec![];
        data.extend(value.to_le_bytes().iter().clone());
        data.extend(string.as_bytes().iter().clone());

        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: FileVersion::V1,
            chunk_begin: Begin::default(),
        };

        assert!(StringWithChunkValue::deserialize(&mut deserializer).is_err());
    }

    #[test]
    fn deserialize_string_with_negative_chunk_value() {
        let string = "The string".to_string();
        let value: i32 = -(string.len() as i32);
        let mut data: Vec<u8> = vec![];
        data.extend(value.to_le_bytes().iter().clone());
        data.extend(string.as_bytes().iter().clone());

        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: FileVersion::V1,
            chunk_begin: Begin {
                typecode: typecode::SHORT,
                value: 0,
                initial_position: 0,
            },
        };

        assert!(StringWithChunkValue::deserialize(&mut deserializer).is_err());
    }
}
