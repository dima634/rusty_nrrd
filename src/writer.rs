use std::io::{BufWriter, Write};
use crate::nrrd::Nrrd;

pub fn write_nrrd<T: Write>(nrrd: &Nrrd, writer: T) -> Result<(), std::io::Error> {
    let mut buf_writer = BufWriter::new(writer);

    for field in &nrrd.fields {
        writeln!(buf_writer, "{}: {}", field.identifier, field.descriptor)?;
    }

    for key_value in &nrrd.key_values {
        writeln!(buf_writer, "{}:={}", key_value.key, key_value.value)?;
    }

    writeln!(buf_writer)?; // Empty line between header and buffer

    buf_writer.write_all(&nrrd.buffer)?;

    Ok(())
}
