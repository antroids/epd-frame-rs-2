//!
//! Generated from an OAS specification by openapi-model-generator(v0.6.0)
//!

use crate::display;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use chrono::{NaiveDate, NaiveDateTime};
use core::fmt::{Display, Formatter};
use defmt::Format;
use defmt_or_log::derive_format_or_debug;
use serde::{Deserialize, Deserializer};

/// For each selected weather variable, data will be returned as a floating point array. Additionally a `time` array will be returned with ISO8601 timestamps.
#[derive(Clone, Deserialize, Debug)]
pub struct HourlyWeather {
    pub time: Vec<DateTime>,
    pub temperature_2m: Option<Vec<f64>>,
    pub relative_humidity_2m: Option<Vec<f64>>,
    pub dew_point_2m: Option<Vec<f64>>,
    pub apparent_temperature: Option<Vec<f64>>,
    pub pressure_msl: Option<Vec<f64>>,
    pub cloud_cover: Option<Vec<f64>>,
    pub cloud_cover_low: Option<Vec<f64>>,
    pub cloud_cover_mid: Option<Vec<f64>>,
    pub cloud_cover_high: Option<Vec<f64>>,
    pub wind_speed_10m: Option<Vec<f64>>,
    pub wind_speed_80m: Option<Vec<f64>>,
    pub wind_speed_120m: Option<Vec<f64>>,
    pub wind_speed_180m: Option<Vec<f64>>,
    pub wind_direction_10m: Option<Vec<f64>>,
    pub wind_direction_80m: Option<Vec<f64>>,
    pub wind_direction_120m: Option<Vec<f64>>,
    pub wind_direction_180m: Option<Vec<f64>>,
    pub wind_gusts_10m: Option<Vec<f64>>,
    pub shortwave_radiation: Option<Vec<f64>>,
    pub direct_radiation: Option<Vec<f64>>,
    pub direct_normal_irradiance: Option<Vec<f64>>,
    pub diffuse_radiation: Option<Vec<f64>>,
    pub vapour_pressure_deficit: Option<Vec<f64>>,
    pub evapotranspiration: Option<Vec<f64>>,
    pub precipitation: Option<Vec<f64>>,
    pub weather_code: Option<Vec<f64>>,
    pub snow_height: Option<Vec<f64>>,
    pub freezing_level_height: Option<Vec<f64>>,
    pub soil_temperature_0cm: Option<Vec<f64>>,
    pub soil_temperature_6cm: Option<Vec<f64>>,
    pub soil_temperature_18cm: Option<Vec<f64>>,
    pub soil_temperature_54cm: Option<Vec<f64>>,
    pub soil_moisture_0_1cm: Option<Vec<f64>>,
    pub soil_moisture_1_3cm: Option<Vec<f64>>,
    pub soil_moisture_3_9cm: Option<Vec<f64>>,
    pub soil_moisture_9_27cm: Option<Vec<f64>>,
    pub soil_moisture_27_81cm: Option<Vec<f64>>,
}

/// For each selected daily weather variable, data will be returned as a floating point array. Additionally a `time` array will be returned with ISO8601 timestamps.
#[derive(Clone, Deserialize, Debug)]
pub struct DailyWeather {
    pub time: Vec<NaiveDate>,
    pub temperature_2m_max: Option<Vec<f64>>,
    pub temperature_2m_min: Option<Vec<f64>>,
    pub apparent_temperature_max: Option<Vec<f64>>,
    pub apparent_temperature_min: Option<Vec<f64>>,
    pub precipitation_sum: Option<Vec<f64>>,
    pub precipitation_hours: Option<Vec<f64>>,
    pub weather_code: Option<Vec<f64>>,
    pub sunrise: Option<Vec<f64>>,
    pub sunset: Option<Vec<f64>>,
    pub wind_speed_10m_max: Option<Vec<f64>>,
    pub wind_gusts_10m_max: Option<Vec<f64>>,
    pub wind_direction_10m_dominant: Option<Vec<f64>>,
    pub shortwave_radiation_sum: Option<Vec<f64>>,
    pub uv_index_max: Option<Vec<f64>>,
    pub uv_index_clear_sky_max: Option<Vec<f64>>,
    pub et0_fao_evapotranspiration: Option<Vec<f64>>,
}

