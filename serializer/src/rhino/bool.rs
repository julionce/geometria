use super::{deserialize::Deserialize, deserializer::Deserializer};

pub struct BoolFromI32(bool);

impl<D> Deserialize<'_, D> for BoolFromI32
where
    D: Deserializer,
{
    type Error = String;

    fn deserialize(deserializer: &mut D) -> Result<Self, Self::Error> {
        Ok(Self(i32::deserialize(deserializer)? != 0))
    }
}

impl From<BoolFromI32> for bool {
    fn from(value: BoolFromI32) -> Self {
        value.0
    }
}
