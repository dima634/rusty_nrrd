use crate::header::{Encoding, Endian, Field, Header, KeyValue, PixelType, Version};
use std::collections::HashSet;

pub enum ReadNrrdErr {
    DuplicateField(String),
    UnknownVersion(String),
    UnsupportedEncoding(String),
    Malformed(String),
}

fn read_header(header_text: String) -> Result<Header, ReadNrrdErr> {
    let mut lines = header_text.lines().enumerate();
    let (_, magic_line) = lines
        .next()
        .ok_or(ReadNrrdErr::Malformed("Empty header".to_string()))?;
    let version = try_read_magic(magic_line)?;

    let mut fields = HashSet::new();
    let mut key_values = HashSet::new();

    let mut required_fields = RequiredFields::default();

    while let Some((line_num, line)) = lines.next() {
        if line.starts_with('#') {
            // Comment
            continue;
        }

        if let Some(field) = try_read_field(line) {
            required_fields.parse(&field)?;
            let exist = fields.insert(field.clone());

            if exist {
                return Err(ReadNrrdErr::DuplicateField(format!(
                    "Duplicate field '{}' at line {}",
                    field.identifier, line_num
                )));
            }

            continue;
        }

        if version < Version::Nrrd2 {
            return Err(ReadNrrdErr::Malformed(format!(
                "Unexpected line at {}: '{}'",
                line_num, line
            )));
        }

        match try_read_key_value(line) {
            Some(kv) => key_values.insert(kv),
            None => {
                return Err(ReadNrrdErr::Malformed(format!(
                    "Unexpected line at {}: '{}'",
                    line_num, line
                )))
            }
        };
    }

    required_fields.validate().map(|required| Header {
        version,
        fields,
        key_values,
        dimension: required.dimension.unwrap(),
        sizes: required.sizes.unwrap(),
        pixel_type: required.pixel_type.unwrap(),
        encoding: required.encoding.unwrap(),
        endian: required.endian.unwrap_or(Endian::Little),
    })
}

#[derive(Debug, Default)]
struct RequiredFields {
    dimension: Option<i32>,
    sizes: Option<Vec<i32>>,
    pixel_type: Option<PixelType>,
    encoding: Option<Encoding>,
    block_size: Option<i32>,
    endian: Option<Endian>,
}

impl RequiredFields {
    fn parse(&mut self, field: &Field) -> Result<(), ReadNrrdErr> {
        match field.identifier.as_str() {
            "DIMENSION" => self.try_parse_dimension(field),
            "SIZES" => self.try_parse_sizes(field),
            "TYPE" => self.try_parse_type(field),
            "ENCODING" => self.try_parse_encoding(field),
            "BLOCK SIZE" | "BLOCKSIZE" => self.try_parse_block_size(field),
            "ENDIAN" => self.try_parse_endian(field),
            _ => Ok(()),
        }
    }

    fn try_parse_dimension(&mut self, field: &Field) -> Result<(), ReadNrrdErr> {
        let dimension = field.descriptor.parse().map_err(|_| {
            let err = format!("Invalid DIMENSION value");
            ReadNrrdErr::Malformed(err)
        })?;
        self.dimension = Some(dimension);
        Ok(())
    }

    fn try_parse_sizes(&mut self, field: &Field) -> Result<(), ReadNrrdErr> {
        let dimension = match self.dimension {
            Some(d) => d,
            None => {
                let err = format!("Per-axis specification before DIMENSION");
                return Err(ReadNrrdErr::Malformed(err));
            }
        };

        let all_sizes = field.descriptor.split_whitespace();
        let mut vec = Vec::new();

        for size in all_sizes {
            let num = size
                .parse()
                .map_err(|_| ReadNrrdErr::Malformed("Invalid SIZES value".to_string()))?;
            vec.push(num);
        }

        let vec_len: i32 = vec
            .len()
            .try_into()
            .map_err(|_| ReadNrrdErr::Malformed("Too many SIZES".to_string()))?;

        if vec_len != dimension {
            let err = format!("Mismatched DIMENSION and SIZES");
            return Err(ReadNrrdErr::Malformed(err));
        }

        self.sizes = Some(vec);
        Ok(())
    }

