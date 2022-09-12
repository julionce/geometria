pub mod application;
pub mod archive;
mod bool;
pub mod chunk;
mod comment;
mod date;
mod deserialize;
mod deserializer;
mod header;
pub mod notes;
mod on_version;
mod preview_image;
mod properties;
mod reader;
pub mod revision_history;
mod start_section;
mod string;
mod time;
mod typecode;
mod version;

#[cfg(test)]
mod tests {
    use super::{archive::Archive, deserialize::Deserialize, version::Version, *};
    use reader::Reader;
    use std::fs::File;

    #[test]
    fn serialize_3dm_v1() {
        let mut deserializer = Reader {
            stream: File::open("tests/resources/serializer/rhino/v1/v1_three_points.3dm").unwrap(),
            version: Version::V1,
            chunk_begin: chunk::Begin::default(),
        };
        match Archive::deserialize(&mut deserializer) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn serialize_3dm_v2() {
        let mut deserializer = Reader {
            stream: File::open("tests/resources/serializer/rhino/v2/v2_my_brep.3dm").unwrap(),
            version: Version::V1,
            chunk_begin: chunk::Begin::default(),
        };
        match Archive::deserialize(&mut deserializer) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }
}
