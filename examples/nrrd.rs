use rusty_nrrd::{
    image::Image,
    nrrd::{writer::write_nrrd, Nrrd},
};
use std::fs::File;

fn main() {
    let in_file = File::open("foolf.nrrd").unwrap();
    let image: Image<f32, 2> = Image::try_read_nrrd(in_file).unwrap();

    let nrrd = Nrrd::from(&image);
    let _ = write_nrrd(&nrrd, File::create("test.nrrd").unwrap());
}
