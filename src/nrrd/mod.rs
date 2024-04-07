pub mod reader;
pub mod writer;

use std::{collections::HashSet, hash::Hash};

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
