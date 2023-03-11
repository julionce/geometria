use geometria_derive::RhinoDeserialize;

use std::io::{Seek, SeekFrom};

use super::{
    bool::BoolFromI32, chunk, chunk::Chunk, deserialize::Deserialize, deserializer::Deserializer,
    sequence::Sequence, string::WStringWithLength, typecode,
};

#[derive(Default, RhinoDeserialize)]
#[big_chunk_version(major > 1)]
#[normal_chunk]
pub struct PlugIn {}

type PlugInList = Sequence<PlugIn>;

#[derive(Default, RhinoDeserialize)]
pub struct UnitsAndTolerances {}

#[derive(Default, RhinoDeserialize)]
#[big_chunk_version(major == 1)]
pub struct MeshParameters {
    #[underlying_type(BoolFromI32)]
    pub compute_curvature: bool,
}

#[derive(Default, RhinoDeserialize)]
#[big_chunk_version(major == 1)]
pub struct Annotation {
    pub dim_scale: f64,
    pub text_height: f64,
    pub dim_exe: f64,
    pub dim_exo: f64,
    pub arrow_length: f64,
    pub arrow_width: f64,
    pub center_mark: f64,
    pub dim_units: i32,
    pub arrow_type: i32,
    pub angular_units: i32,
    pub length_format: i32,
    pub angle_format: i32,
    #[padding(i32)]
    pub resolution: i32,
    #[underlying_type(WStringWithLength)]
    pub face_name: String,
    #[big_chunk_version(minor > 0)]
    pub world_view_text_scale: f64,
    #[big_chunk_version(minor > 0)]
    pub enable_annotation_scaling: u8,
    #[big_chunk_version(minor > 1)]
    pub world_view_hatch_scale: f64,
    #[big_chunk_version(minor > 1)]
    pub enable_hatch_scaling: u8,
    #[big_chunk_version(minor > 2)]
    pub enable_model_space_annotation_scaling: u8,
    #[big_chunk_version(minor > 2)]
    pub enable_layout_space_annotation_scaling: u8,
}

#[derive(Default, RhinoDeserialize)]
#[big_chunk_version(major == 1)]
pub struct Attributes {
    pub line_type_display_scale: f64,
}

#[derive(Default, RhinoDeserialize)]
pub struct CurrentColor {
    pub color: i32,
    pub source: i32,
}

#[derive(Default, RhinoDeserialize)]
#[table(SETTINGS_TABLE)]
pub struct Settings {
    #[table_field(SETTINGS_PLUGINLIST)]
    pub plugin_list: PlugInList,
    #[table_field(SETTINGS_UNITSANDTOLS)]
    pub units_and_tolerances: UnitsAndTolerances,
    #[table_field(SETTINGS_RENDERMESH)]
    pub render_mesh: MeshParameters,
    #[table_field(SETTINGS_ANALYSISMESH)]
    pub analysis_mesh: MeshParameters,
    #[table_field(SETTINGS_ANNOTATION)]
    pub anotation: Annotation,
    #[table_field(SETTINGS_MODEL_URL)]
    #[underlying_type(WStringWithLength)]
    pub model_url: String,
    #[table_field(SETTINGS_ATTRIBUTES)]
    pub attributes: Attributes,
    #[table_field(SETTINGS_CURRENT_COLOR)]
    pub current_color: CurrentColor,
}
