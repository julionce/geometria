pub mod application;
pub mod chunk;
mod comment;
mod date;
mod deserialize;
mod deserializer;
mod header;
pub mod notes;
mod on_version;
mod reader;
mod start_section;
mod string;
mod time;
mod typecode;
mod version;

use chunk::Chunk;
use deserialize::Deserialize;
use deserializer::Deserializer;
use on_version::Version as OnVersion;
use string::StringWithLength;
use time::Time;
use version::Version;

use std::io::{Seek, SeekFrom};

use self::{application::Application, notes::Notes, string::WStringWithLength};

#[derive(Default)]
struct RevisionHistory {
    created_by: String,
    last_edited_by: String,
    create_time: Time,
    last_edit_time: Time,
    revision_count: i32,
}

#[derive(Default)]
struct Properties {
    filename: String,
    version: OnVersion,
    revision_history: RevisionHistory,
    notes: Notes,
    application: Application,
}

impl<D> Deserialize<'_, D> for RevisionHistory
where
    D: Deserializer,
{
    type Error = String;

    fn deserialize(deserializer: &mut D) -> Result<Self, Self::Error> {
        let mut revision_history = RevisionHistory::default();
        if Version::V1 == deserializer.version() {
            revision_history.created_by = StringWithLength::deserialize(deserializer)?.into();
            revision_history.create_time = Time::deserialize(deserializer)?;
            i32::deserialize(deserializer)?;
            revision_history.last_edited_by = StringWithLength::deserialize(deserializer)?.into();
            revision_history.last_edit_time = Time::deserialize(deserializer)?;
            i32::deserialize(deserializer)?;
            revision_history.revision_count = i32::deserialize(deserializer)?;
        } else {
            let chunk_version = chunk::Version::deserialize(deserializer)?;
            if 1u8 == chunk_version.major() {
                revision_history.created_by = WStringWithLength::deserialize(deserializer)?.into();
                revision_history.create_time = Time::deserialize(deserializer)?;
                revision_history.last_edited_by =
                    WStringWithLength::deserialize(deserializer)?.into();
                revision_history.last_edit_time = Time::deserialize(deserializer)?;
                revision_history.revision_count = i32::deserialize(deserializer)?;
            }
        }
        Ok(revision_history)
    }
}

impl<D> Deserialize<'_, D> for Properties
where
    D: Deserializer,
{
    type Error = String;

    fn deserialize(deserializer: &mut D) -> Result<Self, Self::Error> {
        let mut properties = Properties::default();
        if Version::V1 == deserializer.version() {
            deserializer.seek(SeekFrom::Start(32u64)).unwrap();
            loop {
                let mut chunk = Chunk::deserialize(deserializer)?;
                match chunk.chunk_begin().typecode {
                    typecode::COMMENTBLOCK => {
                        // TODO: process _comment
                        let _comment = String::deserialize(&mut chunk)?;
                    }
                    typecode::SUMMARY => {
                        properties.revision_history = RevisionHistory::deserialize(&mut chunk)?;
                    }
                    typecode::NOTES => {
                        properties.notes = Notes::deserialize(&mut chunk)?;
                    }
                    typecode::BITMAPPREVIEW | typecode::CURRENTLAYER | typecode::LAYER => {
                        // TODO
                    }
                    _ => {
                        break;
                    }
                }
                chunk.seek(SeekFrom::End(1)).unwrap();
            }
        } else {
            let mut properties_chunk = Chunk::deserialize(deserializer)?;
            if typecode::PROPERTIES_TABLE == properties_chunk.chunk_begin().typecode {
                loop {
                    let mut chunk = Chunk::deserialize(&mut properties_chunk)?;
                    match chunk.chunk_begin().typecode {
                        typecode::PROPERTIES_OPENNURBS_VERSION => {
                            properties.version = OnVersion::deserialize(&mut chunk)?;
                        }
                        typecode::PROPERTIES_AS_FILE_NAME => {
                            properties.filename =
                                WStringWithLength::deserialize(&mut chunk)?.into();
                        }
                        typecode::PROPERTIES_REVISIONHISTORY => {
                            properties.revision_history = RevisionHistory::deserialize(&mut chunk)?;
                        }
                        typecode::PROPERTIES_NOTES => {
                            properties.notes = Notes::deserialize(&mut chunk)?;
                        }
                        typecode::PROPERTIES_APPLICATION => {
                            properties.application = Application::deserialize(&mut chunk)?;
                        }
                        typecode::PROPERTIES_PREVIEWIMAGE
                        | typecode::PROPERTIES_COMPRESSED_PREVIEWIMAGE => {
                            // TODO
                        }
                        _ => {
                            break;
                        }
                    }
                    chunk.seek(SeekFrom::End(1)).unwrap();
                }
            }
            properties_chunk.seek(SeekFrom::End(1)).unwrap();
        }
        Ok(properties)
    }
}

#[cfg(test)]
mod tests {
    use super::{comment::Comment, start_section::StartSection, *};
    use header::Header;
    use reader::Reader;
    use std::fs::File;

    #[test]
    fn serialize_3dm() {
        let mut deserializer = Reader {
            stream: File::open("src/serializer/rhino/test_file/v1/v1_three_points.3dm").unwrap(),
            version: Version::V1,
            chunk_begin: chunk::Begin::default(),
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
        match Comment::deserialize(&mut deserializer) {
            Ok(_) => {
                assert!(true)
            }
            Err(_) => assert!(false),
        }
        match StartSection::deserialize(&mut deserializer) {
            Ok(_) => {
                assert!(true)
            }
            Err(_) => assert!(false),
        }
        match Properties::deserialize(&mut deserializer) {
            Ok(_) => {
                assert!(true)
            }
            Err(_) => assert!(false),
        }
    }
}
