pub mod reader;
pub mod writer;

use crate::{image::Image, pixel::PixelValue};
use std::{collections::HashSet, hash::Hash, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Version {
    Nrrd1,
    Nrrd2,
    Nrrd3,
    Nrrd4,
    Nrrd5,
}

/// <field>: <desc>
#[derive(Debug, Clone)]
pub struct Field {
    pub identifier: String, // Case-insensitive
    pub descriptor: String, // Whitespace at the end should be ignored
}

impl Hash for Field {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.identifier.hash(state);
    }
}

impl PartialEq for Field {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl Eq for Field {}

/// NNRD2 and above
/// <key>:=<value>
#[derive(Debug, Clone)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

impl Hash for KeyValue {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl PartialEq for KeyValue {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for KeyValue {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelType {
    Int8,
    UInt8,
    Int16,
    UInt16,
    Int32,
    UInt32,
    Int64,
    UInt64,
    Float32,
    Float64,
    Block(i32),
}

impl ToString for PixelType {
    fn to_string(&self) -> String {
        match self {
            PixelType::Int8 => "int8".to_string(),
            PixelType::UInt8 => "uint8".to_string(),
            PixelType::Int16 => "int16".to_string(),
            PixelType::UInt16 => "uint16".to_string(),
            PixelType::Int32 => "int32".to_string(),
            PixelType::UInt32 => "uint32".to_string(),
            PixelType::Int64 => "int64".to_string(),
            PixelType::UInt64 => "uint64".to_string(),
            PixelType::Float32 => "float".to_string(),
            PixelType::Float64 => "double".to_string(),
            PixelType::Block(_) => "block".to_string(),
        }
    }
}

impl FromStr for PixelType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "signed char" | "int8" | "int8_t" => Ok(Self::Int8),
            "uchar" | "unsigned char" | "uint8" | "uint8_t" => Ok(Self::UInt8),
            "short" | "short int" | "signed short" | "signed short int" | "int16" | "int16_t" => {
                Ok(Self::Int16)
            }
            "ushort" | "unsigned short" | "unsigned short int" | "uint16" | "uint16_t" => {
                Ok(Self::UInt16)
            }
            "int" | "signed int" | "int32" | "int32_t" => Ok(Self::Int32),
            "uint" | "unsigned int" | "uint32" | "uint32_t" => Ok(Self::UInt32),
            "longlong"
            | "long long"
            | "long long int"
            | "signed long long"
            | "signed long long int"
            | "int64"
            | "int64_t" => Ok(Self::Int64),
            "ulonglong"
            | "unsigned long long"
            | "unsigned long long int"
            | "uint64"
            | "uint64_t" => Ok(Self::UInt64),
            "float" => Ok(Self::Float32),
            "double" => Ok(Self::Float64),
            "block" => Ok(Self::Block(0)), // Placeholder block size
            _ => return Err(()),
        }
    }
}

impl PixelType {
    /// Returns the size of the pixel type in bytes
    pub fn size(self) -> usize {
        match self {
            PixelType::Int8 | PixelType::UInt8 => 1,
            PixelType::Int16 | PixelType::UInt16 => 2,
            PixelType::Int32 | PixelType::UInt32 | PixelType::Float32 => 4,
            PixelType::Int64 | PixelType::UInt64 | PixelType::Float64 => 8,
            PixelType::Block(size) => size as usize, // NRRD format accepts only positive block size so this should be safe
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Encoding {
    Raw,
    Ascii,
    GZip,
    BZip2,
    Other(String),
}

#[derive(Debug, Clone, Copy)]
pub enum Endian {
    Little,
    Big,
}

#[derive(Debug, Clone)]
pub struct Nrrd {
    version: Version,
    fields: HashSet<Field>,
    key_values: HashSet<KeyValue>,

    dimension: i32,
    sizes: Vec<i32>,
    pixel_type: PixelType,
    encoding: Encoding,
    endian: Endian,

    buffer: Vec<u8>,
}

impl Nrrd {
    #[inline]
    pub fn buffer(&self) -> &Vec<u8> {
        &self.buffer
    }

    #[inline]
    pub fn dimension(&self) -> i32 {
        self.dimension
    }

    #[inline]
    pub fn pixel_type(&self) -> PixelType {
        self.pixel_type
    }

    #[inline]
    pub fn sizes(&self) -> &[i32] {
        &self.sizes
    }

    #[inline]
    pub fn endian(&self) -> Endian {
        self.endian
    }

    #[inline]
    pub fn encoding(&self) -> &Encoding {
        &self.encoding
    }

    #[inline]
    pub fn fields(&self) -> &HashSet<Field> {
        &self.fields
    }

    #[inline]
    pub fn key_values(&self) -> &HashSet<KeyValue> {
        &self.key_values
    }

    #[inline]
    pub fn version(&self) -> Version {
        self.version
    }
}


impl<T: PixelValue, const D: usize> From<&Image<T, D>> for Nrrd {
    fn from(image: &Image<T, D>) -> Self {
        let pixel_size = T::pixel_type().size();
        let mut buffer = vec![Default::default(); image.pixels_count() * pixel_size];
        let endian = Endian::Little;

        let mut offset = 0;
        for pixel in image.pixels() {
            pixel.to_bytes(&mut buffer[offset..offset + pixel_size], endian);
            offset += pixel_size;
        }

        Nrrd {
            endian,
            buffer,
            dimension: D as i32,
            sizes: image.sizes().iter().map(|&x| x as i32).collect(),
            pixel_type: T::pixel_type(),
            encoding: Encoding::Raw,
            version: Version::Nrrd5,
            fields: [
                Field {
                    identifier: "type".to_string(),
                    descriptor: T::pixel_type().to_string(),
                },
                Field {
                    identifier: "dimension".to_string(),
                    descriptor: D.to_string(),
                },
                Field {
                    identifier: "sizes".to_string(),
                    descriptor: image
                        .sizes()
                        .iter()
                        .map(|&x| x.to_string())
                        .collect::<Vec<String>>()
                        .join(" "),
                },
                Field {
                    identifier: "endian".to_string(),
                    descriptor: match endian {
                        Endian::Little => "little".to_string(),
                        Endian::Big => "big".to_string(),
                    },
                },
                Field {
                    identifier: "encoding".to_string(),
                    descriptor: "raw".to_string(),
                },
            ]
            .into_iter()
            .collect(),
            key_values: HashSet::new(),
        }
    }
}
