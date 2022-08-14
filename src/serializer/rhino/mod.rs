mod chunk;
mod deserialize;
mod deserializer;
mod reader;
mod typecode;
mod version;

use deserialize::Deserialize;
use deserializer::Deserializer;
use version::Version;

use std::{io::Read, io::SeekFrom, mem};

const FILE_BEGIN: &[u8] = "3D Geometry File Format ".as_bytes();

struct Header;

struct ChunkVersion {
    minor: u8,
    major: u8,
}

struct Chunk<T> {
    begin: chunk::Begin,
    data: T,
}

struct ChunkString(String);

struct Comment(String);

struct StartSection;

struct StringWithLength {
    length: u32,
    string: String,
}

#[derive(Default)]
struct Time {
    second: u32,
    minute: u32,
    hour: u32,
    month_day: u32,
    month: u32,
    year: u32,
    week_day: u32,
    year_day: u32,
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
    version: on_version::Version,
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

impl Deserialize for Header {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, String>
    where
        D: Deserializer,
    {
        let mut buffer = [0; FILE_BEGIN.len()];
        match deserializer.deserialize_bytes(&mut buffer) {
            Ok(()) => match FILE_BEGIN == buffer {
                true => Ok(Header {}),
                false => Err("3dm file error: invalid file begin".to_string()),
            },
            Err(e) => Err(e),
        }
    }
}

impl Deserialize for Version {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, String>
    where
        D: Deserializer,
    {
        const ERROR_MSG: &str = "3dm file error: unable to read file version";
        let mut buffer = [0; 8];
        match deserializer.deserialize_bytes(&mut buffer) {
            Ok(()) => {
                match buffer
                    .iter()
                    .skip_while(|x| **x == ' ' as u8)
                    .try_fold(0u8, |acc, x| match (*x as char).to_digit(10) {
                        Some(d) => Ok(acc * 10u8 + (d as u8)),
                        None => Err(""),
                    }) {
                    Ok(v) => match Version::try_from(v) {
                        Ok(version) => {
                            deserializer.set_version(version);
                            Ok(version)
                        }
                        Err(e) => Err(e),
                    },
                    Err(_) => Err(ERROR_MSG.to_string()),
                }
            }
            Err(_) => Err(ERROR_MSG.to_string()),
        }
    }
}

impl Deserialize for ChunkVersion {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, String>
    where
        D: Deserializer,
    {
        let raw_version = deserializer.deserialize_u8()?;
        Ok(ChunkVersion {
            minor: raw_version | 0x0F,
            major: raw_version >> 4,
        })
    }
}

impl<T> Deserialize for Chunk<T>
where
    T: DeserializeChunk,
{
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, String>
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

impl Deserialize for ChunkString {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, String>
    where
        D: Deserializer,
    {
        let chunk_begin = chunk::Begin::deserialize(deserializer).unwrap();
        let mut buf = String::default();
        deserializer
            .take(chunk_begin.value as u64)
            .read_to_string(&mut buf)
            .unwrap();
        Ok(ChunkString(buf))
    }
}

impl Deserialize for Comment {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, String>
    where
        D: Deserializer,
    {
        let comment = Chunk::<String>::deserialize(deserializer).unwrap().data;
        Ok(Comment(comment))
    }
}

impl Deserialize for StartSection {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, String>
    where
        D: Deserializer,
    {
        let initial_position = SeekFrom::Start(deserializer.stream_position().unwrap());
        if Version::V1 == deserializer.version() {
            loop {
                let empty_chunk = Chunk::<BackwardEmptyChunk>::deserialize(deserializer).unwrap();
                match empty_chunk.begin.typecode {
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
                        let _forward_chunk =
                            Chunk::<ForwardChunk>::deserialize(deserializer).unwrap();
                        let _testing = false;
                    }
                    _ => {
                        if typecode::TABLE == empty_chunk.begin.typecode & 0xFFFF0000 {
                            deserializer.set_version(Version::V2);
                        }
                        break;
                    }
                }
            }
        }

        if Version::V1 == deserializer.version() {
            deserializer.seek(initial_position).unwrap();
        }
        Ok(StartSection {})
    }
}

impl Deserialize for StringWithLength {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, String>
    where
        D: Deserializer,
    {
        let length = deserializer.deserialize_u32().unwrap();
        let mut string = String::default();
        deserializer
            .take(length as u64)
            .read_to_string(&mut string)
            .unwrap();
        Ok(StringWithLength { length, string })
    }
}

impl Deserialize for Time {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, String>
    where
        D: Deserializer,
    {
        let mut time = Time::default();
        time.second = deserializer.deserialize_u32()?;
        time.minute = deserializer.deserialize_u32()?;
        time.hour = deserializer.deserialize_u32()?;
        time.month_day = deserializer.deserialize_u32()?;
        time.month = deserializer.deserialize_u32()?;
        time.year = deserializer.deserialize_u32()?;
        time.week_day = deserializer.deserialize_u32()?;
        time.year_day = deserializer.deserialize_u32()?;
        Ok(time)
    }
}

impl Deserialize for RevisionHistory {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, String>
    where
        D: Deserializer,
    {
        let mut revision_history = RevisionHistory::default();
        if Version::V1 == deserializer.version() {
            revision_history.created_by = StringWithLength::deserialize(deserializer)?.string;
            revision_history.create_time = Time::deserialize(deserializer)?;
            deserializer.deserialize_i32()?;
            revision_history.last_edited_by = StringWithLength::deserialize(deserializer)?.string;
            revision_history.last_edit_time = Time::deserialize(deserializer)?;
            deserializer.deserialize_i32()?;
            revision_history.revision_count = deserializer.deserialize_i32()?;
        } else {
            let chunk_version = ChunkVersion::deserialize(deserializer)?;
            if 1u8 == chunk_version.major {
                // TODO
                // revision_history.created_by = WStringWithLength::deserialize(deserializer)?;
                revision_history.create_time = Time::deserialize(deserializer)?;
                // TODO
                // revision_history.last_edited_by = WStringWithLength::deserialize(deserializer)?;
                revision_history.last_edit_time = Time::deserialize(deserializer)?;
                revision_history.revision_count = deserializer.deserialize_i32()?;
            }
        }
        Ok(revision_history)
    }
}

impl Deserialize for Notes {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, String>
    where
        D: Deserializer,
    {
        let mut notes = Notes::default();
        if Version::V1 == deserializer.version() {
            notes.visible = deserializer.deserialize_i32()? != 0i32;
            notes.window_left = deserializer.deserialize_i32()?;
            notes.window_top = deserializer.deserialize_i32()?;
            notes.window_right = deserializer.deserialize_i32()?;
            notes.window_bottom = deserializer.deserialize_i32()?;
            notes.data = StringWithLength::deserialize(deserializer)?.string;
        } else {
            let chunk_version = ChunkVersion::deserialize(deserializer)?;
            if 1u8 == chunk_version.major {
                notes.html_encoded = deserializer.deserialize_i32()? != 0i32;
                // TODO
                // notes.data = WStringWithLength::deserialize(deserializer)?.string;
                notes.visible = deserializer.deserialize_i32()? != 0i32;
                notes.window_left = deserializer.deserialize_i32()?;
                notes.window_top = deserializer.deserialize_i32()?;
                notes.window_right = deserializer.deserialize_i32()?;
                notes.window_bottom = deserializer.deserialize_i32()?;
            }
        }
        Ok(notes)
    }
}

impl Deserialize for Properties {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, String>
    where
        D: Deserializer,
    {
        let mut properties = Properties::default();
        if Version::V1 == deserializer.version() {
            deserializer.seek(SeekFrom::Start(32u64)).unwrap();
            loop {
                let chunk_begin = chunk::Begin::deserialize(deserializer).unwrap();
                match chunk_begin.typecode {
                    typecode::COMMENTBLOCK => {
                        let _comment = String::deserialize(deserializer, chunk_begin).unwrap();
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
                match deserializer.seek(SeekFrom::Start(
                    chunk_begin.initial_position + chunk_begin.value as u64,
                )) {
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
    use super::*;
    use reader::Reader;
    use std::{fs::File, io::BufReader};

    #[test]
    fn serialize_3dm() {
        let file = File::open("src/serializer/rhino/test_file/v1/v1_three_points.3dm").unwrap();
        let mut deserializer = Reader {
            stream: &mut BufReader::new(file),
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
