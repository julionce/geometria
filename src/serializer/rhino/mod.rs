mod chunk;
mod comment;
mod date;
mod deserialize;
mod deserializer;
mod header;
mod on_version;
mod reader;
mod start_section;
mod string;
mod time;
mod typecode;
mod version;

use deserialize::Deserialize;
use deserializer::Deserializer;
use on_version::Version as OnVersion;
use string::StringWithLength;
use time::Time;
use typecode::Typecode;
use version::Version;

use std::{io::Read, io::SeekFrom};

use self::{chunk::Value, comment::Comment};

struct Chunk<T> {
    begin: chunk::Begin,
    data: T,
}

#[derive(Default)]
struct RevisionHistory {
    created_by: String,
    last_edited_by: String,
    create_time: Time,
    last_edit_time: Time,
    revision_count: i32,
}

#[derive(Default)]
struct Notes {
    data: String,
    visible: bool,
    html_encoded: bool,
    window_left: i32,
    window_top: i32,
    window_right: i32,
    window_bottom: i32,
}

#[derive(Default)]
struct Properties {
    version: OnVersion,
    revision_history: RevisionHistory,
    notes: Notes,
}

trait DeserializeChunk
where
    Self: Sized,
{
    fn deserialize<D>(deserializer: &mut D, chunk_begin: chunk::Begin) -> Result<Self, String>
    where
        D: Deserializer;
}

impl<T> Deserialize for Chunk<T>
where
    T: DeserializeChunk,
{
    type Error = String;

    fn deserialize<D>(deserializer: &mut D) -> Result<Self, Self::Error>
    where
        D: Deserializer,
    {
        let begin = chunk::Begin::deserialize(deserializer).unwrap();
        let data = T::deserialize(deserializer, begin).unwrap();
        Ok(Chunk::<T> {
            begin: begin,
            data: data,
        })
    }
}

struct EmptyChunk;
struct BackwardEmptyChunk;
struct ForwardChunk;

impl DeserializeChunk for EmptyChunk {
    fn deserialize<D>(_deserializer: &mut D, _chunk_begin: chunk::Begin) -> Result<Self, String>
    where
        D: Deserializer,
    {
        Ok(EmptyChunk {})
    }
}

impl DeserializeChunk for BackwardEmptyChunk {
    fn deserialize<D>(deserializer: &mut D, chunk_begin: chunk::Begin) -> Result<Self, String>
    where
        D: Deserializer,
    {
        deserializer
            .seek(SeekFrom::Current(
                -(mem::size_of_val(&chunk_begin.typecode) as i64
                    + chunk::Begin::size_of_length(deserializer.version()) as i64),
            ))
            .unwrap();
        Ok(BackwardEmptyChunk {})
    }
}

impl DeserializeChunk for ForwardChunk {
    fn deserialize<D>(deserializer: &mut D, chunk_begin: chunk::Begin) -> Result<Self, String>
    where
        D: Deserializer,
    {
        deserializer
            .seek(SeekFrom::Current(chunk_begin.value))
            .unwrap();
        Ok(ForwardChunk {})
    }
}

impl DeserializeChunk for String {
    fn deserialize<D>(deserializer: &mut D, chunk_begin: chunk::Begin) -> Result<Self, String>
    where
        D: Deserializer,
    {
        let mut buf = String::default();
        deserializer
            .take(chunk_begin.value as u64)
            .read_to_string(&mut buf)
            .unwrap();
        Ok(buf)
    }
}

impl Deserialize for RevisionHistory {
    type Error = String;

    fn deserialize<D>(deserializer: &mut D) -> Result<Self, Self::Error>
    where
        D: Deserializer,
    {
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
                // TODO
                // revision_history.created_by = WStringWithLength::deserialize(deserializer)?;
                revision_history.create_time = Time::deserialize(deserializer)?;
                // TODO
                // revision_history.last_edited_by = WStringWithLength::deserialize(deserializer)?;
                revision_history.last_edit_time = Time::deserialize(deserializer)?;
                revision_history.revision_count = i32::deserialize(deserializer)?;
            }
        }
        Ok(revision_history)
    }
}

impl Deserialize for Notes {
    type Error = String;

    fn deserialize<D>(deserializer: &mut D) -> Result<Self, Self::Error>
    where
        D: Deserializer,
    {
        let mut notes = Notes::default();
        if Version::V1 == deserializer.version() {
            notes.visible = i32::deserialize(deserializer)? != 0i32;
            notes.window_left = i32::deserialize(deserializer)?;
            notes.window_top = i32::deserialize(deserializer)?;
            notes.window_right = i32::deserialize(deserializer)?;
            notes.window_bottom = i32::deserialize(deserializer)?;
            notes.data = StringWithLength::deserialize(deserializer)?.into();
        } else {
            let chunk_version = chunk::Version::deserialize(deserializer)?;
            if 1u8 == chunk_version.major() {
                notes.html_encoded = i32::deserialize(deserializer)? != 0i32;
                // TODO
                // notes.data = WStringWithLength::deserialize(deserializer)?.string;
                notes.visible = i32::deserialize(deserializer)? != 0i32;
                notes.window_left = i32::deserialize(deserializer)?;
                notes.window_top = i32::deserialize(deserializer)?;
                notes.window_right = i32::deserialize(deserializer)?;
                notes.window_bottom = i32::deserialize(deserializer)?;
            }
        }
        Ok(notes)
    }
}

impl Deserialize for Properties {
    type Error = String;

    fn deserialize<D>(deserializer: &mut D) -> Result<Self, Self::Error>
    where
        D: Deserializer,
    {
        let mut properties = Properties::default();
        if Version::V1 == deserializer.version() {
            deserializer.seek(SeekFrom::Start(32u64)).unwrap();
            loop {
                let backup_position = deserializer.stream_position().unwrap();
                let typecode = Typecode::deserialize(deserializer)?;
                // TODO: implement TryFrom<Value> for u64.
                let value: i64 = Value::deserialize(deserializer)?.into();
                if 0 > value {
                    return Err("Invalid Chunk value".to_string());
                }
                let final_position = deserializer.stream_position().unwrap() + value as u64;
                match typecode {
                    typecode::COMMENTBLOCK => {
                        deserializer.seek(SeekFrom::Start(backup_position)).unwrap();
                        // TODO: process _comment
                        let _comment = Comment::deserialize(deserializer)?;
                    }
                    typecode::SUMMARY => {
                        properties.revision_history = RevisionHistory::deserialize(deserializer)?;
                    }
                    typecode::NOTES => {
                        properties.notes = Notes::deserialize(deserializer)?;
                    }
                    typecode::BITMAPPREVIEW => {
                        break;
                    }
                    typecode::CURRENTLAYER | typecode::LAYER => {
                        break;
                    }
                    _ => {}
                }
                // TODO: create a ScopedChunkBegin
                match deserializer.seek(SeekFrom::Start(final_position)) {
                    Ok(_) => {}
                    Err(e) => return Err(format!("{}", e)),
                }
            }
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
