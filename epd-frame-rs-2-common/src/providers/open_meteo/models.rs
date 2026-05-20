//!
//! Generated from an OAS specification by openapi-model-generator(v0.6.0)
//!

use crate::display::weather;
use alloc::vec::Vec;
use chrono::{FixedOffset, NaiveDate, NaiveDateTime};
use core::fmt::{Display, Formatter};
use defmt::Format;
use serde::Deserialize;

/// For each selected weather variable, data will be returned as a floating point array. Additionally a `time` array will be returned with ISO8601 timestamps.
#[derive(Clone, Deserialize, Debug)]
pub struct HourlyWeather {
    pub time: Vec<DateTime>,
    pub temperature_2m: Vec<f32>,
    pub relative_humidity_2m: Vec<f32>,
    pub apparent_temperature: Vec<f32>,
    pub cloud_cover: Vec<f32>,
    pub wind_speed_10m: Vec<f32>,
    pub wind_direction_10m: Vec<f32>,
    pub wind_gusts_10m: Vec<f32>,
    pub precipitation: Vec<f32>,
    pub precipitation_probability: Vec<u32>,
    pub weather_code: Vec<WeatherCode>,
    pub is_day: Vec<u8>,
    pub uv_index: Vec<weather::UvIndex>,
}

/// For each selected daily weather variable, data will be returned as a floating point array. Additionally a `time` array will be returned with ISO8601 timestamps.
#[derive(Clone, Deserialize, Debug)]
pub struct DailyWeather {
    pub time: Vec<NaiveDate>,
    pub temperature_2m_max: Vec<f32>,
    pub temperature_2m_min: Vec<f32>,
    pub relative_humidity_2m_mean: Vec<f32>,
    pub apparent_temperature_max: Vec<f32>,
    pub apparent_temperature_min: Vec<f32>,
    pub precipitation_sum: Vec<f32>,
    pub precipitation_probability_max: Vec<u32>,
    pub weather_code: Vec<WeatherCode>,
    pub wind_speed_10m_max: Vec<f32>,
    pub wind_gusts_10m_max: Vec<f32>,
    pub wind_direction_10m_dominant: Vec<f32>,
    pub uv_index_max: Vec<weather::UvIndex>,
}

/// Current weather conditions with the attributes: time, temperature, wind_speed, wind_direction and weather_code
#[derive(Clone, Deserialize, Debug)]
pub struct CurrentWeather {
    pub time: DateTime,
    pub temperature_2m: f32,
    pub apparent_temperature: f32,
    pub relative_humidity_2m: u16,
    pub wind_speed_10m: f32,
    pub wind_direction_10m: f32,
    pub wind_gusts_10m: f32,
    pub precipitation: f32,
    pub cloud_cover: f32,
    pub weather_code: WeatherCode,
    pub is_day: u8,
    pub uv_index: weather::UvIndex,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Response {
    pub latitude: f32,
    pub longitude: f32,
    pub elevation: f32,
    pub generationtime_ms: f32,
    pub utc_offset_seconds: i32,
    pub current: CurrentWeather,
    pub hourly: HourlyWeather,
    pub daily: DailyWeather,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DateTime(pub NaiveDateTime);

impl<'de> serde::de::Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_str(DateTimeVisitor)
            .map(|date_time| Self(date_time))
    }
}

pub struct DateTimeVisitor;

impl<'a> serde::de::Visitor<'a> for DateTimeVisitor {
    type Value = NaiveDateTime;
    fn expecting(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            concat!("a(n) `", "NaiveDateTime", "` in the format \"{}\"",),
            "%y-%m-%dT%H:%M"
        ))
    }
    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<NaiveDateTime, E> {
        NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M").map_err(E::custom)
    }
}

impl DateTimeVisitor {
    pub fn deserialize<'a, D: serde::Deserializer<'a>>(
        deserializer: D,
    ) -> Result<NaiveDateTime, D::Error> {
        deserializer.deserialize_str(Self)
    }
}

