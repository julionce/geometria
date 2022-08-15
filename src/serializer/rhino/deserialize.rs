use std::fmt::Debug;
use std::fmt::Display;
use std::mem;

use super::deserializer::Deserializer;

pub trait Deserialize
where
    Self: Sized,
{
    type Error: Debug + Display;

    fn deserialize<D>(deserializer: &mut D) -> Result<Self, Self::Error>
    where
        D: Deserializer;
}

macro_rules! impl_deserialize_num {
    ($sty:ty) => {
        impl Deserialize for $sty {
            type Error = String;

            fn deserialize<D>(deserializer: &mut D) -> Result<Self, Self::Error>
            where
                D: Deserializer,
            {
                let mut bytes = [0; mem::size_of::<Self>()];
                match deserializer.read_exact(&mut bytes) {
                    Ok(()) => Ok(Self::from_le_bytes(bytes)),
                    Err(e) => Err(format!("{}", e)),
                }
            }
        }
    };
}

impl_deserialize_num! {u8}
impl_deserialize_num! {u16}
impl_deserialize_num! {u32}
impl_deserialize_num! {u64}
impl_deserialize_num! {u128}

impl_deserialize_num! {i8}
impl_deserialize_num! {i16}
impl_deserialize_num! {i32}
impl_deserialize_num! {i64}
impl_deserialize_num! {i128}

impl_deserialize_num! {usize}
impl_deserialize_num! {isize}

impl_deserialize_num! {f32}
impl_deserialize_num! {f64}