    fn try_parse_type(&mut self, field: &Field) -> Result<(), ReadNrrdErr> {
        let pixel_type = match field.descriptor.as_str() {
            "signed char" | "int8" | "int8_t" => PixelType::Int8,
            "uchar" | "unsigned char" | "uint8" | "uint8_t" => PixelType::UInt8,
            "short" | "short int" | "signed short" | "signed short int" | "int16" | "int16_t" => PixelType::Int16,
            "ushort" | "unsigned short" | "unsigned short int" | "uint16" | "uint16_t" => PixelType::UInt16,
            "int" | "signed int" | "int32" | "int32_t" => PixelType::Int32,
            "uint" | "unsigned int" | "uint32" | "uint32_t" => PixelType::UInt32,
            "longlong" | "long long" | "long long int" | "signed long long" | "signed long long int" | "int64" | "int64_t" => PixelType::Int64,
            "ulonglong" | "unsigned long long" | "unsigned long long int" | "uint64" | "uint64_t" => PixelType::UInt64,
            "float" => PixelType::Float32,
            "double" => PixelType::Float64,
            "block" => PixelType::Block(0), // Placeholder block size
            _ => return Err(ReadNrrdErr::Malformed("Invalid TYPE value".to_string())),
        };

        self.pixel_type = Some(pixel_type);
        Ok(())
    }

    fn try_parse_encoding(&mut self, field: &Field) -> Result<(), ReadNrrdErr> {
        let encoding = match field.descriptor.as_str() {
            "raw" => Encoding::Raw,
            "ascii" | "text" | "txt" => Encoding::Ascii,
            "gzip" | "gz" => Encoding::GZip,
            "bzip2" | "bz2" => Encoding::BZip2,
            _ => Encoding::Other(field.descriptor.clone()),
        };

        self.encoding = Some(encoding);
        Ok(())
    }

    fn try_parse_block_size(&mut self, field: &Field) -> Result<(), ReadNrrdErr> {
        let block_size = field.descriptor.parse().map_err(|_| {
            let err = format!("Invalid BLOCK SIZE value");
            ReadNrrdErr::Malformed(err)
        })?;
        self.block_size = Some(block_size);
        Ok(())
    }

    fn try_parse_endian(&mut self, field: &Field) -> Result<(), ReadNrrdErr> {
        let endian = match field.descriptor.as_str() {
            "little" => Endian::Little,
            "big" => Endian::Big,
            _ => return Err(ReadNrrdErr::Malformed("Invalid ENDIAN value".to_string())),
        };

        self.endian = Some(endian);
        Ok(())
    }

    fn validate(mut self) -> Result<Self, ReadNrrdErr> {
        if self.dimension.is_none() {
            return Err(ReadNrrdErr::Malformed("Missing DIMENSION field".to_string()));
        }

        if self.sizes.is_none() {
            return Err(ReadNrrdErr::Malformed("Missing SIZES field".to_string()));
        }

        match &mut self.pixel_type {
            Some(PixelType::Block(block_size)) => {
                // Block type NRRD should have a positive block size
                match self.block_size {
                    Some(size) if size > 0 => *block_size = size,
                    Some(_) => return Err(ReadNrrdErr::Malformed("Invalid BLOCK SIZE value".to_string())),
                    None => return Err(ReadNrrdErr::Malformed("Missing BLOCK SIZE field".to_string())),
                };
            },
            Some(_) => {
                // NRRD that has type which size is bigger than 1 byte should have endian
                match (self.endian, self.pixel_type) {
                    (None, Some(PixelType::Int8)) | (None, Some(PixelType::UInt8)) => (),
                    _ => return Err(ReadNrrdErr::Malformed("Missing ENDIAN field".to_string())),
                };
            },
            None => return Err(ReadNrrdErr::Malformed("Missing TYPE field".to_string())),
        };

        if self.encoding.is_none() {
            return Err(ReadNrrdErr::Malformed("Missing ENCODING field".to_string()));
        }

        Ok(self)
    }
}

fn try_read_magic(magic_line: &str) -> Result<Version, ReadNrrdErr> {
    match magic_line {
        "NRRD1" => Ok(Version::Nrrd1),
        "NRRD2" => Ok(Version::Nrrd2),
        "NRRD3" => Ok(Version::Nrrd3),
        "NRRD4" => Ok(Version::Nrrd4),
        "NRRD5" => Ok(Version::Nrrd5),
        _ => Err(ReadNrrdErr::UnknownVersion(format!(
            "Unknown NRRD version: {}",
            magic_line
        ))),
    }
}

fn try_read_field(line: &str) -> Option<Field> {
    let (ident, desc) = line.split_once(": ")?;
    let clean_ident = ident.to_uppercase();
    let clean_desc = desc.trim_end();

    Some(Field {
        identifier: clean_ident,
        descriptor: clean_desc.into(),
    })
}

fn try_read_key_value(line: &str) -> Option<KeyValue> {
    let (key, value) = line.split_once(":=")?;

    if key.is_empty() {
        return None;
    }

    Some(KeyValue {
        key: key.into(),
        value: value.into(),
    })
}
