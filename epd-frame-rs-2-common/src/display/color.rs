use defmt_or_log::derive_format_or_debug;
use embedded_graphics::pixelcolor::{BinaryColor, PixelColor, Rgb888, RgbColor};
use embedded_graphics::prelude::{Dimensions, DrawTarget};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::{Drawable, Pixel};
use mplusfonts::color::{Invert, Screen, WeightedAvg};

pub type DisplayRgbColor = (u8, u8, u8);

const E6_PALETTE: [DisplayRgbColor; 7] = {
    [
        (0, 0, 0),
        (255, 255, 255),
        (255, 255, 0),
        (255, 0, 0),
        (0, 0, 255),
        (0, 0, 255),
        (0, 255, 0),
    ]
};

#[derive(Copy, Clone, PartialOrd, PartialEq)]
#[derive_format_or_debug]
#[repr(u8)]
pub enum E6Color {
    Black = 0,
    White = 1,
    Yellow = 2,
    Red = 3,
    Blue = 5,
    Green = 6,
}

// Embedded Graphics Impl
impl PixelColor for E6Color {
    type Raw = ();
}

impl From<u8> for E6Color {
    fn from(value: u8) -> Self {
        match value {
            0 => E6Color::Black,
            1 => E6Color::White,
            2 => E6Color::Yellow,
            3 => E6Color::Red,
            5 => E6Color::Blue,
            6 => E6Color::Green,
            _ => panic!("Unknown E6 color index {}", value),
        }
    }
}

impl From<E6Color> for u8 {
    fn from(value: E6Color) -> Self {
        value as u8
    }
}

impl From<E6Color> for Rgb888 {
    fn from(value: E6Color) -> Self {
        let triplet = E6_PALETTE[value as usize];
        Self::new(triplet.0, triplet.1, triplet.2)
    }
}

impl From<Rgb888> for E6Color {
    fn from(value: Rgb888) -> Self {
        let color: DisplayRgbColor = (value.r(), value.g(), value.b()).into();
        for (index, c) in E6_PALETTE.iter().enumerate() {
            if color == *c {
                return E6Color::from(index as u8);
            }
        }
        panic!("Invalid E6Color: {:?}", color);
    }
}

impl From<BinaryColor> for E6Color {
    fn from(value: BinaryColor) -> Self {
        match value {
            BinaryColor::Off => E6Color::White,
            BinaryColor::On => E6Color::Black,
        }
    }
}

impl E6Color {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(E6Color::Black),
            1 => Some(E6Color::White),
            2 => Some(E6Color::Yellow),
            3 => Some(E6Color::Red),
            5 => Some(E6Color::Blue),
            6 => Some(E6Color::Green),
            _ => None,
        }
    }
}

impl Default for E6Color {
    fn default() -> Self {
        Self::Black
    }
}

impl Invert for E6Color {
    fn invert(self) -> Self {
        match self {
            E6Color::Black => E6Color::White,
            E6Color::White => E6Color::Black,
            E6Color::Yellow => E6Color::Blue,
            E6Color::Red => E6Color::Green,
            E6Color::Blue => E6Color::Yellow,
            E6Color::Green => E6Color::Red,
        }
    }
}

impl Screen for E6Color {
    fn screen(self, other: Self, start: Self, end: Self) -> Self {
        if self == end || other == end {
            end
        } else {
            start
        }
    }
}

impl WeightedAvg for E6Color {
    fn weighted_avg(
        self,
        other: Self,
        start: Self,
        end: Self,
        other_start: Self,
        other_end: Self,
    ) -> Self {
        if start == other_start {
            if self == end || other == other_end {
                end
            } else {
                start
            }
        } else {
            self
        }
    }
}

pub struct BinaryColorAdapter<'d, D: DrawTarget<Color = E6Color>> {
    on_color: Option<E6Color>,
    off_color: Option<E6Color>,
    draw_target: &'d mut D,
}

impl<'d, D: DrawTarget<Color = E6Color>> BinaryColorAdapter<'d, D> {
    pub fn new(
        on_color: Option<E6Color>,
        off_color: Option<E6Color>,
        draw_target: &'d mut D,
    ) -> Self {
        Self {
            on_color,
            off_color,
            draw_target,
        }
    }

    pub fn draw<DRAWABLE: Drawable<Color = BinaryColor>>(
        on_color: Option<E6Color>,
        off_color: Option<E6Color>,
        drawable: &'d DRAWABLE,
        draw_target: &'d mut D,
    ) -> Result<DRAWABLE::Output, D::Error> {
        drawable.draw(&mut Self::new(on_color, off_color, draw_target))
    }

    pub fn draw_transparent<DRAWABLE: Drawable<Color = BinaryColor>>(
        on_color: E6Color,
        drawable: &'d DRAWABLE,
        draw_target: &'d mut D,
    ) -> Result<DRAWABLE::Output, D::Error> {
        Self::draw(Some(on_color), None, drawable, draw_target)
    }
}

impl<'d, D: DrawTarget<Color = E6Color>> Dimensions for BinaryColorAdapter<'d, D> {
    fn bounding_box(&self) -> Rectangle {
        self.draw_target.bounding_box()
    }
}

impl<'d, D: DrawTarget<Color = E6Color>> DrawTarget for BinaryColorAdapter<'d, D> {
    type Color = BinaryColor;
    type Error = D::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let iter = pixels.into_iter().filter_map(|Pixel(point, color)| {
            let color = match color {
                BinaryColor::Off => self.off_color,
                BinaryColor::On => self.on_color,
            };
            color.map(|c| Pixel(point, c))
        });
        self.draw_target.draw_iter(iter)
    }
}
