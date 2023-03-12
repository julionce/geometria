use std::io::Read;

use super::deserializer::Deserializer;

trait Deserialize
where
    Self: Sized,
{
    type Error;

    fn deserialize<D>(deserializer: &mut D) -> Result<Self, Self::Error>
    where
        D: Deserializer;
}

macro_rules! impl_deserialize_for_number {
    ($type: ty, $method: ident) => {
        impl Deserialize for $type
        where
            Self: Sized,
        {
            type Error = String;

            fn deserialize<D>(deserializer: &mut D) -> Result<Self, Self::Error>
            where
                D: Deserializer,
            {
                match deserializer.$method() {
                    Ok(v) => Ok(v),
                    Err(e) => Err(e.to_string()),
                }
            }
        }
    };
}

impl_deserialize_for_number! {i8, read_i8}
impl_deserialize_for_number! {i16, read_i16}
impl_deserialize_for_number! {i32, read_i32}
impl_deserialize_for_number! {i64, read_i64}
impl_deserialize_for_number! {i128, read_i128}
impl_deserialize_for_number! {u8, read_u8}
impl_deserialize_for_number! {u16, read_u16}
impl_deserialize_for_number! {u32, read_u32}
impl_deserialize_for_number! {u64, read_u64}
impl_deserialize_for_number! {u128, read_u128}
impl_deserialize_for_number! {f32, read_f32}
impl_deserialize_for_number! {f64, read_f64}

impl Deserialize for String
where
    Self: Sized,
{
    type Error = String;

    fn deserialize<D>(deserializer: &mut D) -> Result<Self, Self::Error>
    where
        D: Deserializer,
    {
        let length = i32::deserialize(deserializer)?;
        if 0 > length {
            Err("invalid string length".to_string())
        } else {
            let mut string = String::new();
            match deserializer.take(length as u64).read_to_string(&mut string) {
                Ok(size) => {
                    if size as u64 == length as u64 {
                        Ok(string)
                    } else {
                        Err("invalid string length".to_string())
                    }
                }
                Err(e) => Err(format!("{}", e)),
            }
        }
    }
}

impl<T> Deserialize for Vec<T>
where
    T: Deserialize,
    String: From<<T as Deserialize>::Error>,
{
    type Error = String;

    fn deserialize<D>(deserializer: &mut D) -> Result<Self, Self::Error>
    where
        D: Deserializer,
    {
        let length = i32::deserialize(deserializer)?;
        if 0 > length {
            Err("invalid vector length".to_string())
        } else {
            let mut vector: Vec<T> = vec![];
            for _ in 0..length {
                vector.push(T::deserialize(deserializer)?);
            }
            Ok(vector)
        }
    }
}

