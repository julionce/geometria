use super::typecode;
use super::version;
// TODO: remove
use super::*;

#[derive(Copy, Clone, Default)]
pub struct Begin {
    pub typecode: u32,
    pub value: i64,
    pub initial_position: u64,
}

impl Begin {
    // TODO: mark as private
    pub fn size_of_length(version: version::Version) -> u8 {
        match version {
            version::Version::V1
            | version::Version::V2
            | version::Version::V3
            | version::Version::V4 => 4u8,
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

impl Deserialize for chunk::Begin {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, String>
    where
        D: Deserializer,
    {
        let mut chunk_begin = chunk::Begin {
            typecode: deserializer.deserialize_u32().unwrap(),
            value: 0i64,
            initial_position: 0u64,
        };
        if 8 == chunk::Begin::size_of_length(deserializer.version()) {
            chunk_begin.value = deserializer.deserialize_i64().unwrap();
        } else if chunk_begin.is_unsigned() {
            chunk_begin.value = deserializer.deserialize_u32().unwrap() as i64;
        } else {
            chunk_begin.value = deserializer.deserialize_i32().unwrap() as i64;
        }
        match deserializer.stream_position() {
            Ok(position) => chunk_begin.initial_position = position,
            Err(e) => return Err(format!("{}", e)),
        }
        deserializer.set_chunk_begin(chunk_begin);
        Ok(chunk_begin)
    }
}
