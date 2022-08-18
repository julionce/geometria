use super::{deserialize::Deserialize, deserializer::Deserializer};

#[derive(Default)]
pub struct Time {
    second: u32,
    minute: u32,
    hour: u32,
    month_day: u32,
    month: u32,
    year: u32,
    week_day: u32,
    year_day: u32,
}

impl Deserialize for Time {
    type Error = String;

    fn deserialize<D>(deserializer: &mut D) -> Result<Self, Self::Error>
    where
        D: Deserializer,
    {
        Ok(Self {
            second: u32::deserialize(deserializer)?,
            minute: u32::deserialize(deserializer)?,
            hour: u32::deserialize(deserializer)?,
            month_day: u32::deserialize(deserializer)?,
            month: u32::deserialize(deserializer)?,
            year: u32::deserialize(deserializer)?,
            week_day: u32::deserialize(deserializer)?,
            year_day: u32::deserialize(deserializer)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::Cursor,
        io::{Seek, SeekFrom, Write},
        mem,
    };

    use crate::serializer::rhino::{chunk, reader::Reader, version::Version};

    use super::*;

    #[test]
    fn deserialize_ok() {
        let data = [0; mem::size_of::<Time>()];
        let mut cursor = Cursor::new(data);
        let second = 1u32;
        cursor.write(&second.to_le_bytes()).unwrap();
        let minute = 2u32;
        cursor.write(&minute.to_le_bytes()).unwrap();
        let hour = 3u32;
        cursor.write(&hour.to_le_bytes()).unwrap();
        let month_day = 4u32;
        cursor.write(&month_day.to_le_bytes()).unwrap();
        let month = 5u32;
        cursor.write(&month.to_le_bytes()).unwrap();
        let year = 6u32;
        cursor.write(&year.to_le_bytes()).unwrap();
        let week_day = 7u32;
        cursor.write(&week_day.to_le_bytes()).unwrap();
        let year_day = 8u32;
        cursor.write(&year_day.to_le_bytes()).unwrap();
        cursor.seek(SeekFrom::Start(0)).unwrap();

        let mut deserializer = Reader {
            stream: &mut cursor,
            version: Version::V1,
            chunk_begin: chunk::Begin::default(),
        };

        let time = Time::deserialize(&mut deserializer).unwrap();
        assert_eq!(time.second, second);
        assert_eq!(time.minute, minute);
        assert_eq!(time.hour, hour);
        assert_eq!(time.month_day, month_day);
        assert_eq!(time.month, month);
        assert_eq!(time.year, year);
        assert_eq!(time.week_day, week_day);
        assert_eq!(time.year_day, year_day);
    }
}
