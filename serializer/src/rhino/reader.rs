use super::chunk;
use super::deserializer::Deserializer;
use super::version::Version;

use std::{io::Read, io::Seek, io::SeekFrom};

pub struct Reader<T>
where
    T: Read + Seek,
{
    pub stream: T,
    pub version: Version,
    pub chunk_begin: chunk::Begin,
}

impl<T> Read for Reader<T>
where
    T: Read + Seek,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(buf)
    }
}

impl<T> Seek for Reader<T>
where
    T: Read + Seek,
{
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.stream.seek(pos)
    }
}

impl<T> Deserializer for Reader<T>
where
    T: Read + Seek,
{
    fn deserialize_bytes(&mut self, buf: &mut [u8]) -> Result<(), String> {
        match self.read_exact(buf) {
            Ok(()) => Ok(()),
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
