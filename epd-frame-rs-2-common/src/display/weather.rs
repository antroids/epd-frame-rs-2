use crate::display::color::{BinaryColorAdapter, E6Color};
use crate::display::image::E6ImageSource;
use crate::display::weather::frog::Frog130x180;
use alloc::format;
use alloc::string::ToString;
use chrono::{DateTime, FixedOffset, NaiveDate};
use core::f32::consts::PI;
use defmt_or_log::derive_format_or_debug;
use embedded_graphics::Drawable;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Point;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::{Primitive, Size};
use embedded_graphics::primitives::{Line, PrimitiveStyle, StyledDrawable};
use embedded_graphics::text::Text;
use micromath::F32Ext;
use mplusfonts::BitmapFont;
use mplusfonts::style::BitmapFontStyle;
use mplusfonts_macros::mplus;

pub mod frog;

const CLEAR_DAY: &[u8] = include_bytes!("../../resources/weather_icons_64/clear-day.e6spectra");
const CLEAR_NIGHT: &[u8] = include_bytes!("../../resources/weather_icons_64/clear-night.e6spectra");
const CLOUDY: &[u8] = include_bytes!("../../resources/weather_icons_64/cloudy.e6spectra");
const FOG: &[u8] = include_bytes!("../../resources/weather_icons_64/fog.e6spectra");
const HAIL: &[u8] = include_bytes!("../../resources/weather_icons_64/hail.e6spectra");
const PARTLY_CLOUDY_DAY: &[u8] =
    include_bytes!("../../resources/weather_icons_64/partly-cloudy-day.e6spectra");
const PARTLY_CLOUDY_NIGHT: &[u8] =
    include_bytes!("../../resources/weather_icons_64/partly-cloudy-night.e6spectra");
const RAIN: &[u8] = include_bytes!("../../resources/weather_icons_64/rain.e6spectra");
const RAIN_SNOW: &[u8] = include_bytes!("../../resources/weather_icons_64/rain-snow.e6spectra");
const RAIN_SNOW_SHOWERS_DAY: &[u8] =
    include_bytes!("../../resources/weather_icons_64/rain-snow-showers-day.e6spectra");
const RAIN_SNOW_SHOWERS_NIGHT: &[u8] =
    include_bytes!("../../resources/weather_icons_64/rain-snow-showers-night.e6spectra");
const SHOWERS_DAY: &[u8] = include_bytes!("../../resources/weather_icons_64/showers-day.e6spectra");
const SHOWERS_NIGHT: &[u8] =
    include_bytes!("../../resources/weather_icons_64/showers-night.e6spectra");
const SLEET: &[u8] = include_bytes!("../../resources/weather_icons_64/sleet.e6spectra");
const SNOW: &[u8] = include_bytes!("../../resources/weather_icons_64/snow.e6spectra");
const SNOW_SHOWERS_DAY: &[u8] =
    include_bytes!("../../resources/weather_icons_64/snow-showers-day.e6spectra");
const SNOW_SHOWERS_NIGHT: &[u8] =
    include_bytes!("../../resources/weather_icons_64/snow-showers-night.e6spectra");
const THUNDER: &[u8] = include_bytes!("../../resources/weather_icons_64/thunder.e6spectra");
const THUNDER_RAIN: &[u8] =
    include_bytes!("../../resources/weather_icons_64/thunder-rain.e6spectra");
const THUNDER_SHOWERS_DAY: &[u8] =
    include_bytes!("../../resources/weather_icons_64/thunder-showers-day.e6spectra");
const THUNDER_SHOWERS_NIGHT: &[u8] =
    include_bytes!("../../resources/weather_icons_64/thunder-showers-night.e6spectra");
const WIND: &[u8] = include_bytes!("../../resources/weather_icons_64/wind.e6spectra");

const HUMIDITY_16: &[u8] = include_bytes!("../../resources/icons_16/humidity_16.e6spectra");
const WATER_16: &[u8] = include_bytes!("../../resources/icons_16/water_16.e6spectra");
const TEMPERATURE_16: &[u8] = include_bytes!("../../resources/icons_16/temperature_16.e6spectra");
const WIND_16: &[u8] = include_bytes!("../../resources/icons_16/wind_16.e6spectra");

