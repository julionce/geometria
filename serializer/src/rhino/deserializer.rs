use once_io::OStream;

use super::chunk;
use super::version::Version;

pub trait Deserializer
where
    Self: Sized + OStream,
{
    fn deserialize_bytes(&mut self, buf: &mut [u8]) -> Result<(), String>;

    fn version(&self) -> Version;
    fn set_version(&mut self, version: Version);

    fn chunk_begin(&self) -> chunk::Begin;
    fn set_chunk_begin(&mut self, chunk_begin: chunk::Begin);
}
