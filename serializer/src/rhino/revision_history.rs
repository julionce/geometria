use geometria_derive::RhinoDeserialize;

use super::{
    chunk,
    deserialize::Deserialize,
    deserializer::Deserializer,
    string::{StringWithLength, WStringWithLength},
    time::Time,
    version::Version,
};

#[derive(Default, RhinoDeserialize)]
pub struct RevisionHistoryV1 {
    #[underlying_type(StringWithLength)]
    pub created_by: String,
    pub create_time: Time,
    #[padding(i32)]
    #[underlying_type(StringWithLength)]
    pub last_edited_by: String,
    pub last_edit_time: Time,
    #[padding(i32)]
    pub revision_count: i32,
}

#[derive(Default, RhinoDeserialize)]
#[big_chunk_version(major == 1)]
pub struct RevisionHistoryV2 {
    #[underlying_type(WStringWithLength)]
    pub created_by: String,
    pub create_time: Time,
    #[underlying_type(WStringWithLength)]
    pub last_edited_by: String,
    pub last_edit_time: Time,
    pub revision_count: i32,
}

pub enum RevisionHistory {
    V1(RevisionHistoryV1),
    V2(RevisionHistoryV2),
}

impl Default for RevisionHistory {
    fn default() -> Self {
        Self::V1(RevisionHistoryV1::default())
    }
}

impl<D> Deserialize<'_, D> for RevisionHistory
where
    D: Deserializer,
{
    type Error = String;

    fn deserialize(deserializer: &mut D) -> Result<Self, Self::Error> {
        let revision_history;
        if Version::V1 == deserializer.version() {
            revision_history = RevisionHistory::V1(RevisionHistoryV1::deserialize(deserializer)?);
        } else {
            revision_history = RevisionHistory::V2(RevisionHistoryV2::deserialize(deserializer)?);
        }
        Ok(revision_history)
    }
}