/// Current weather conditions with the attributes: time, temperature, wind_speed, wind_direction and weather_code
#[derive(Clone, Deserialize, Debug)]
pub struct CurrentWeather {
    pub time: DateTime,
    pub temperature_2m: f64,
    pub windspeed: Option<f64>,
    pub winddirection: Option<f64>,
    pub weather_code: i64,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Response {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub elevation: Option<f32>,
    pub generationtime_ms: Option<f64>,
    pub utc_offset_seconds: Option<i32>,
    pub timezone: Option<String>,
    pub timezone_abbreviation: Option<String>,
    pub current_units: Option<BTreeMap<String, String>>,
    pub current: Option<CurrentWeather>,
    pub hourly_units: Option<BTreeMap<String, String>>,
    pub hourly: Option<HourlyWeather>,
    pub daily_units: Option<BTreeMap<String, String>>,
    pub daily: Option<DailyWeather>,
}

#[derive(Clone, Debug)]
pub struct DateTime(pub NaiveDateTime);

impl<'de> ::serde::de::Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer
            .deserialize_str(DateTimeVisitor)
            .map(|date_time| Self(date_time))
    }
}

pub struct DateTimeVisitor;

impl<'a> ::serde::de::Visitor<'a> for DateTimeVisitor {
    type Value = NaiveDateTime;
    fn expecting(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            concat!("a(n) `", "NaiveDateTime", "` in the format \"{}\"",),
            "%y-%m-%dT%H:%M"
        ))
    }
    fn visit_str<E: ::serde::de::Error>(self, value: &str) -> Result<NaiveDateTime, E> {
        NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M").map_err(E::custom)
    }
}

impl DateTimeVisitor {
    pub fn deserialize<'a, D: ::serde::Deserializer<'a>>(
        deserializer: D,
    ) -> Result<NaiveDateTime, D::Error> {
        deserializer.deserialize_str(Self)
    }
}

#[derive(Deserialize)]
#[derive_format_or_debug]
#[serde(try_from = "u8")]
#[repr(u8)]
pub enum WeatherCode {
    ClearSky,
    MainlyClear,
    PartlyClear,
    Overcast,

    Fog,
    DepositingRimeFog,

    LightDrizzle,
    ModerateDrizzle,
    DenseDrizzle,

    LightFreezingDrizzle,
    DenseFreezingDrizzle,

    SlightRain,
    ModerateRain,
    HeavyRain,

    SlightFreezingRain,
    HeavyFreezingRain,

    SlightSnow,
    ModerateSnow,
    HeavySnow,
    SnowGrains,

    SlightRainShowers,
    ModerateRainShowers,
    ViolentRainShowers,

    SlightSnowShowers,
    HeavySnowShowers,

    Thunderstorm,
    ThunderstormSlightHail,
    ThunderstormHeavyHail,
}

#[derive(Debug, Copy, Clone, Format)]
pub struct InvalidWeatherCode;

impl Display for InvalidWeatherCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("invalid weather code")
    }
}

impl TryFrom<u8> for WeatherCode {
    type Error = InvalidWeatherCode;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(WeatherCode::ClearSky),
            1 => Ok(WeatherCode::MainlyClear),
            2 => Ok(WeatherCode::PartlyClear),
            3 => Ok(WeatherCode::Overcast),
            45 => Ok(WeatherCode::Fog),
            48 => Ok(WeatherCode::DepositingRimeFog),
            51 => Ok(WeatherCode::LightDrizzle),
            53 => Ok(WeatherCode::ModerateDrizzle),
            55 => Ok(WeatherCode::DenseDrizzle),
            56 => Ok(WeatherCode::LightFreezingDrizzle),
            57 => Ok(WeatherCode::DenseFreezingDrizzle),
            61 => Ok(WeatherCode::SlightRain),
            63 => Ok(WeatherCode::ModerateRain),
            65 => Ok(WeatherCode::HeavyRain),
            66 => Ok(WeatherCode::LightFreezingDrizzle),
            67 => Ok(WeatherCode::HeavyFreezingRain),
            71 => Ok(WeatherCode::SlightSnow),
            73 => Ok(WeatherCode::ModerateSnow),
            75 => Ok(WeatherCode::HeavySnow),
            77 => Ok(WeatherCode::SnowGrains),
            80 => Ok(WeatherCode::SlightRainShowers),
            81 => Ok(WeatherCode::ModerateRainShowers),
            82 => Ok(WeatherCode::ViolentRainShowers),
            85 => Ok(WeatherCode::SlightSnowShowers),
            86 => Ok(WeatherCode::HeavySnowShowers),
            95 => Ok(WeatherCode::Thunderstorm),
            96 => Ok(WeatherCode::ThunderstormSlightHail),
            99 => Ok(WeatherCode::ThunderstormHeavyHail),

            _ => Err(InvalidWeatherCode),
        }
    }
}

