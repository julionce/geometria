use std::io::Read;

pub trait NumberReader {
    fn read_i8(&mut self) -> std::io::Result<i8>;
    fn read_i16(&mut self) -> std::io::Result<i16>;
    fn read_i32(&mut self) -> std::io::Result<i32>;
    fn read_i64(&mut self) -> std::io::Result<i64>;
    fn read_i128(&mut self) -> std::io::Result<i128>;

    fn read_u8(&mut self) -> std::io::Result<u8>;
    fn read_u16(&mut self) -> std::io::Result<u16>;
    fn read_u32(&mut self) -> std::io::Result<u32>;
    fn read_u64(&mut self) -> std::io::Result<u64>;
    fn read_u128(&mut self) -> std::io::Result<u128>;

    fn read_f32(&mut self) -> std::io::Result<f32>;
    fn read_f64(&mut self) -> std::io::Result<f64>;
}

pub struct BigEndianNumberReader<T>
where
    T: Read,
{
    pub source: T,
}

pub struct LittleEndianNumberReader<T>
where
    T: Read,
{
    pub source: T,
}

macro_rules! impl_read_number_in_endian {
    ($primitive: ty, $method: ident, $from: ident) => {
        fn $method(&mut self) -> std::io::Result<$primitive> {
            let mut buf = [0u8; std::mem::size_of::<$primitive>()];
            match self.source.read_exact(&mut buf) {
                Ok(()) => Ok(<$primitive>::$from(buf)),
                Err(e) => Err(e),
            }
        }
    };
}

impl<T> NumberReader for BigEndianNumberReader<T>
where
    T: Read,
{
    impl_read_number_in_endian! {i8, read_i8, from_be_bytes}
    impl_read_number_in_endian! {i16, read_i16, from_be_bytes}
    impl_read_number_in_endian! {i32, read_i32, from_be_bytes}
    impl_read_number_in_endian! {i64, read_i64, from_be_bytes}
    impl_read_number_in_endian! {i128, read_i128, from_be_bytes}

    impl_read_number_in_endian! {u8, read_u8, from_be_bytes}
    impl_read_number_in_endian! {u16, read_u16, from_be_bytes}
    impl_read_number_in_endian! {u32, read_u32, from_be_bytes}
    impl_read_number_in_endian! {u64, read_u64, from_be_bytes}
    impl_read_number_in_endian! {u128, read_u128, from_be_bytes}

    impl_read_number_in_endian! {f32, read_f32, from_be_bytes}
    impl_read_number_in_endian! {f64, read_f64, from_be_bytes}
}

