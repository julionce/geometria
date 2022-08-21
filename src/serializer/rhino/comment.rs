use super::{
    deserialize::Deserialize, deserializer::Deserializer, string::StringWithChunkValue, typecode,
    typecode::Typecode,
};

pub struct Comment(String);

impl Deserialize for Comment {
    type Error = String;

    fn deserialize<D>(deserializer: &mut D) -> Result<Self, Self::Error>
    where
        D: Deserializer,
    {
        let typecode = Typecode::deserialize(deserializer)?;
        if typecode::COMMENTBLOCK == typecode {
            Ok(Comment(
                StringWithChunkValue::deserialize(deserializer)?.into(),
            ))
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
