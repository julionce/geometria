use super::chunk;
use super::deserializer::Deserializer;
use super::version::Version;

use std::{io::Read, io::Seek, io::SeekFrom, mem};

pub struct Reader<'a, T>
where
    T: Read + Seek,
{
    pub stream: &'a mut T,
    pub version: Version,
    pub chunk_begin: chunk::Begin,
}

impl<T> Read for Reader<'_, T>
where
    T: Read + Seek,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(buf)
    }
}

impl<T> Seek for Reader<'_, T>
where
    T: Read + Seek,
{
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.stream.seek(pos)
    }
}

impl<T> Deserializer for Reader<'_, T>
where
    T: Read + Seek,
{
    fn deserialize_bytes(&mut self, buf: &mut [u8]) -> Result<(), String> {
        match self.read_exact(buf) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("{}", e)),
        }
    }

    fn deserialize_u8(&mut self) -> Result<u8, String> {
        let mut buffer = [0; mem::size_of::<u8>()];
        match self.read_exact(&mut buffer) {
            Ok(()) => Ok(u8::from_le_bytes(buffer)),
            Err(e) => Err(format!("{}", e)),
        }
    }

    fn deserialize_i32(&mut self) -> Result<i32, String> {
        let mut buffer = [0; mem::size_of::<i32>()];
        match self.read_exact(&mut buffer) {
            Ok(()) => Ok(i32::from_le_bytes(buffer)),
            Err(e) => Err(format!("{}", e)),
        }
    }

    fn deserialize_u32(&mut self) -> Result<u32, String> {
        let mut buffer = [0; mem::size_of::<u32>()];
        match self.read_exact(&mut buffer) {
            Ok(()) => Ok(u32::from_le_bytes(buffer)),
            Err(e) => Err(format!("{}", e)),
        }
    }

    fn deserialize_i64(&mut self) -> Result<i64, String> {
        let mut buffer = [0; mem::size_of::<i64>()];
        match self.read_exact(&mut buffer) {
            Ok(()) => Ok(i64::from_le_bytes(buffer)),
            Err(e) => Err(format!("{}", e)),
        }
    }

    fn version(&self) -> Version {
        return self.version;
    }

    fn set_version(&mut self, version: Version) {
        self.version = version;
    }

    fn chunk_begin(&self) -> chunk::Begin {
        return self.chunk_begin;
    }

    fn set_chunk_begin(&mut self, chunk_begin: chunk::Begin) {
        self.chunk_begin = chunk_begin;
    }
}