#[derive(Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
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
    pub fn into_day_icon(self) -> weather::Icon64 {
        match self {
            WeatherCode::ClearSky => weather::Icon64::ClearDay,
            WeatherCode::MainlyClear => weather::Icon64::PartlyCloudyDay,
            WeatherCode::PartlyClear => weather::Icon64::PartlyCloudyDay,
            WeatherCode::Overcast => weather::Icon64::Cloudy,
            WeatherCode::Fog => weather::Icon64::Fog,
            WeatherCode::DepositingRimeFog => weather::Icon64::Fog,
            WeatherCode::LightDrizzle => weather::Icon64::Rain,
            WeatherCode::ModerateDrizzle => weather::Icon64::Rain,
            WeatherCode::DenseDrizzle => weather::Icon64::Rain,
            WeatherCode::LightFreezingDrizzle => weather::Icon64::RainSnow,
            WeatherCode::DenseFreezingDrizzle => weather::Icon64::RainSnow,
            WeatherCode::SlightRain => weather::Icon64::Rain,
            WeatherCode::ModerateRain => weather::Icon64::Rain,
            WeatherCode::HeavyRain => weather::Icon64::ShowersDay,
            WeatherCode::SlightFreezingRain => weather::Icon64::RainSnowShowersDay,
            WeatherCode::HeavyFreezingRain => weather::Icon64::RainSnowShowersDay,
            WeatherCode::SlightSnow => weather::Icon64::Snow,
            WeatherCode::ModerateSnow => weather::Icon64::Snow,
            WeatherCode::HeavySnow => weather::Icon64::Snow,
            WeatherCode::SnowGrains => weather::Icon64::Hail,
            WeatherCode::SlightRainShowers => weather::Icon64::ShowersDay,
            WeatherCode::ModerateRainShowers => weather::Icon64::ShowersDay,
            WeatherCode::ViolentRainShowers => weather::Icon64::ShowersDay,
            WeatherCode::SlightSnowShowers => weather::Icon64::SnowShowersDay,
            WeatherCode::HeavySnowShowers => weather::Icon64::SnowShowersDay,
            WeatherCode::Thunderstorm => weather::Icon64::Thunder,
            WeatherCode::ThunderstormSlightHail => weather::Icon64::ThunderShowersDay,
            WeatherCode::ThunderstormHeavyHail => weather::Icon64::ThunderShowersDay,
        }
    }

    pub fn into_night_icon(self) -> weather::Icon64 {
        match self {
            WeatherCode::ClearSky => weather::Icon64::ClearNight,
            WeatherCode::MainlyClear => weather::Icon64::PartlyCloudyNight,
            WeatherCode::PartlyClear => weather::Icon64::PartlyCloudyNight,
            WeatherCode::Overcast => weather::Icon64::Cloudy,
            WeatherCode::Fog => weather::Icon64::Fog,
            WeatherCode::DepositingRimeFog => weather::Icon64::Fog,
            WeatherCode::LightDrizzle => weather::Icon64::Rain,
            WeatherCode::ModerateDrizzle => weather::Icon64::Rain,
            WeatherCode::DenseDrizzle => weather::Icon64::Rain,
            WeatherCode::LightFreezingDrizzle => weather::Icon64::RainSnow,
            WeatherCode::DenseFreezingDrizzle => weather::Icon64::RainSnow,
            WeatherCode::SlightRain => weather::Icon64::Rain,
            WeatherCode::ModerateRain => weather::Icon64::Rain,
            WeatherCode::HeavyRain => weather::Icon64::ShowersNight,
            WeatherCode::SlightFreezingRain => weather::Icon64::RainSnowShowersNight,
            WeatherCode::HeavyFreezingRain => weather::Icon64::RainSnowShowersNight,
            WeatherCode::SlightSnow => weather::Icon64::Snow,
            WeatherCode::ModerateSnow => weather::Icon64::Snow,
            WeatherCode::HeavySnow => weather::Icon64::Snow,
            WeatherCode::SnowGrains => weather::Icon64::Hail,
            WeatherCode::SlightRainShowers => weather::Icon64::ShowersNight,
            WeatherCode::ModerateRainShowers => weather::Icon64::ShowersNight,
            WeatherCode::ViolentRainShowers => weather::Icon64::ShowersNight,
            WeatherCode::SlightSnowShowers => weather::Icon64::SnowShowersNight,
            WeatherCode::HeavySnowShowers => weather::Icon64::SnowShowersNight,
            WeatherCode::Thunderstorm => weather::Icon64::Thunder,
            WeatherCode::ThunderstormSlightHail => weather::Icon64::ThunderShowersNight,
            WeatherCode::ThunderstormHeavyHail => weather::Icon64::ThunderShowersNight,
        }
    }
}

