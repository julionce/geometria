use std::io::Seek;

use super::{chunk::Chunk, deserialize::Deserialize, deserializer::Deserializer, typecode};

pub struct Comment(String);

impl<D> Deserialize<'_, D> for Comment
where
    D: Deserializer,
{
    type Error = String;

    fn deserialize(deserializer: &mut D) -> Result<Self, Self::Error> {
        let mut chunk = Chunk::deserialize(deserializer)?;
        if typecode::COMMENTBLOCK == chunk.chunk_begin().typecode {
            Ok(Comment(String::deserialize(&mut chunk)?))
        } else {
            Err("Invalid typecode".to_string())
        }
    }
}

impl From<Comment> for String {
    fn from(comment: Comment) -> Self {
        comment.0
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::serializer::rhino::{
        chunk::Begin, deserialize::Deserialize, reader::Reader, typecode,
        version::Version as FileVersion,
    };

    use super::Comment;

    #[test]
    fn deserialize_comment() {
        let string = "The comment".to_string();
        let value = string.len() as u32;
        let typecode = typecode::COMMENTBLOCK;
        let mut data: Vec<u8> = Vec::new();
        data.extend(typecode.to_le_bytes().iter().clone());
        data.extend(value.to_le_bytes().iter().clone());
        data.extend(string.as_bytes().iter().clone());

        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: FileVersion::V1,
            chunk_begin: Begin::default(),
        };

        let comment = Comment::deserialize(&mut deserializer).unwrap();
        assert_eq!(string, String::from(comment));
    }

    #[test]
    fn deserialize_comment_with_invalid_typecode() {
        let string = "The comment".to_string();
        let value = string.len() as u32;
        let typecode = 0u32;
        let mut data: Vec<u8> = Vec::new();
        data.extend(typecode.to_le_bytes().iter().clone());
        data.extend(value.to_le_bytes().iter().clone());
        data.extend(string.as_bytes().iter().clone());

        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: FileVersion::V1,
            chunk_begin: Begin::default(),
        };

        assert!(Comment::deserialize(&mut deserializer).is_err());
    }
}
