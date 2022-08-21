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

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::serializer::rhino::typecode::{self};
    use crate::serializer::rhino::version::Version as FileVersion;
    use crate::serializer::rhino::{deserialize::Deserialize, reader::Reader};

    use super::{Begin, Value, Version};

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
}
