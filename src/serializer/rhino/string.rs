use std::io::Read;

use super::{deserialize::Deserialize, deserializer::Deserializer};

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

pub struct WStringWithLength(pub String);

impl<D> Deserialize<'_, D> for WStringWithLength
where
    D: Deserializer,
{
    type Error = String;

    fn deserialize(deserializer: &mut D) -> Result<Self, Self::Error> {
        let length = u32::deserialize(deserializer)? - 1;
        let mut buf: Vec<u16> = vec![];
        for _ in 0..length {
            buf.push(u16::deserialize(deserializer)?);
        }
        u16::deserialize(deserializer)?;
        match String::from_utf16(&buf) {
            Ok(string) => Ok(Self(string)),
            Err(e) => Err(e.to_string()),
        }
    }
}

impl From<WStringWithLength> for String {
    fn from(value: WStringWithLength) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::serializer::rhino::chunk::Begin;
    use crate::serializer::rhino::deserialize::Deserialize;
    use crate::serializer::rhino::reader::Reader;
    use crate::serializer::rhino::string::WStringWithLength;
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
    fn deserialize_wstring_with_length_ok() {
        let mut string = "The string\0".to_string();
        let size: u32 = string.encode_utf16().count() as u32;
        let mut data: Vec<u8> = vec![];
        data.extend(size.to_le_bytes().iter().clone());
        string
            .encode_utf16()
            .for_each(|r| data.extend(r.to_le_bytes().iter()));
        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: FileVersion::V1,
            chunk_begin: Begin::default(),
        };
        let wstring_with_length = WStringWithLength::deserialize(&mut deserializer).unwrap();
        string.pop();
        assert_eq!(string, String::from(wstring_with_length));
    }

    #[test]
    fn deserialize_wstring_with_invalid_lenth() {
        let string = "The string\0".to_string();
        let size: u32 = (string.encode_utf16().count() + 1) as u32;
        let mut data: Vec<u8> = vec![];
        data.extend(size.to_le_bytes().iter().clone());
        string
            .encode_utf16()
            .for_each(|r| data.extend(r.to_le_bytes().iter()));
        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: FileVersion::V1,
            chunk_begin: Begin::default(),
        };
        assert!(WStringWithLength::deserialize(&mut deserializer).is_err());
    }
}
