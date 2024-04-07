use super::Field;
use crate::nrrd::Nrrd;
use std::io::{BufWriter, Write};

pub fn write_nrrd<T: Write>(nrrd: &Nrrd, writer: T) -> Result<(), std::io::Error> {
    let mut buf_writer = BufWriter::new(writer);

    // Write NRRD version
    writeln!(buf_writer, "NRRD0005")?;

    // Write fields in a specific order
    let mut ordered_fields = nrrd.fields.iter().collect::<Vec<_>>();
    ordered_fields.sort_by_key(|f| field_order(f));

    for field in ordered_fields {
        writeln!(buf_writer, "{}: {}", field.identifier, field.descriptor)?;
    }

    // Write key-value pairs
    for key_value in &nrrd.key_values {
        writeln!(buf_writer, "{}:={}", key_value.key, key_value.value)?;
    }

    // Empty line between header and buffer
    writeln!(buf_writer)?;

    // Write pixel data
    buf_writer.write_all(&nrrd.buffer)?;

    Ok(())
}

fn field_order(f: &Field) -> usize {
    match f.identifier.as_str() {
        "type" => 0,
        "dimension" => 1,
        "sizes" => 2,
        _ => usize::MAX,
    }
}