impl Response {
    fn hourly_weather(&self, index: usize, timezone: FixedOffset) -> weather::HourlyWeather {
        weather::HourlyWeather {
            time: self.hourly.time[index]
                .0
                .and_local_timezone(timezone)
                .unwrap(),
            temperature: self.hourly.temperature_2m[index],
            apparent_temperature: self.hourly.apparent_temperature[index],
            is_day: self.hourly.is_day[index] != 0,
            humidity: self.hourly.relative_humidity_2m[index] as weather::Percentage,
            precipitation: self.hourly.precipitation[index],
            precipitation_probability: self.hourly.precipitation_probability[index]
                as weather::Percentage,
            wind_speed: self.hourly.wind_speed_10m[index] as weather::SpeedKilometersPerHour,
            wind_direction: self.hourly.wind_direction_10m[index] as weather::DirectionDegrees,
            wind_gusts: self.hourly.wind_gusts_10m[index] as weather::SpeedKilometersPerHour,
            weather_icon: if self.hourly.is_day[index] != 0 {
                self.hourly.weather_code[index].into_day_icon()
            } else {
                self.hourly.weather_code[index].into_night_icon()
            },
            uv_index: self.hourly.uv_index[index],
        }
    }

    fn daily_weather(&self, index: usize, _timezone: FixedOffset) -> weather::DailyWeather {
        weather::DailyWeather {
            time: self.daily.time[index].and_hms_opt(0, 0, 0).unwrap().date(),
            temperature_max: self.daily.temperature_2m_max[index],
            temperature_min: self.daily.temperature_2m_min[index],
            humidity: self.daily.relative_humidity_2m_mean[index] as weather::Percentage,
            apparent_temperature_max: self.daily.apparent_temperature_max[index],
            apparent_temperature_min: self.daily.apparent_temperature_min[index],
            precipitation: self.daily.precipitation_sum[index],
            precipitation_probability: self.daily.precipitation_probability_max[index]
                as weather::Percentage,
            wind_speed: self.daily.wind_speed_10m_max[index] as weather::SpeedKilometersPerHour,
            wind_direction: self.daily.wind_direction_10m_dominant[index]
                as weather::DirectionDegrees,
            wind_gusts: self.daily.wind_gusts_10m_max[index] as weather::SpeedKilometersPerHour,
            weather_icon: self.daily.weather_code[index].into_day_icon(),
            uv_index_max: self.daily.uv_index_max[index],
        }
    }
}

impl From<Response> for weather::Weather {
    fn from(v: Response) -> Self {
        let timezone = FixedOffset::east_opt(v.utc_offset_seconds)
            .unwrap_or_else(|| FixedOffset::east_opt(0).unwrap());
        let current = weather::CurrentWeather {
            time: v.current.time.0.and_local_timezone(timezone).unwrap(),
            temperature: v.current.temperature_2m,
            apparent_temperature: v.current.apparent_temperature,
            is_day: v.current.is_day != 0,
            humidity: v.current.relative_humidity_2m,
            precipitation: v.current.precipitation,
            wind_speed: v.current.wind_speed_10m as weather::SpeedKilometersPerHour,
            wind_direction: v.current.wind_direction_10m as weather::DirectionDegrees,
            wind_gusts: v.current.wind_gusts_10m as weather::SpeedKilometersPerHour,
            weather_icon: if v.current.is_day != 0 {
                v.current.weather_code.into_day_icon()
            } else {
                v.current.weather_code.into_night_icon()
            },
            cloud_cover: v.current.cloud_cover as weather::Percentage,
            uv_index: v.current.uv_index,
        };

        let hourly = core::array::from_fn(|index| v.hourly_weather(index, timezone));
        let daily = core::array::from_fn(|index| v.daily_weather(index, timezone));

        Self {
            current,
            hourly,
            daily,
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

    #[test]
    fn convert_response() {
        let response = include_str!("../../../resources/test/open_meteo/response.json");
        let deserialized: Response = serde_json::from_str(response.trim()).unwrap();
        let weather: weather::Weather = deserialized.into();
        info!("{:#?}", weather);
    }
}
