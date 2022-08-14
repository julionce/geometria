use std::fmt::Debug;
use std::fmt::Display;

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
