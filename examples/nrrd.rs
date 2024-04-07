use std::{fs::File, path::Path};

use rusty_nrrd::{reader::read_nrrd, writer::write_nrrd};

fn main() {
    let nrrd = read_nrrd(File::open("foolf.nrrd").unwrap()).unwrap();

    let _ = write_nrrd(&nrrd, File::create("test.nrrd").unwrap());
}