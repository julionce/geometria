use std::io::SeekFrom;

use super::{
    chunk::Value, deserialize::Deserialize, deserializer::Deserializer, typecode,
    typecode::Typecode, version::Version,
};

// TODO: add version::Version as member of StartSection.
pub struct StartSection;

impl Deserialize for StartSection {
    type Error = String;

    fn deserialize<D>(deserializer: &mut D) -> Result<Self, Self::Error>
    where
        D: Deserializer,
    {
        let backup_position = SeekFrom::Start(deserializer.stream_position().unwrap());
        if Version::V1 == deserializer.version() {
            loop {
                let typecode = Typecode::deserialize(deserializer)?;
                match typecode {
                    typecode::SUMMARY
                    | typecode::BITMAPPREVIEW
                    | typecode::UNIT_AND_TOLERANCES
                    | typecode::VIEWPORT
                    | typecode::LAYER
                    | typecode::RENDERMESHPARAMS
                    | typecode::CURRENTLAYER
                    | typecode::ANNOTATION_SETTINGS
                    | typecode::NOTES
                    | typecode::NAMED_CPLANE
                    | typecode::NAMED_VIEW => {
                        let value: i64 = Value::deserialize(deserializer)?.into();
                        deserializer.seek(SeekFrom::Current(value)).unwrap();
                    }
                    _ => {
                        if typecode::TABLE == typecode & 0xFFFF0000 {
                            deserializer.set_version(Version::V2);
                        }
                        break;
                    }
                }
            }
        }

        if Version::V1 == deserializer.version() {
            deserializer.seek(backup_position).unwrap();
        }
        Ok(StartSection {})
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Seek};

    use crate::serializer::rhino::{
        chunk::Begin, deserialize::Deserialize, reader::Reader, typecode,
        version::Version as FileVersion,
    };

    use super::StartSection;

    #[test]
    fn deserialize_start_section_with_v1_header_and_body() {
        let summary_typecode = typecode::SUMMARY;
        let content = [0; 8];
        let value = content.len() as u32;
        let mut data: Vec<u8> = Vec::new();
        let empty_typecode = 0u32;
        data.extend(summary_typecode.to_le_bytes().iter().clone());
        data.extend(value.to_le_bytes().iter().clone());
        data.extend(content.iter().clone());
        data.extend(empty_typecode.to_le_bytes().iter().clone());

        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: FileVersion::V1,
            chunk_begin: Begin::default(),
        };

        assert!(StartSection::deserialize(&mut deserializer).is_ok());
        assert_eq!(deserializer.stream.stream_position().unwrap(), 0);
    }

    #[test]
    fn deserialize_start_section_with_v1_header_and_v2_body() {
        let summary_typecode = typecode::SUMMARY;
        let content = [0; 8];
        let value = content.len() as u32;
        let mut data: Vec<u8> = Vec::new();
        let empty_typecode = typecode::TABLE;
        data.extend(summary_typecode.to_le_bytes().iter().clone());
        data.extend(value.to_le_bytes().iter().clone());
        data.extend(content.iter().clone());
        data.extend(empty_typecode.to_le_bytes().iter().clone());

        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: FileVersion::V1,
            chunk_begin: Begin::default(),
        };

        assert!(StartSection::deserialize(&mut deserializer).is_ok());
        assert_ne!(deserializer.stream.stream_position().unwrap(), 0);
    }
}