#[derive(Clone, Copy)]
#[derive_format_or_debug]
#[repr(u8)]
pub enum Icon64 {
    ClearDay,
    ClearNight,
    Cloudy,
    Fog,
    Hail,
    PartlyCloudyDay,
    PartlyCloudyNight,
    Rain,
    RainSnow,
    RainSnowShowersDay,
    RainSnowShowersNight,
    ShowersDay,
    ShowersNight,
    Sleet,
    Snow,
    SnowShowersDay,
    SnowShowersNight,
    Thunder,
    ThunderRain,
    ThunderShowersDay,
    ThunderShowersNight,
    Wind,
}

impl E6ImageSource for Icon64 {
    fn source_bytes(&self) -> &[u8] {
        match self {
            Icon64::ClearDay => CLEAR_DAY,
            Icon64::ClearNight => CLEAR_NIGHT,
            Icon64::Cloudy => CLOUDY,
            Icon64::Fog => FOG,
            Icon64::Hail => HAIL,
            Icon64::PartlyCloudyDay => PARTLY_CLOUDY_DAY,
            Icon64::PartlyCloudyNight => PARTLY_CLOUDY_NIGHT,
            Icon64::Rain => RAIN,
            Icon64::RainSnow => RAIN_SNOW,
            Icon64::RainSnowShowersDay => RAIN_SNOW_SHOWERS_DAY,
            Icon64::RainSnowShowersNight => RAIN_SNOW_SHOWERS_NIGHT,
            Icon64::ShowersDay => SHOWERS_DAY,
            Icon64::ShowersNight => SHOWERS_NIGHT,
            Icon64::Sleet => SLEET,
            Icon64::Snow => SNOW,
            Icon64::SnowShowersDay => SNOW_SHOWERS_DAY,
            Icon64::SnowShowersNight => SNOW_SHOWERS_NIGHT,
            Icon64::Thunder => THUNDER,
            Icon64::ThunderRain => THUNDER_RAIN,
            Icon64::ThunderShowersDay => THUNDER_SHOWERS_DAY,
            Icon64::ThunderShowersNight => THUNDER_SHOWERS_NIGHT,
            Icon64::Wind => WIND,
        }
    }
}

#[derive(Clone, Copy)]
#[derive_format_or_debug]
#[repr(u8)]
pub enum Icon16 {
    Temperature,
    Humidity,
    Water,
    Wind,
}

impl E6ImageSource for Icon16 {
    fn source_bytes(&self) -> &[u8] {
        match self {
            Icon16::Temperature => TEMPERATURE_16,
            Icon16::Humidity => HUMIDITY_16,
            Icon16::Water => WATER_16,
            Icon16::Wind => WIND_16,
        }
    }
}

pub type TemperatureCelsium = f32;
pub type Percentage = u16;
pub type SpeedKilometersPerHour = u16;
pub type DirectionDegrees = u16;
pub type LevelMillimeters = f32;

#[derive(Clone, Copy)]
#[derive_format_or_debug]
pub struct Weather {
    pub current: CurrentWeather,
    pub hourly: [HourlyWeather; 4],
    pub daily: [DailyWeather; 7],
}

#[derive(Clone, Copy)]
#[derive_format_or_debug]
pub struct CurrentWeather {
    pub time: DateTime<FixedOffset>,
    pub temperature: TemperatureCelsium,
    pub apparent_temperature: TemperatureCelsium,
    pub is_day: bool,
    pub humidity: Percentage,
    pub precipitation: LevelMillimeters,
    pub wind_speed: SpeedKilometersPerHour,
    pub wind_direction: DirectionDegrees,
    pub wind_gusts: SpeedKilometersPerHour,
    pub weather_icon: Icon64,
    pub cloud_cover: Percentage,
}

#[derive(Clone, Copy)]
#[derive_format_or_debug]
pub struct HourlyWeather {
    pub time: DateTime<FixedOffset>,
    pub temperature: TemperatureCelsium,
    pub apparent_temperature: TemperatureCelsium,
    pub is_day: bool,
    pub precipitation: LevelMillimeters,
    pub precipitation_probability: Percentage,
    pub wind_speed: SpeedKilometersPerHour,
    pub wind_direction: DirectionDegrees,
    pub wind_gusts: SpeedKilometersPerHour,
    pub weather_icon: Icon64,
}

