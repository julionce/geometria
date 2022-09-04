use super::{
    chunk,
    deserialize::Deserialize,
    deserializer::Deserializer,
    string::{StringWithLength, WStringWithLength},
    time::Time,
    version::Version,
};

#[derive(Default)]
pub struct RevisionHistory {
    created_by: String,
    last_edited_by: String,
    create_time: Time,
    last_edit_time: Time,
    revision_count: i32,
}

impl<D> Deserialize<'_, D> for RevisionHistory
where
    D: Deserializer,
{
    type Error = String;

    fn deserialize(deserializer: &mut D) -> Result<Self, Self::Error> {
        let mut revision_history = RevisionHistory::default();
        if Version::V1 == deserializer.version() {
            revision_history.created_by = StringWithLength::deserialize(deserializer)?.into();
            revision_history.create_time = Time::deserialize(deserializer)?;
            i32::deserialize(deserializer)?;
            revision_history.last_edited_by = StringWithLength::deserialize(deserializer)?.into();
            revision_history.last_edit_time = Time::deserialize(deserializer)?;
            i32::deserialize(deserializer)?;
            revision_history.revision_count = i32::deserialize(deserializer)?;
        } else {
            let chunk_version = chunk::Version::deserialize(deserializer)?;
            if 1u8 == chunk_version.major() {
                revision_history.created_by = WStringWithLength::deserialize(deserializer)?.into();
                revision_history.create_time = Time::deserialize(deserializer)?;
                revision_history.last_edited_by =
                    WStringWithLength::deserialize(deserializer)?.into();
                revision_history.last_edit_time = Time::deserialize(deserializer)?;
                revision_history.revision_count = i32::deserialize(deserializer)?;
            }
        }
        Ok(revision_history)
    }
}
