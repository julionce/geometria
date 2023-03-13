use geometria_derive::JtDeserialize;

use super::{deserialize::Deserialize, deserializer::Deserializer};

#[derive(Default, JtDeserialize)]
pub struct CoordF32(pub [f32; 3]);

#[derive(Default, JtDeserialize)]
pub struct DirF32(pub [f32; 3]);

#[derive(Default, JtDeserialize)]
pub struct BBoxF32 {
    pub min_corner: CoordF32,
    pub max_corner: CoordF32,
}

#[derive(Default, JtDeserialize)]
pub struct GUID(pub u32, pub [u16; 2], pub [u8; 8]);

pub struct MbString(pub String);

//TODO implement Deserialize trait for MbString

#[derive(Default, JtDeserialize)]
pub struct Mx4F32(pub [f32; 16]);

#[derive(Default, JtDeserialize)]
pub struct Mx4F64(pub [f64; 16]);

#[derive(Default, JtDeserialize)]
pub struct PlaneF32(pub [f32; 4]);

#[derive(Default, JtDeserialize)]
pub struct Quaternion(pub [f32; 4]);

#[derive(Default, JtDeserialize)]
pub struct RGB(pub [f32; 3]);

#[derive(Default, JtDeserialize)]
pub struct RGBA(pub [f32; 4]);
