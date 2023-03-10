use std::io::{Read, Seek};

use crate::common::reader::{BigEndianNumberReader, LittleEndianNumberReader, NumberReader};

pub trait Deserializer: NumberReader + Read + Seek {}

impl<T> Read for BigEndianNumberReader<T>
where
    T: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.source.read(buf)
    }
}

impl<T> Seek for BigEndianNumberReader<T>
where
    T: Read + Seek,
{
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.source.seek(pos)
    }
}

impl<T> Deserializer for BigEndianNumberReader<T> where T: Read + Seek {}

impl<T> Read for LittleEndianNumberReader<T>
where
    T: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.source.read(buf)
    }
}

impl<T> Seek for LittleEndianNumberReader<T>
where
    T: Read + Seek,
{
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.source.seek(pos)
    }
}

impl<T> Deserializer for LittleEndianNumberReader<T> where T: Read + Seek {}
