use crate::nrrd::{Endian, PixelType};

pub trait PixelValue: Sized + Default + Clone {
    fn from_bytes(buffer: &[u8], endian: Endian) -> Self;
    fn to_bytes(&self, buffer: &mut [u8], endian: Endian);
    fn pixel_type() -> PixelType;
}

macro_rules! impl_pixel_value {
    ($type: ty, $pixel_type: expr) => {
        impl PixelValue for $type {
            fn from_bytes(buffer: &[u8], endian: Endian) -> Self {
                const SIZE: usize = std::mem::size_of::<$type>();
                let mut bytes = [0; SIZE];
                bytes.copy_from_slice(&buffer[..SIZE]);

                match endian {
                    Endian::Big => <$type>::from_be_bytes(bytes),
                    Endian::Little => <$type>::from_le_bytes(bytes),
                }
            }

            fn to_bytes(&self, buffer: &mut [u8], endian: Endian) {
                match endian {
                    Endian::Big => buffer.copy_from_slice(&self.to_be_bytes()),
                    Endian::Little => buffer.copy_from_slice(&self.to_le_bytes()),
                }
            }

            fn pixel_type() -> PixelType {
                $pixel_type
            }
        }
    };
}

impl_pixel_value!(i8, PixelType::Int8);
impl_pixel_value!(u8, PixelType::UInt8);
impl_pixel_value!(i16, PixelType::Int16);
impl_pixel_value!(u16, PixelType::UInt16);
impl_pixel_value!(i32, PixelType::Int32);
impl_pixel_value!(u32, PixelType::UInt32);
impl_pixel_value!(i64, PixelType::Int64);
impl_pixel_value!(u64, PixelType::UInt64);
impl_pixel_value!(f32, PixelType::Float32);
impl_pixel_value!(f64, PixelType::Float64);