#[derive(Clone, Copy)]
#[derive_format_or_debug]
pub struct DailyWeather {
    pub time: NaiveDate,
    pub temperature_min: TemperatureCelsium,
    pub temperature_max: TemperatureCelsium,
    pub apparent_temperature_min: TemperatureCelsium,
    pub apparent_temperature_max: TemperatureCelsium,
    pub precipitation: LevelMillimeters,
    pub precipitation_probability: Percentage,
    pub wind_speed: SpeedKilometersPerHour,
    pub wind_direction: DirectionDegrees,
    pub wind_gusts: SpeedKilometersPerHour,
    pub weather_icon: Icon64,
}

// 110x180
impl HourlyWeather {
    pub async fn draw<D: DrawTarget<Color = E6Color>>(
        &self,
        position: Point,
        draw_target: &mut D,
        rand: &mut fastrand::Rng,
    ) -> Result<(), D::Error> {
        let time = self.time.format("%_H:%M").to_string();

        BinaryColorAdapter::draw_transparent(
            E6Color::Black,
            &Text::new(
                &format!("{: >5}", time.as_str()),
                position + Point::new(32, 18),
                date_time_style(),
            ),
            draw_target,
        )?;
        draw_weather_icon_background(
            position + Point::new(4, 76),
            Size::new(102, 54),
            draw_target,
            rand,
        )?;
        self.weather_icon
            .draw(position + Point::new(23, 80), draw_target)?;
        draw_wind_arrow(
            draw_target,
            self.wind_direction as f32,
            position + Point::new(90, 30),
        )?;
        draw_temperature_small(
            self.temperature,
            position + Point::new(15, 110),
            draw_target,
        )?;

        draw_icon_value_10(
            Icon16::Temperature,
            &format!("{:+}C°", self.apparent_temperature.round() as u32),
            position + Point::new(4, 142),
            draw_target,
        )?;

        draw_icon_value_10(
            Icon16::Wind,
            &format!("{:}km/h", self.wind_speed),
            position + Point::new(4, 158),
            draw_target,
        )?;

        draw_icon_value_10(
            Icon16::Water,
            &format!(
                "{:.1}mm {:}%",
                self.precipitation, self.precipitation_probability
            ),
            position + Point::new(4, 176),
            draw_target,
        )?;

        Ok(())
    }
}

// 110x200
impl DailyWeather {
    pub async fn draw<D: DrawTarget<Color = E6Color>>(
        &self,
        position: Point,
        draw_target: &mut D,
        rand: &mut fastrand::Rng,
    ) -> Result<(), D::Error> {
        let date = self.time.format("%A").to_string();

        BinaryColorAdapter::draw_transparent(
            E6Color::Black,
            &Text::new(
                &format!("{: >9}", date),
                position + Point::new(10, 18),
                date_time_style(),
            ),
            draw_target,
        )?;
        draw_weather_icon_background(
            position + Point::new(4, 76),
            Size::new(102, 54),
            draw_target,
            rand,
        )?;
        self.weather_icon
            .draw(position + Point::new(23, 80), draw_target)?;
        draw_wind_arrow(
            draw_target,
            self.wind_direction as f32,
            position + Point::new(90, 30),
        )?;
        draw_temperature_small(
            self.temperature_max,
            position + Point::new(15, 110),
            draw_target,
        )?;
        draw_temperature_small_secondary(
            self.temperature_min,
            position + Point::new(35, 132),
            draw_target,
        )?;

        draw_icon_value_10(
            Icon16::Temperature,
            &format!(
                "{:+}/{:+}C°",
                self.apparent_temperature_max.round() as u32,
                self.apparent_temperature_min.round() as u32
            ),
            position + Point::new(4, 160),
            draw_target,
        )?;

        draw_icon_value_10(
            Icon16::Wind,
            &format!("{:}km/h", self.wind_speed),
            position + Point::new(4, 178),
            draw_target,
        )?;

        draw_icon_value_10(
            Icon16::Water,
            &format!(
                "{:.1}mm {:}%",
                self.precipitation, self.precipitation_probability
            ),
            position + Point::new(4, 196),
            draw_target,
        )?;

        Ok(())
    }
}