impl<T> NumberReader for LittleEndianNumberReader<T>
where
    T: Read,
{
    impl_read_number_in_endian! {i8, read_i8, from_le_bytes}
    impl_read_number_in_endian! {i16, read_i16, from_le_bytes}
    impl_read_number_in_endian! {i32, read_i32, from_le_bytes}
    impl_read_number_in_endian! {i64, read_i64, from_le_bytes}
    impl_read_number_in_endian! {i128, read_i128, from_le_bytes}

    impl_read_number_in_endian! {u8, read_u8, from_le_bytes}
    impl_read_number_in_endian! {u16, read_u16, from_le_bytes}
    impl_read_number_in_endian! {u32, read_u32, from_le_bytes}
    impl_read_number_in_endian! {u64, read_u64, from_le_bytes}
    impl_read_number_in_endian! {u128, read_u128, from_le_bytes}

    impl_read_number_in_endian! {f32, read_f32, from_le_bytes}
    impl_read_number_in_endian! {f64, read_f64, from_le_bytes}
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::{BigEndianNumberReader, LittleEndianNumberReader, NumberReader};

    //TODO: use concat_idents! macro to simplify the tests
    macro_rules! generate_read_in_be_test {
        ($test_name: ident, $method: ident, $value: expr) => {
            #[test]
            fn $test_name() {
                let data = $value.to_be_bytes();
                let mut reader = BigEndianNumberReader {
                    source: Cursor::new(data),
                };
                assert_eq!($value, reader.$method().unwrap());
            }
        };
    }

    macro_rules! generate_read_in_le_test {
        ($test_name: ident, $method: ident, $value: expr) => {
            #[test]
            fn $test_name() {
                let data = $value.to_le_bytes();
                let mut reader = LittleEndianNumberReader {
                    source: Cursor::new(data),
                };
                assert_eq!($value, reader.$method().unwrap());
            }
        };
    }

    generate_read_in_be_test! {read_i8_ram_val_in_be, read_i8, 11i8}
    generate_read_in_be_test! {read_i8_max_val_in_be, read_i8, i8::MAX}
    generate_read_in_be_test! {read_i8_min_val_in_be, read_i8, i8::MIN}

    generate_read_in_be_test! {read_i16_ram_val_in_be, read_i16, 11i16}
    generate_read_in_be_test! {read_i16_max_val_in_be, read_i16, i16::MAX}
    generate_read_in_be_test! {read_i16_min_val_in_be, read_i16, i16::MIN}

    generate_read_in_be_test! {read_i32_ram_val_in_be, read_i32, 11i32}
    generate_read_in_be_test! {read_i32_max_val_in_be, read_i32, i32::MAX}
    generate_read_in_be_test! {read_i32_min_val_in_be, read_i32, i32::MIN}

    generate_read_in_be_test! {read_i64_ram_val_in_be, read_i64, 11i64}
    generate_read_in_be_test! {read_i64_max_val_in_be, read_i64, i64::MAX}
    generate_read_in_be_test! {read_i64_min_val_in_be, read_i64, i64::MIN}

    generate_read_in_be_test! {read_i128_ram_val_in_be, read_i128, 11i128}
    generate_read_in_be_test! {read_i128_max_val_in_be, read_i128, i128::MAX}
    generate_read_in_be_test! {read_i128_min_val_in_be, read_i128, i128::MIN}

    generate_read_in_be_test! {read_u8_ram_val_in_be, read_u8, 11u8}
    generate_read_in_be_test! {read_u8_max_val_in_be, read_u8, u8::MAX}
    generate_read_in_be_test! {read_u8_min_val_in_be, read_u8, u8::MIN}

    generate_read_in_be_test! {read_u16_ram_val_in_be, read_u16, 11u16}
    generate_read_in_be_test! {read_u16_max_val_in_be, read_u16, u16::MAX}
    generate_read_in_be_test! {read_u16_min_val_in_be, read_u16, u16::MIN}

    generate_read_in_be_test! {read_u32_ram_val_in_be, read_u32, 11u32}
    generate_read_in_be_test! {read_u32_max_val_in_be, read_u32, u32::MAX}
    generate_read_in_be_test! {read_u32_min_val_in_be, read_u32, u32::MIN}

    generate_read_in_be_test! {read_u64_ram_val_in_be, read_u64, 11u64}
    generate_read_in_be_test! {read_u64_max_val_in_be, read_u64, u64::MAX}
    generate_read_in_be_test! {read_u64_min_val_in_be, read_u64, u64::MIN}

    generate_read_in_be_test! {read_u128_ram_val_in_be, read_u128, 11u128}
    generate_read_in_be_test! {read_u128_max_val_in_be, read_u128, u128::MAX}
    generate_read_in_be_test! {read_u128_min_val_in_be, read_u128, u128::MIN}

    generate_read_in_be_test! {read_f32_ram_val_in_be, read_f32, 11.0f32}
    generate_read_in_be_test! {read_f32_max_val_in_be, read_f32, f32::MAX}
    generate_read_in_be_test! {read_f32_min_val_in_be, read_f32, f32::MIN}

    generate_read_in_be_test! {read_f64_ram_val_in_be, read_f64, 11.0f64}
    generate_read_in_be_test! {read_f64_max_val_in_be, read_f64, f64::MAX}
    generate_read_in_be_test! {read_f64_min_val_in_be, read_f64, f64::MIN}

    generate_read_in_le_test! {read_i8_ram_val_in_le, read_i8, 11i8}
    generate_read_in_le_test! {read_i8_max_val_in_le, read_i8, i8::MAX}
    generate_read_in_le_test! {read_i8_min_val_in_le, read_i8, i8::MIN}

    generate_read_in_le_test! {read_i16_ram_val_in_le, read_i16, 11i16}
    generate_read_in_le_test! {read_i16_max_val_in_le, read_i16, i16::MAX}
    generate_read_in_le_test! {read_i16_min_val_in_le, read_i16, i16::MIN}

    generate_read_in_le_test! {read_i32_ram_val_in_le, read_i32, 11i32}
    generate_read_in_le_test! {read_i32_max_val_in_le, read_i32, i32::MAX}
    generate_read_in_le_test! {read_i32_min_val_in_le, read_i32, i32::MIN}

    generate_read_in_le_test! {read_i64_ram_val_in_le, read_i64, 11i64}
    generate_read_in_le_test! {read_i64_max_val_in_le, read_i64, i64::MAX}
    generate_read_in_le_test! {read_i64_min_val_in_le, read_i64, i64::MIN}

    generate_read_in_le_test! {read_i128_ram_val_in_le, read_i128, 11i128}
    generate_read_in_le_test! {read_i128_max_val_in_le, read_i128, i128::MAX}
    generate_read_in_le_test! {read_i128_min_val_in_le, read_i128, i128::MIN}

    generate_read_in_le_test! {read_u8_ram_val_in_le, read_u8, 11u8}
    generate_read_in_le_test! {read_u8_max_val_in_le, read_u8, u8::MAX}
    generate_read_in_le_test! {read_u8_min_val_in_le, read_u8, u8::MIN}

    generate_read_in_le_test! {read_u16_ram_val_in_le, read_u16, 11u16}
    generate_read_in_le_test! {read_u16_max_val_in_le, read_u16, u16::MAX}
    generate_read_in_le_test! {read_u16_min_val_in_le, read_u16, u16::MIN}

    generate_read_in_le_test! {read_u32_ram_val_in_le, read_u32, 11u32}
    generate_read_in_le_test! {read_u32_max_val_in_le, read_u32, u32::MAX}
    generate_read_in_le_test! {read_u32_min_val_in_le, read_u32, u32::MIN}

    generate_read_in_le_test! {read_u64_ram_val_in_le, read_u64, 11u64}
    generate_read_in_le_test! {read_u64_max_val_in_le, read_u64, u64::MAX}
    generate_read_in_le_test! {read_u64_min_val_in_le, read_u64, u64::MIN}

    generate_read_in_le_test! {read_u128_ram_val_in_le, read_u128, 11u128}
    generate_read_in_le_test! {read_u128_max_val_in_le, read_u128, u128::MAX}
    generate_read_in_le_test! {read_u128_min_val_in_le, read_u128, u128::MIN}

    generate_read_in_le_test! {read_f32_ram_val_in_le, read_f32, 11.0f32}
    generate_read_in_le_test! {read_f32_max_val_in_le, read_f32, f32::MAX}
    generate_read_in_le_test! {read_f32_min_val_in_le, read_f32, f32::MIN}

    generate_read_in_le_test! {read_f64_ram_val_in_le, read_f64, 11.0f64}
    generate_read_in_le_test! {read_f64_max_val_in_le, read_f64, f64::MAX}
    generate_read_in_le_test! {read_f64_min_val_in_le, read_f64, f64::MIN}
}
