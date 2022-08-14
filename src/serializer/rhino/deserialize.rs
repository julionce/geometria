use super::deserializer::Deserializer;

pub trait Deserialize
where
    Self: Sized,
{
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, String>
    where
        D: Deserializer;
}
