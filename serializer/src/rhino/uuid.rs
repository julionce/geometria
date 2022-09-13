use geometria_derive::Deserialize;

use super::{deserialize::Deserialize, deserializer::Deserializer};

#[derive(Deserialize)]
pub struct Uuid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 4],
}
