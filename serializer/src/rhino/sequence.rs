use super::{deserialize::Deserialize, deserializer::Deserializer};

#[derive(Default)]
pub struct Sequence<T> {
    pub data: Vec<T>,
}

impl<T> From<Sequence<T>> for Vec<T> {
    fn from(array: Sequence<T>) -> Self {
        array.data
    }
}

impl<D, T> Deserialize<'_, D> for Sequence<T>
where
    D: Deserializer,
    T: for<'a> Deserialize<'a, D>,
    String: for<'a> From<<T as Deserialize<'a, D>>::Error>,
{
    type Error = String;

    fn deserialize(deserializer: &mut D) -> Result<Self, Self::Error> {
        let length = i32::deserialize(deserializer)?;
        if 0 <= length {
            let mut data: Vec<T> = vec![];
            for _ in 0..length {
                data.push(T::deserialize(deserializer)?);
            }
            Ok(Self { data })
        } else {
            Err("invalid array length".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::rhino::{chunk, reader::Reader, version::Version};

    use super::*;

    #[test]
    fn invalid_length() {
        let mut data: Vec<u8> = vec![];
        data.extend((-1i32).to_le_bytes());

        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: Version::V1,
            chunk_begin: chunk::Begin::default(),
        };
        assert!(Sequence::<u8>::deserialize(&mut deserializer).is_err());
    }

    #[test]
    fn short_length() {
        let mut data: Vec<u8> = vec![];
        data.extend((2i32).to_le_bytes());
        data.push(0);

        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: Version::V1,
            chunk_begin: chunk::Begin::default(),
        };
        assert!(Sequence::<u8>::deserialize(&mut deserializer).is_err());
    }

    #[test]
    fn ok_length() {
        let mut data: Vec<u8> = vec![];
        data.extend((2i32).to_le_bytes());
        data.push(0);
        data.push(1);

        let mut deserializer = Reader {
            stream: &mut Cursor::new(data),
            version: Version::V1,
            chunk_begin: chunk::Begin::default(),
        };
        assert_eq!(
            Vec::<u8>::from(Sequence::<u8>::deserialize(&mut deserializer).ok().unwrap()),
            vec![0, 1]
        );
    }
}
