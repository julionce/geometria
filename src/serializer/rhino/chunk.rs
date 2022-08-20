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

    use crate::serializer::rhino::{deserialize::Deserialize, reader::Reader, version};

    use super::{Begin, Version};

    #[test]
    fn deserialize_version() {
        let major_version = 1u8;
        let minor_version = 2u8;
        let data = [major_version << 4 | (minor_version & 0x0F); 1];

        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: version::Version::V1,
            chunk_begin: Begin::default(),
        };

        let version = Version::deserialize(&mut deserializer).unwrap();
        assert_eq!(major_version, version.major());
        assert_eq!(minor_version, version.minor());
    }
}
