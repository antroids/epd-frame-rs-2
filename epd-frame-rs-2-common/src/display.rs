use crate::display::color::{BinaryColorAdapter, E6Color};
use crate::display::epd_spectra_6::FrameBuffer;
use crate::display::weather::Weather;
use crate::errors::DeviceError;
use crate::storage::{LastRunStatistics, LastRunStatus};
use alloc::format;
use alloc::vec::Vec;
use embedded_graphics::Pixel;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::{Dimensions, PixelColor};
use embedded_graphics::primitives::{
    CornerRadii, PrimitiveStyle, Rectangle, RoundedRectangle, StyledDrawable,
};
use embedded_graphics::text::Text;
use mplusfonts::BitmapFont;
use mplusfonts::style::BitmapFontStyle;
use mplusfonts_macros::mplus;

pub mod color;
pub mod epd_spectra_6;
pub mod image;

pub mod config_mode;
pub mod weather;

pub const DISPLAY_WIDTH: u16 = 800;
pub const DISPLAY_HEIGHT: u16 = 480;

pub static DEFAULT_FONT_12: BitmapFont<BinaryColor, 2> = mplus!(
    2,
    BOLD,
    cap_height(12),
    true,
    2,
    1,
    '0'..='9',
    'A'..='Z',
    'a'..='z',
    [" :,.()/%"]
);

pub struct CroppedDrawTarget<'a, D: DrawTarget>(pub &'a mut D, pub Rectangle);

impl<'a, D: DrawTarget> DrawTarget for CroppedDrawTarget<'a, D> {
    type Color = D::Color;
    type Error = D::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.0.draw_iter(
            pixels
                .into_iter()
                .filter(|Pixel(pos, _)| self.1.contains(*pos)),
        )
    }
}

impl<'a, D: DrawTarget> Dimensions for CroppedDrawTarget<'a, D> {
    fn bounding_box(&self) -> Rectangle {
        self.1
    }
}

pub async fn draw_weather(
    frame_buffer: &mut FrameBuffer<Vec<u8>>,
    weather: &Weather,
    rand: &mut fastrand::Rng,
) -> Result<(), DeviceError> {
    let forecast_frame_style = PrimitiveStyle::with_stroke(E6Color::Black, 1);

    let mut current_weather_target = CroppedDrawTarget(
        frame_buffer,
        Rectangle::new(Point::new(0, 0), Size::new(340, 180)),
    );
    weather
        .current
        .draw(Point::new(0, 0), &mut current_weather_target, rand)
        .await?;

    for (index, hourly) in weather.hourly.iter().enumerate() {
        let point = Point::new((110 + 5) * (index + 3) as i32, 0);
        RoundedRectangle::new(
            Rectangle::new(point, Size::new(110, 180)),
            CornerRadii::new((4, 4).into()),
        )
        .draw_styled(&forecast_frame_style, frame_buffer)?;
        hourly.draw(point, frame_buffer, rand).await?;
    }
    for (index, hourly) in weather.daily.iter().enumerate() {
        let point = Point::new((110 + 5) * index as i32, 185);
        RoundedRectangle::new(
            Rectangle::new(point, Size::new(110, 200)),
            CornerRadii::new((4, 4).into()),
        )
        .draw_styled(&forecast_frame_style, frame_buffer)?;
        hourly.draw(point, frame_buffer, rand).await?;
    }
    Ok(())
}

pub async fn draw_last_run_statistics(
    draw_target: &mut FrameBuffer<Vec<u8>>,
    last_run_statistics: &LastRunStatistics,
) -> Result<(), DeviceError> {
    if let LastRunStatus::Failed = last_run_statistics.status {
        let style = BitmapFontStyle::new(&DEFAULT_FONT_12, BinaryColor::On);
        let text = format!("Last run failed with: {:?}", last_run_statistics.failed_cause);
        BinaryColorAdapter::draw_transparent(
            E6Color::Red,
            &Text::new(text.as_str(), Point::new(5, 460), style),
            draw_target,
        )?;
    }
    Ok(())
}
