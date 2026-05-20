use crate::display::color::{BinaryColorAdapter, E6Color};
use crate::display::epd_spectra_6::FrameBuffer;
use crate::display::image::E6ImageSource;
use crate::display::weather::Weather;
use crate::display::weather::frog::Frog130x180;
use crate::display::widgets::weather::{
    CurrentWeatherWidget, DailyWeatherWidget, HourlyWeatherWidget, IconValue16,
};
use crate::errors::DeviceError;
use crate::scheduler::HourlyScheduler;
use crate::storage::{LastRunStatistics, LastRunStatus};
use alloc::format;
use alloc::vec::Vec;
use defmt_or_log::derive_format_or_debug;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Dimensions;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::text::Text;
use embedded_graphics::{Drawable, Pixel};
use embedded_layout::View;
use embedded_layout::prelude::{Align, horizontal, vertical};
use mplusfonts::BitmapFont;
use mplusfonts::style::BitmapFontStyle;
use mplusfonts_macros::mplus;

pub mod color;
pub mod epd_spectra_6;
pub mod image;
pub mod widgets;

pub mod config_mode;
pub mod weather;

pub const DISPLAY_WIDTH: u16 = 800;
pub const DISPLAY_HEIGHT: u16 = 480;

pub type BinaryFontStyleType = BitmapFontStyle<'static, 'static, BinaryColor, BinaryColor, 2>;
pub const DEFAULT_FONT_12: BitmapFont<BinaryColor, 2> = mplus!(
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
pub const DEFAULT_FONT_12_STYLE: BinaryFontStyleType =
    BitmapFontStyle::new(&DEFAULT_FONT_12, BinaryColor::On);

pub const DEFAULT_FONT_10: BitmapFont<BinaryColor, 2> = mplus!(
    1,
    BOLD,
    cap_height(10),
    true,
    2,
    1,
    '0'..='9',
    'A'..='Z',
    'a'..='z',
    [" :,.()/%+-°"]
);

pub const DEFAULT_FONT_10_STYLE: BinaryFontStyleType =
    BitmapFontStyle::new(&DEFAULT_FONT_10, BinaryColor::On);

pub struct CroppedDrawTarget<'a, D: DrawTarget>(pub &'a mut D, pub Rectangle);

const HUMIDITY_16: &[u8] = include_bytes!("../resources/icons_16/humidity_16.e6spectra");
const WATER_16: &[u8] = include_bytes!("../resources/icons_16/water_16.e6spectra");
const TEMPERATURE_16: &[u8] = include_bytes!("../resources/icons_16/temperature_16.e6spectra");
const WIND_16: &[u8] = include_bytes!("../resources/icons_16/wind_16.e6spectra");
const UV_16: &[u8] = include_bytes!("../resources/icons_16/uv_16.e6spectra");
const WIFI_16: &[u8] = include_bytes!("../resources/icons_16/wifi_16.e6spectra");
const CLOCK_16: &[u8] = include_bytes!("../resources/icons_16/clock_16.e6spectra");

#[derive(Clone, Copy)]
#[derive_format_or_debug]
#[repr(u8)]
pub enum Icon16 {
    Temperature,
    Humidity,
    Water,
    Wind,
    Uv,
    Wifi,
    Clock,
}

impl E6ImageSource for Icon16 {
    fn source_bytes(&self) -> &[u8] {
        match self {
            Icon16::Temperature => TEMPERATURE_16,
            Icon16::Humidity => HUMIDITY_16,
            Icon16::Water => WATER_16,
            Icon16::Wind => WIND_16,
            Icon16::Uv => UV_16,
            Icon16::Wifi => WIFI_16,
            Icon16::Clock => CLOCK_16,
        }
    }

    fn size(&self) -> Size {
        (16, 16).into()
    }
}

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
    let mut current_weather_target = CroppedDrawTarget(
        frame_buffer,
        Rectangle::new(Point::new(0, 0), Size::new(340, 219)),
    );
    CurrentWeatherWidget::new(&weather.current).draw(&mut current_weather_target)?;

    for (index, hourly) in weather.hourly.iter().enumerate() {
        let point = Point::new((110 + 5) * (index + 3) as i32, 0);
        HourlyWeatherWidget::new(hourly, rand)
            .translate_mut(point)
            .draw(frame_buffer)?;
    }
    for (index, daily) in weather.daily.iter().enumerate() {
        let point = Point::new((110 + 5) * index as i32, 224);
        DailyWeatherWidget::new(daily, rand)
            .translate_mut(point)
            .draw(frame_buffer)?;
    }
    Ok(())
}

pub async fn draw_weather_error(
    frame_buffer: &mut FrameBuffer<Vec<u8>>,
    error: &DeviceError,
) -> Result<(), DeviceError> {
    let text = format!("Weather Error: {}", error);
    let boundaries = Rectangle::new(Point::new(0, 0), Size::new(800, 480));

    widgets::Icon::new(&Frog130x180::Thunder)
        .align_to(&boundaries, horizontal::Center, vertical::Top)
        .draw(frame_buffer)?;
    widgets::Text::new(text.as_str(), DEFAULT_FONT_10_STYLE, E6Color::Black)
        .align_to(&boundaries, horizontal::Center, vertical::Center)
        .draw(frame_buffer)?;

    Ok(())
}

pub async fn draw_last_run_statistics(
    draw_target: &mut FrameBuffer<Vec<u8>>,
    last_run_statistics: &LastRunStatistics,
) -> Result<(), DeviceError> {
    if let LastRunStatus::Failed = last_run_statistics.status {
        let text = format!(
            "Last run failed with: {:?}",
            last_run_statistics.failed_cause
        );
        BinaryColorAdapter::draw_transparent(
            E6Color::Red,
            &Text::new(text.as_str(), Point::new(5, 460), DEFAULT_FONT_12_STYLE),
            draw_target,
        )?;
    }
    Ok(())
}

pub async fn draw_status_bar(
    draw_target: &mut FrameBuffer<Vec<u8>>,
    weather: &Result<Weather, DeviceError>,
    task_scheduler: &HourlyScheduler,
    last_status: &LastRunStatistics,
) -> Result<(), DeviceError> {
    let frame = Rectangle::new((5, 464).into(), (800, 16).into());
    let (connection_status, connection_status_color) = match weather {
        Ok(_) => ("Online", E6Color::Black),
        Err(_) => ("Failed", E6Color::Red),
    };
    let last_status_text = match last_status.status {
        LastRunStatus::None => format!(
            "First run. Next run in: {} min",
            task_scheduler.minutes_delay
        ),
        LastRunStatus::Successful => format!(
            "Previous update was successful. Next run in: {} min",
            task_scheduler.minutes_delay
        ),
        LastRunStatus::Failed => format!(
            "Previous update failed: {:?}. Next run in: {} min",
            last_status.failed_cause, task_scheduler.minutes_delay
        ),
    }
    .leak();
    let connection_status = IconValue16::new(
        &Icon16::Wifi,
        connection_status,
        connection_status_color,
    )
    .align_to(&frame, horizontal::Left, vertical::Center);
    let last_update =
        IconValue16::new(&Icon16::Clock, last_status_text, E6Color::Black).translate(Point::new(
            connection_status
                .bounds()
                .bottom_right()
                .unwrap_or_default()
                .x
                + 5,
            464,
        ));

    connection_status.draw(draw_target)?;
    last_update.draw(draw_target)?;
    Ok(())
}
