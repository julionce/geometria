use geometria_derive::Deserialize;

use super::{
    chunk, deserialize::Deserialize, deserializer::Deserializer, string::WStringWithLength,
};

#[derive(Default, Deserialize)]
#[big_chunk_version]
pub struct Application {
    #[underlying_type(WStringWithLength)]
    name: String,
    #[underlying_type(WStringWithLength)]
    url: String,
    #[underlying_type(WStringWithLength)]
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
