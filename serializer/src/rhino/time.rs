use super::{deserialize::Deserialize, deserializer::Deserializer};
use geometria_derive::Deserialize;

#[derive(Default, Deserialize)]
pub struct Time {
    pub second: u32,
    pub minute: u32,
    pub hour: u32,
    pub month_day: u32,
    pub month: u32,
    pub year: u32,
    pub week_day: u32,
    pub year_day: u32,
}

#[cfg(test)]
mod tests {
    use std::{
        io::Cursor,
        io::{Seek, SeekFrom, Write},
        mem,
    };

    use crate::rhino::{chunk, reader::Reader, version::Version};

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