impl CurrentWeather {
    // 340x180
    pub async fn draw<D: DrawTarget<Color = E6Color>>(
        &self,
        position: Point,
        draw_target: &mut D,
        _rand: &mut fastrand::Rng,
    ) -> Result<(), D::Error> {
        let date_time = self.time.format("%A, %d %B %Y, %_H:%M").to_string();
        let frog_icon: Frog130x180 = self.weather_icon.into();

        frog_icon.draw(position + Point::new(210, 180), draw_target)?;

        BinaryColorAdapter::draw_transparent(
            E6Color::Black,
            &Text::new(
                &format!("{:}", date_time),
                position + Point::new(5, 17),
                date_time_style(),
            ),
            draw_target,
        )?;

        draw_temperature(self.temperature, position + Point::new(5, 75), draw_target)?;

        let last_position = BinaryColorAdapter::draw_transparent(
            E6Color::Black,
            &Text::new(
                "Feels like ",
                position + Point::new(5, 105),
                current_weather_details_style(),
            ),
            draw_target,
        )?;
        draw_temperature_small(self.apparent_temperature, last_position, draw_target)?;

        BinaryColorAdapter::draw_transparent(
            E6Color::Black,
            &Text::new(
                &format!(
                    "Wind: {:}km/h ({:}km/h)\nHumidity: {:}%\nPrecipitation: {:}mm\nClouds: {:}%",
                    self.wind_speed,
                    self.wind_gusts,
                    self.humidity,
                    self.precipitation,
                    self.cloud_cover
                ),
                position + Point::new(5, 130),
                current_weather_details_style(),
            ),
            draw_target,
        )?;

        Ok(())
    }
}

fn date_time_style() -> BitmapFontStyle<'static, 'static, BinaryColor, BinaryColor, 2> {
    BitmapFontStyle::new(&super::DEFAULT_FONT_12, BinaryColor::On)
}

fn current_weather_details_style() -> BitmapFontStyle<'static, 'static, BinaryColor, BinaryColor, 2>
{
    date_time_style()
}

fn temperature_color(temperature: TemperatureCelsium) -> E6Color {
    match temperature as i32 {
        ..1 => E6Color::Blue,
        18..26 => E6Color::Green,
        26.. => E6Color::Red,
        _ => E6Color::Black,
    }
}

