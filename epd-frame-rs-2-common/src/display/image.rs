use crate::display::color::E6Color;
use defmt_or_log::{info, warn};
use embedded_graphics::Pixel;
use embedded_graphics::prelude::{DrawTarget, Point};
use zerocopy::{FromBytes, Immutable, KnownLayout};

#[derive(FromBytes, KnownLayout, Immutable)]
#[repr(C, packed)]
pub struct E6Image {
    version: u8,
    width: u16,
    height: u16,
    _reserved: u8,
    nibbles: [u8],
}

impl<'a> IntoIterator for &'a E6Image {
    type Item = Pixel<E6Color>;
    type IntoIter = E6ImageIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        E6ImageIterator::new(self)
    }
}

impl E6Image {
    pub fn draw<E>(
        &self,
        point: Point,
        draw_target: &mut impl DrawTarget<Color = E6Color, Error = E>,
    ) -> Result<(), E> {
        draw_target.draw_iter(self.into_iter().map(|Pixel(p, c)| Pixel(p + point, c)))
    }
}

pub struct E6ImageIterator<'a> {
    image: &'a E6Image,
    pixel_index: usize,
    array_index: usize,
    pixels_count: usize,
    nibbles_count: usize,
}

impl<'a> Iterator for E6ImageIterator<'a> {
    type Item = Pixel<E6Color>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.array_index_in_range() && self.pixel_index_in_range() {
            let mut val = read_nibble(&self.image.nibbles, self.array_index);
            self.array_index += 1;

            while val & 0b00001000 > 0 {
                // nibble with high 1 are markers of transparent section
                let val2 = read_nibble(&self.image.nibbles, self.array_index);
                self.array_index += 1;
                let skip = ((val & 0b00000111) << 4) + val2;
                self.pixel_index += skip as usize;
                if self.array_index_in_range() && self.pixel_index_in_range() {
                    val = read_nibble(&self.image.nibbles, self.array_index);
                    self.array_index += 1;
                } else {
                    return None;
                }
            }

            if let Some(color) = E6Color::from_u8(val) {
                let pixel = Pixel(
                    Point::new(
                        self.pixel_index as i32 % self.image.width as i32,
                        self.pixel_index as i32 / self.image.width as i32
                            - self.image.height as i32,
                    ),
                    color,
                );
                self.pixel_index += 1;
                Some(pixel)
            } else {
                warn!("Unknown pixel {} color {}", self.pixel_index, val);
                self.pixel_index += 1;
                None
            }
        } else {
            None
        }
    }
}

impl<'a> E6ImageIterator<'a> {
    fn new(image: &'a E6Image) -> Self {
        Self {
            image,
            pixel_index: 0,
            array_index: 0,
            pixels_count: image.width as usize * image.height as usize,
            nibbles_count: image.nibbles.len() * 2,
        }
    }

    fn array_index_in_range(&self) -> bool {
        self.array_index < self.nibbles_count
    }

    fn pixel_index_in_range(&self) -> bool {
        self.pixel_index < self.pixels_count
    }
}

fn read_nibble(data: &[u8], index: usize) -> u8 {
    let pair = data[index / 2];
    let left = index % 2 == 0;
    if left { pair >> 4 } else { pair & 0x0F }
}

pub trait E6ImageSource {
    fn source_bytes(&self) -> &[u8];

    fn draw<E>(
        &self,
        point: Point,
        draw_target: &mut impl DrawTarget<Color = E6Color, Error = E>,
    ) -> Result<(), E> {
        E6Image::ref_from_bytes(self.source_bytes())
            .expect("Wrong image format!")
            .draw(point, draw_target)
    }
}
