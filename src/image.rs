use crate::{
    nrrd::{
        reader::{read_nrrd, ReadNrrdErr},
        Encoding, Nrrd,
    },
    pixel::PixelValue,
};
use std::{
    io::Read,
    ops::{Index, IndexMut},
};

pub struct Image<TPixel: PixelValue, const D: usize> {
    buffer: Vec<TPixel>,
    sizes: [usize; D],
}

impl<T: PixelValue, const D: usize> Image<T, D> {
    pub fn new(background: T, sizes: [usize; D]) -> Self {
        Self {
            sizes,
            buffer: vec![background; sizes.iter().product()],
        }
    }

    #[inline]
    pub fn pixels_count(&self) -> usize {
        self.sizes.iter().product()
    }

    #[inline]
    pub fn get(&self, index: &[usize; D]) -> &T {
        &self.buffer[self.offset(index)]
    }

    #[inline]
    pub fn get_mut(&mut self, index: &[usize; D]) -> &mut T {
        let offset = self.offset(index);
        &mut self.buffer[offset]
    }

    pub fn try_read_nrrd<TRead: Read>(reader: TRead) -> Result<Self, ImageFromNrrdErr> {
        let nrrd = read_nrrd(reader)?;
        Self::try_from(&nrrd)
    }

    #[inline]
    pub fn pixels(&self) -> &[T] {
        &self.buffer
    }

    #[inline]
    pub fn sizes(&self) -> &[usize; D] {
        &self.sizes
    }

    #[inline]
    fn offset(&self, index: &[usize; D]) -> usize {
        let mut offset = 0;
        let mut stride = 1;

        for i in 0..D {
            offset += index[i] * stride;
            stride *= self.sizes[i];
        }

        offset
    }
}

#[derive(Debug)]
pub enum ImageFromNrrdErr {
    DimensionsDoNotMatch,
    PixelTypesDoNotMatch,
    CannotReadNrrd(ReadNrrdErr),
    UnsupportedEncoding,
}

impl From<ReadNrrdErr> for ImageFromNrrdErr {
    #[inline]
    fn from(value: ReadNrrdErr) -> Self {
        Self::CannotReadNrrd(value)
    }
}

impl<T: PixelValue, const D: usize> TryFrom<&Nrrd> for Image<T, D> {
    type Error = ImageFromNrrdErr;

    fn try_from(nrrd: &Nrrd) -> Result<Self, Self::Error> {
        if nrrd.dimension() as usize != D {
            return Err(ImageFromNrrdErr::DimensionsDoNotMatch);
        }

        if T::pixel_type() != nrrd.pixel_type() {
            return Err(ImageFromNrrdErr::PixelTypesDoNotMatch);
        }

        if *nrrd.encoding() != Encoding::Raw {
            return Err(ImageFromNrrdErr::UnsupportedEncoding);
        }

        let mut sizes = [0; D];
        for i in 0..D {
            sizes[i] = nrrd.sizes()[i] as usize;
        }

        let pixels = sizes.iter().product();
        let mut buffer = vec![T::default(); pixels];

        let pixel_size = T::pixel_type().size();
        let mut offset = 0;

        for i in 0..pixels {
            buffer[i] = T::from_bytes(&nrrd.buffer()[offset..], nrrd.endian());
            offset += pixel_size;
        }

        Ok(Self { buffer, sizes })
    }
}

impl<T: PixelValue, const D: usize> Index<&[usize; D]> for Image<T, D> {
    type Output = T;

    #[inline]
    fn index(&self, index: &[usize; D]) -> &Self::Output {
        self.get(index)
    }
}

impl<T: PixelValue, const D: usize> Index<[usize; D]> for Image<T, D> {
    type Output = T;

    #[inline]
    fn index(&self, index: [usize; D]) -> &Self::Output {
        self.get(&index)
    }
}

impl<T: PixelValue, const D: usize> IndexMut<&[usize; D]> for Image<T, D> {
    #[inline]
    fn index_mut(&mut self, index: &[usize; D]) -> &mut Self::Output {
        self.get_mut(index)
    }
}

impl<T: PixelValue, const D: usize> IndexMut<[usize; D]> for Image<T, D> {
    fn index_mut(&mut self, index: [usize; D]) -> &mut Self::Output {
        self.get_mut(&index)
    }
}
