use crate::display::image::E6ImageSource;
use chrono::{DateTime, FixedOffset, NaiveDate};
use defmt_or_log::derive_format_or_debug;
use embedded_graphics::prelude::Size;

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

    fn size(&self) -> Size {
        (64, 64).into()
    }
}

pub type TemperatureCelsium = f32;
pub type Percentage = u16;
pub type SpeedKilometersPerHour = u16;
pub type DirectionDegrees = u16;
pub type LevelMillimeters = f32;
pub type UvIndex = f32;

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
    pub uv_index: UvIndex,
}

#[derive(Clone, Copy)]
#[derive_format_or_debug]
pub struct HourlyWeather {
    pub time: DateTime<FixedOffset>,
    pub temperature: TemperatureCelsium,
    pub apparent_temperature: TemperatureCelsium,
    pub is_day: bool,
    pub humidity: Percentage,
    pub precipitation: LevelMillimeters,
    pub precipitation_probability: Percentage,
    pub wind_speed: SpeedKilometersPerHour,
    pub wind_direction: DirectionDegrees,
    pub wind_gusts: SpeedKilometersPerHour,
    pub weather_icon: Icon64,
    pub uv_index: UvIndex,
}

#[derive(Clone, Copy)]
#[derive_format_or_debug]
pub struct DailyWeather {
    pub time: NaiveDate,
    pub temperature_min: TemperatureCelsium,
    pub temperature_max: TemperatureCelsium,
    pub humidity: Percentage,
    pub apparent_temperature_min: TemperatureCelsium,
    pub apparent_temperature_max: TemperatureCelsium,
    pub precipitation: LevelMillimeters,
    pub precipitation_probability: Percentage,
    pub wind_speed: SpeedKilometersPerHour,
    pub wind_direction: DirectionDegrees,
    pub wind_gusts: SpeedKilometersPerHour,
    pub weather_icon: Icon64,
    pub uv_index_max: UvIndex,
}