impl<T, const N: usize> Deserialize for [T; N]
where
    T: Deserialize + Default + Copy,
    String: From<<T as Deserialize>::Error>,
{
    type Error = String;

    fn deserialize<D>(deserializer: &mut D) -> Result<Self, Self::Error>
    where
        D: Deserializer,
    {
        let mut rv: Self = [T::default(); N];
        for i in 0..N {
            rv[i] = T::deserialize(deserializer)?;
        }
        Ok(rv)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::common::reader::{BigEndianNumberReader, LittleEndianNumberReader};

    use super::*;

    #[test]
    fn deserialize_u8() {
        let data = 11u8.to_le_bytes();
        let mut reader = BigEndianNumberReader {
            source: Cursor::new(data),
        };
        assert_eq!(11u8, u8::deserialize(&mut reader).unwrap());
    }

    macro_rules! generate_deserialize_in_be_test {
        ($test_name: ident, $type: ty, $value: expr) => {
            #[test]
            fn $test_name() {
                let data = $value.to_be_bytes();
                let mut deserializer = BigEndianNumberReader {
                    source: Cursor::new(data),
                };
                assert_eq!($value, <$type>::deserialize(&mut deserializer).unwrap());
            }
        };
    }

    macro_rules! generate_deserialize_in_le_test {
        ($test_name: ident, $type: ty, $value: expr) => {
            #[test]
            fn $test_name() {
                let data = $value.to_le_bytes();
                let mut deserializer = LittleEndianNumberReader {
                    source: Cursor::new(data),
                };
                assert_eq!($value, <$type>::deserialize(&mut deserializer).unwrap());
            }
        };
    }

    generate_deserialize_in_be_test! {deserialize_i8_ram_val_in_be, i8, 11i8}
    generate_deserialize_in_be_test! {deserialize_i8_max_val_in_be, i8, i8::MAX}
    generate_deserialize_in_be_test! {deserialize_i8_min_val_in_be, i8, i8::MIN}

    generate_deserialize_in_be_test! {deserialize_i16_ram_val_in_be, i16, 11i16}
    generate_deserialize_in_be_test! {deserialize_i16_max_val_in_be, i16, i16::MAX}
    generate_deserialize_in_be_test! {deserialize_i16_min_val_in_be, i16, i16::MIN}

    generate_deserialize_in_be_test! {deserialize_i32_ram_val_in_be, i32, 11i32}
    generate_deserialize_in_be_test! {deserialize_i32_max_val_in_be, i32, i32::MAX}
    generate_deserialize_in_be_test! {deserialize_i32_min_val_in_be, i32, i32::MIN}

    generate_deserialize_in_be_test! {deserialize_i64_ram_val_in_be, i64, 11i64}
    generate_deserialize_in_be_test! {deserialize_i64_max_val_in_be, i64, i64::MAX}
    generate_deserialize_in_be_test! {deserialize_i64_min_val_in_be, i64, i64::MIN}

    generate_deserialize_in_be_test! {deserialize_i128_ram_val_in_be, i128, 11i128}
    generate_deserialize_in_be_test! {deserialize_i128_max_val_in_be, i128, i128::MAX}
    generate_deserialize_in_be_test! {deserialize_i128_min_val_in_be, i128, i128::MIN}

    generate_deserialize_in_be_test! {deserialize_u8_ram_val_in_be, u8, 11u8}
    generate_deserialize_in_be_test! {deserialize_u8_max_val_in_be, u8, u8::MAX}
    generate_deserialize_in_be_test! {deserialize_u8_min_val_in_be, u8, u8::MIN}

    generate_deserialize_in_be_test! {deserialize_u16_ram_val_in_be, u16, 11u16}
    generate_deserialize_in_be_test! {deserialize_u16_max_val_in_be, u16, u16::MAX}
    generate_deserialize_in_be_test! {deserialize_u16_min_val_in_be, u16, u16::MIN}

    generate_deserialize_in_be_test! {deserialize_u32_ram_val_in_be, u32, 11u32}
    generate_deserialize_in_be_test! {deserialize_u32_max_val_in_be, u32, u32::MAX}
    generate_deserialize_in_be_test! {deserialize_u32_min_val_in_be, u32, u32::MIN}

    generate_deserialize_in_be_test! {deserialize_u64_ram_val_in_be, u64, 11u64}
    generate_deserialize_in_be_test! {deserialize_u64_max_val_in_be, u64, u64::MAX}
    generate_deserialize_in_be_test! {deserialize_u64_min_val_in_be, u64, u64::MIN}

    generate_deserialize_in_be_test! {deserialize_u128_ram_val_in_be, u128, 11u128}
    generate_deserialize_in_be_test! {deserialize_u128_max_val_in_be, u128, u128::MAX}
    generate_deserialize_in_be_test! {deserialize_u128_min_val_in_be, u128, u128::MIN}

    generate_deserialize_in_be_test! {deserialize_f32_ram_val_in_be, f32, 11f32}
    generate_deserialize_in_be_test! {deserialize_f32_max_val_in_be, f32, f32::MAX}
    generate_deserialize_in_be_test! {deserialize_f32_min_val_in_be, f32, f32::MIN}

    generate_deserialize_in_be_test! {deserialize_f64_ram_val_in_be, f64, 11f64}
    generate_deserialize_in_be_test! {deserialize_f64_max_val_in_be, f64, f64::MAX}
    generate_deserialize_in_be_test! {deserialize_f64_min_val_in_be, f64, f64::MIN}

    generate_deserialize_in_le_test! {deserialize_i8_ram_val_in_le, i8, 11i8}
    generate_deserialize_in_le_test! {deserialize_i8_max_val_in_le, i8, i8::MAX}
    generate_deserialize_in_le_test! {deserialize_i8_min_val_in_le, i8, i8::MIN}

    generate_deserialize_in_le_test! {deserialize_i16_ram_val_in_le, i16, 11i16}
    generate_deserialize_in_le_test! {deserialize_i16_max_val_in_le, i16, i16::MAX}
    generate_deserialize_in_le_test! {deserialize_i16_min_val_in_le, i16, i16::MIN}

    generate_deserialize_in_le_test! {deserialize_i32_ram_val_in_le, i32, 11i32}
    generate_deserialize_in_le_test! {deserialize_i32_max_val_in_le, i32, i32::MAX}
    generate_deserialize_in_le_test! {deserialize_i32_min_val_in_le, i32, i32::MIN}

    generate_deserialize_in_le_test! {deserialize_i64_ram_val_in_le, i64, 11i64}
    generate_deserialize_in_le_test! {deserialize_i64_max_val_in_le, i64, i64::MAX}
    generate_deserialize_in_le_test! {deserialize_i64_min_val_in_le, i64, i64::MIN}

    generate_deserialize_in_le_test! {deserialize_i128_ram_val_in_le, i128, 11i128}
    generate_deserialize_in_le_test! {deserialize_i128_max_val_in_le, i128, i128::MAX}
    generate_deserialize_in_le_test! {deserialize_i128_min_val_in_le, i128, i128::MIN}

    generate_deserialize_in_le_test! {deserialize_u8_ram_val_in_le, u8, 11u8}
    generate_deserialize_in_le_test! {deserialize_u8_max_val_in_le, u8, u8::MAX}
    generate_deserialize_in_le_test! {deserialize_u8_min_val_in_le, u8, u8::MIN}

    generate_deserialize_in_le_test! {deserialize_u16_ram_val_in_le, u16, 11u16}
    generate_deserialize_in_le_test! {deserialize_u16_max_val_in_le, u16, u16::MAX}
    generate_deserialize_in_le_test! {deserialize_u16_min_val_in_le, u16, u16::MIN}

    generate_deserialize_in_le_test! {deserialize_u32_ram_val_in_le, u32, 11u32}
    generate_deserialize_in_le_test! {deserialize_u32_max_val_in_le, u32, u32::MAX}
    generate_deserialize_in_le_test! {deserialize_u32_min_val_in_le, u32, u32::MIN}

    generate_deserialize_in_le_test! {deserialize_u64_ram_val_in_le, u64, 11u64}
    generate_deserialize_in_le_test! {deserialize_u64_max_val_in_le, u64, u64::MAX}
    generate_deserialize_in_le_test! {deserialize_u64_min_val_in_le, u64, u64::MIN}

    generate_deserialize_in_le_test! {deserialize_u128_ram_val_in_le, u128, 11u128}
    generate_deserialize_in_le_test! {deserialize_u128_max_val_in_le, u128, u128::MAX}
    generate_deserialize_in_le_test! {deserialize_u128_min_val_in_le, u128, u128::MIN}

    generate_deserialize_in_le_test! {deserialize_f32_ram_val_in_le, f32, 11f32}
    generate_deserialize_in_le_test! {deserialize_f32_max_val_in_le, f32, f32::MAX}
    generate_deserialize_in_le_test! {deserialize_f32_min_val_in_le, f32, f32::MIN}

    generate_deserialize_in_le_test! {deserialize_f64_ram_val_in_le, f64, 11f64}
    generate_deserialize_in_le_test! {deserialize_f64_max_val_in_le, f64, f64::MAX}
    generate_deserialize_in_le_test! {deserialize_f64_min_val_in_le, f64, f64::MIN}
}
