use super::chunk;
use super::version::Version;
use std::{io::Read, io::Seek};

pub trait Deserializer
where
    Self: Sized + Read + Seek,
{
    fn deserialize_bytes(&mut self, buf: &mut [u8]) -> Result<(), String>;
    fn deserialize_u8(&mut self) -> Result<u8, String>;
    fn deserialize_i32(&mut self) -> Result<i32, String>;
    fn deserialize_u32(&mut self) -> Result<u32, String>;
    fn deserialize_i64(&mut self) -> Result<i64, String>;

    fn version(&self) -> Version;
    fn set_version(&mut self, version: Version);

    fn chunk_begin(&self) -> chunk::Begin;
    fn set_chunk_begin(&mut self, chunk_begin: chunk::Begin);
}
