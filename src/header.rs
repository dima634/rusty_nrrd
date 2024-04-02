use std::{collections::HashSet, hash::Hash};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    pub identifier: String, // Case-sensitive
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

#[derive(Debug)]
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
    Block,
}

#[derive(Debug)]
pub enum Encoding {
    Raw,
    Ascii,
    GZip,
    BZip2,
    Other(String),
}

#[derive(Debug)]
pub struct Header {
    pub version: Version,
    pub fields: HashSet<Field>,
    pub key_values: HashSet<KeyValue>,

    pub dimension: i32,
    pub sizes: Vec<i32>,
    pub pixel_type: PixelType,
    pub encoding: Encoding,
}