fn draw_temperature<D>(
    temperature: TemperatureCelsium,
    position: Point,
    draw_target: &mut D,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = E6Color>,
{
    static TEMPERATURE_FONT: BitmapFont<BinaryColor, 2> =
        mplus!(1, MEDIUM, cap_height(50), true, 2, 1, '0'..='9', ["+-C°"]);
    let color = temperature_color(temperature);
    let style = BitmapFontStyle::new(&TEMPERATURE_FONT, BinaryColor::On);
    let text = format!("{:+}C°", temperature.round() as u32);
    BinaryColorAdapter::draw_transparent(
        color,
        &Text::new(text.as_str(), position, style),
        draw_target,
    )?;

    Ok(())
}

fn draw_temperature_small<D>(
    temperature: TemperatureCelsium,
    position: Point,
    draw_target: &mut D,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = E6Color>,
{
    static TEMPERATURE_FONT: BitmapFont<BinaryColor, 2> =
        mplus!(1, BOLD, cap_height(20), true, 2, 1, '0'..='9', ["+-C°"]);
    let color = temperature_color(temperature);
    let style = BitmapFontStyle::new(&TEMPERATURE_FONT, BinaryColor::On);
    let text = format!("{:+}C°", temperature.round() as u32);
    BinaryColorAdapter::draw_transparent(
        color,
        &Text::new(text.as_str(), position, style),
        draw_target,
    )?;
    Ok(())
}

fn draw_temperature_small_secondary<D>(
    temperature: TemperatureCelsium,
    position: Point,
    draw_target: &mut D,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = E6Color>,
{
    static TEMPERATURE_FONT: BitmapFont<BinaryColor, 2> =
        mplus!(1, BOLD, cap_height(12), true, 2, 1, '0'..='9', ["+-C°"]);
    let color = temperature_color(temperature);
    let style = BitmapFontStyle::new(&TEMPERATURE_FONT, BinaryColor::On);
    let text = format!("{:+}C°", temperature.round() as u32);
    BinaryColorAdapter::draw_transparent(
        color,
        &Text::new(text.as_str(), position, style),
        draw_target,
    )?;
    Ok(())
}

fn draw_wind_arrow<D>(target: &mut D, degrees: f32, Point { x, y }: Point) -> Result<(), D::Error>
where
    D: DrawTarget<Color = E6Color>,
{
    let rad = degrees * PI / 180.0;
    let sin_a = rad.sin();
    let cos_a = rad.cos();
    let radius = 7.0; // Half the length of the main line
    let head_len = 6.0; // Length of the arrowhead wings
    let head_angle = 145.0 * PI / 180.0; // The sweep angle of the arrowhead wings
    let tip_x = x as f32 + radius * sin_a;
    let tip_y = y as f32 - radius * cos_a;
    let base_x = x as f32 - radius * sin_a;
    let base_y = y as f32 + radius * cos_a;
    let w1_rad = rad + head_angle;
    let w2_rad = rad - head_angle;
    let w1_x = tip_x + head_len * w1_rad.sin();
    let w1_y = tip_y - head_len * w1_rad.cos();
    let w2_x = tip_x + head_len * w2_rad.sin();
    let w2_y = tip_y - head_len * w2_rad.cos();

    let p_base = Point::new(base_x.round() as i32, base_y.round() as i32);
    let p_tip = Point::new(tip_x.round() as i32, tip_y.round() as i32);
    let p_w1 = Point::new(w1_x.round() as i32, w1_y.round() as i32);
    let p_w2 = Point::new(w2_x.round() as i32, w2_y.round() as i32);

    let style = PrimitiveStyle::with_stroke(E6Color::Black, 3);

    Line::new(p_base, p_tip).into_styled(style).draw(target)?;
    Line::new(p_tip, p_w1).into_styled(style).draw(target)?;
    Line::new(p_tip, p_w2).into_styled(style).draw(target)?;

    Ok(())
}

fn draw_icon_value_10<D>(
    icon: Icon16,
    value: &str,
    position: Point,
    draw_target: &mut D,
) -> Result<Point, D::Error>
where
    D: DrawTarget<Color = E6Color>,
{
    icon.draw(position, draw_target)?;
    draw_value_10(value, position + Point::new(18, -2), draw_target)
}

fn draw_value_10<D>(value: &str, position: Point, draw_target: &mut D) -> Result<Point, D::Error>
where
    D: DrawTarget<Color = E6Color>,
{
    static VALUE_FONT: BitmapFont<BinaryColor, 2> = mplus!(
        1,
        BOLD,
        cap_height(10),
        true,
        2,
        1,
        '0'..='9',
        ["., +-C°km/h%"]
    );
    let style = BitmapFontStyle::new(&VALUE_FONT, BinaryColor::On);
    Ok(BinaryColorAdapter::draw_transparent(
        E6Color::Black,
        &Text::new(value, position, style),
        draw_target,
    )?)
}

fn draw_weather_icon_background<D>(
    position: Point,
    size: Size,
    draw_target: &mut D,
    rand: &mut fastrand::Rng,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = E6Color>,
{
    let height = size.height as i32;
    let shape_part = size.width as f32 / 6f32;
    let random_part = (size.width / 8) as u8;
    let start_x = position.x;
    let end_x = position.x + size.width as i32;
    let style = PrimitiveStyle::with_stroke(E6Color::Blue, 1);
    for y in (0..height).step_by(5) {
        let shape_diff = y.abs_diff(height / 2) as f32 / (height / 2) as f32;
        let y = position.y - y;
        let length = (shape_diff * shape_part) as i32 + rand.u8(0..random_part) as i32;
        Line::new((start_x, y).into(), (start_x + length, y).into())
            .draw_styled(&style, draw_target)?;

        let length = (shape_diff * shape_part) as i32 + rand.u8(0..random_part) as i32;
        Line::new((end_x - length, y).into(), (end_x, y).into())
            .draw_styled(&style, draw_target)?;
    }
    Ok(())
}
