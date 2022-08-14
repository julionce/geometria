use super::deserializer::Deserializer;

pub trait Deserialize
where
    Self: Sized,
{
    type Error;

    fn deserialize<D>(deserializer: &mut D) -> Result<Self, Self::Error>
    where
        D: Deserializer;
}