impl WeatherCode {
    pub fn into_day_icon(self) -> display::weather::Icon {
        match self {
            WeatherCode::ClearSky => display::weather::Icon::ClearDay,
            WeatherCode::MainlyClear => display::weather::Icon::PartlyCloudyDay,
            WeatherCode::PartlyClear => display::weather::Icon::PartlyCloudyDay,
            WeatherCode::Overcast => display::weather::Icon::Cloudy,
            WeatherCode::Fog => display::weather::Icon::Fog,
            WeatherCode::DepositingRimeFog => display::weather::Icon::Fog,
            WeatherCode::LightDrizzle => display::weather::Icon::Rain,
            WeatherCode::ModerateDrizzle => display::weather::Icon::Rain,
            WeatherCode::DenseDrizzle => display::weather::Icon::Rain,
            WeatherCode::LightFreezingDrizzle => display::weather::Icon::RainSnow,
            WeatherCode::DenseFreezingDrizzle => display::weather::Icon::RainSnow,
            WeatherCode::SlightRain => display::weather::Icon::Rain,
            WeatherCode::ModerateRain => display::weather::Icon::Rain,
            WeatherCode::HeavyRain => display::weather::Icon::ShowersDay,
            WeatherCode::SlightFreezingRain => display::weather::Icon::RainSnowShowersDay,
            WeatherCode::HeavyFreezingRain => display::weather::Icon::RainSnowShowersDay,
            WeatherCode::SlightSnow => display::weather::Icon::Snow,
            WeatherCode::ModerateSnow => display::weather::Icon::Snow,
            WeatherCode::HeavySnow => display::weather::Icon::Snow,
            WeatherCode::SnowGrains => display::weather::Icon::Hail,
            WeatherCode::SlightRainShowers => display::weather::Icon::ShowersDay,
            WeatherCode::ModerateRainShowers => display::weather::Icon::ShowersDay,
            WeatherCode::ViolentRainShowers => display::weather::Icon::ShowersDay,
            WeatherCode::SlightSnowShowers => display::weather::Icon::SnowShowersDay,
            WeatherCode::HeavySnowShowers => display::weather::Icon::SnowShowersDay,
            WeatherCode::Thunderstorm => display::weather::Icon::Thunder,
            WeatherCode::ThunderstormSlightHail => display::weather::Icon::ThunderShowersDay,
            WeatherCode::ThunderstormHeavyHail => display::weather::Icon::ThunderShowersDay,
        }
    }

    pub fn into_night_icon(self) -> display::weather::Icon {
        match self {
            WeatherCode::ClearSky => display::weather::Icon::ClearNight,
            WeatherCode::MainlyClear => display::weather::Icon::PartlyCloudyNight,
            WeatherCode::PartlyClear => display::weather::Icon::PartlyCloudyNight,
            WeatherCode::Overcast => display::weather::Icon::Cloudy,
            WeatherCode::Fog => display::weather::Icon::Fog,
            WeatherCode::DepositingRimeFog => display::weather::Icon::Fog,
            WeatherCode::LightDrizzle => display::weather::Icon::Rain,
            WeatherCode::ModerateDrizzle => display::weather::Icon::Rain,
            WeatherCode::DenseDrizzle => display::weather::Icon::Rain,
            WeatherCode::LightFreezingDrizzle => display::weather::Icon::RainSnow,
            WeatherCode::DenseFreezingDrizzle => display::weather::Icon::RainSnow,
            WeatherCode::SlightRain => display::weather::Icon::Rain,
            WeatherCode::ModerateRain => display::weather::Icon::Rain,
            WeatherCode::HeavyRain => display::weather::Icon::ShowersNight,
            WeatherCode::SlightFreezingRain => display::weather::Icon::RainSnowShowersNight,
            WeatherCode::HeavyFreezingRain => display::weather::Icon::RainSnowShowersNight,
            WeatherCode::SlightSnow => display::weather::Icon::Snow,
            WeatherCode::ModerateSnow => display::weather::Icon::Snow,
            WeatherCode::HeavySnow => display::weather::Icon::Snow,
            WeatherCode::SnowGrains => display::weather::Icon::Hail,
            WeatherCode::SlightRainShowers => display::weather::Icon::ShowersNight,
            WeatherCode::ModerateRainShowers => display::weather::Icon::ShowersNight,
            WeatherCode::ViolentRainShowers => display::weather::Icon::ShowersNight,
            WeatherCode::SlightSnowShowers => display::weather::Icon::SnowShowersNight,
            WeatherCode::HeavySnowShowers => display::weather::Icon::SnowShowersNight,
            WeatherCode::Thunderstorm => display::weather::Icon::Thunder,
            WeatherCode::ThunderstormSlightHail => display::weather::Icon::ThunderShowersNight,
            WeatherCode::ThunderstormHeavyHail => display::weather::Icon::ThunderShowersNight,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use defmt_or_log::info;

    #[test]
    fn parse_response() {
        let response = include_str!("../../../resources/test/open_meteo/response.json");
        let deserialized: Response = serde_json::from_str(response.trim()).unwrap();
        info!("{:#?}", deserialized);
    }
}
