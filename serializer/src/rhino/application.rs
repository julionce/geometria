use super::{
    chunk, deserialize::Deserialize, deserializer::Deserializer, string::WStringWithLength,
};

#[derive(Default)]
pub struct Application {
    name: String,
    url: String,
    details: String,
}

impl Application {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn details(&self) -> &str {
        &self.details
    }
}

impl<D> Deserialize<'_, D> for Application
where
    D: Deserializer,
{
    type Error = String;

    fn deserialize(deserializer: &mut D) -> Result<Self, Self::Error> {
        let _chunk_version = chunk::Version::deserialize(deserializer)?;
        Ok(Application {
            name: WStringWithLength::deserialize(deserializer)?.into(),
            url: WStringWithLength::deserialize(deserializer)?.into(),
            details: WStringWithLength::deserialize(deserializer)?.into(),
        })
    }
}
