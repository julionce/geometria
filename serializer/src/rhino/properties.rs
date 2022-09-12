use geometria_derive::Deserialize;
use std::io::{Seek, SeekFrom};

use super::{
    application::Application, chunk::Chunk, deserialize::Deserialize, deserializer::Deserializer,
    notes::Notes, on_version::Version as OnVersion, preview_image::CompressedPreviewImage,
    preview_image::PreviewImage, revision_history::RevisionHistory, string::WStringWithLength,
    typecode, version::Version,
};

#[derive(Default, Deserialize)]
#[table]
pub struct PropertiesV1 {
    #[table_field(COMMENTBLOCK)]
    comment: String,
    #[table_field(SUMMARY)]
    revision_history: RevisionHistory,
    #[table_field(NOTES)]
    notes: Notes,
    #[table_field(BITMAPPREVIEW)]
    preview_image: PreviewImage,
}

#[derive(Default, Deserialize)]
#[table(PROPERTIES_TABLE)]
pub struct PropertiesV2 {
    #[table_field(PROPERTIES_AS_FILE_NAME)]
    #[underlying_type(WStringWithLength)]
    filename: String,
    #[table_field(PROPERTIES_OPENNURBS_VERSION)]
    version: OnVersion,
    #[table_field(PROPERTIES_REVISIONHISTORY)]
    revision_history: RevisionHistory,
    #[table_field(PROPERTIES_NOTES)]
    notes: Notes,
    #[table_field(PROPERTIES_APPLICATION)]
    application: Application,
    #[table_field(PROPERTIES_PREVIEWIMAGE)]
    preview_image: PreviewImage,
    #[table_field(PROPERTIES_COMPRESSED_PREVIEWIMAGE)]
    compressed_preview_image: CompressedPreviewImage,
}

pub enum Properties {
    V1(PropertiesV1),
    V2(PropertiesV2),
}

impl Default for Properties {
    fn default() -> Self {
        Self::V1(PropertiesV1::default())
    }
}

impl<D> Deserialize<'_, D> for Properties
where
    D: Deserializer,
{
    type Error = String;

    fn deserialize(deserializer: &mut D) -> Result<Self, Self::Error> {
        let properties: Properties;
        if Version::V1 == deserializer.version() {
            deserializer.seek(SeekFrom::Start(32u64)).unwrap();
            properties = Properties::V1(PropertiesV1::deserialize(deserializer)?);
        } else {
            properties = Properties::V2(PropertiesV2::deserialize(deserializer)?);
        }
        Ok(properties)
    }
}
