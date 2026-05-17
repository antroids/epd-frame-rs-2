use crate::display::image::E6ImageSource;
use crate::display::weather::Icon64;
use defmt_or_log::derive_format_or_debug;
use embedded_graphics::geometry::Size;

const CLEAR_NIGHT: &[u8] =
    include_bytes!("../../../resources/frog_130_180/ClearNight.e6spectra");
const CLOUDY: &[u8] =
    include_bytes!("../../../resources/frog_130_180/Cloudy.e6spectra");
const FOG: &[u8] = include_bytes!("../../../resources/frog_130_180/Fog.e6spectra");
const HAIL: &[u8] = include_bytes!("../../../resources/frog_130_180/Hail.e6spectra");
const PARTLY_CLOUDY_DAY: &[u8] =
    include_bytes!("../../../resources/frog_130_180/PartlyCloudyDay.e6spectra");
const PARTLY_CLOUDY_NIGHT: &[u8] =
    include_bytes!("../../../resources/frog_130_180/PartlyCloudyNight.e6spectra");
const RAIN: &[u8] = include_bytes!("../../../resources/frog_130_180/Rain.e6spectra");
const RAIN_SNOW: &[u8] =
    include_bytes!("../../../resources/frog_130_180/RainSnow.e6spectra");
const RAIN_SNOW_NIGHT: &[u8] =
    include_bytes!("../../../resources/frog_130_180/RainSnowShowersNight.e6spectra");
const RAIN_SNOW_SHOWERS_DAY: &[u8] =
    include_bytes!("../../../resources/frog_130_180/RainSnowShowersDay.e6spectra");
const RAIN_SNOW_SHOWERS_NIGHT: &[u8] =
    include_bytes!("../../../resources/frog_130_180/RainSnowShowersNight.e6spectra");
const SHOWERS_DAY: &[u8] =
    include_bytes!("../../../resources/frog_130_180/ShowersDay.e6spectra");
const SHOWERS_NIGHT: &[u8] =
    include_bytes!("../../../resources/frog_130_180/ShowersNight.e6spectra");
const SLEET: &[u8] = include_bytes!("../../../resources/frog_130_180/Sleet.e6spectra");
const SNOW: &[u8] = include_bytes!("../../../resources/frog_130_180/Snow.e6spectra");
const SNOW_SHOWERS_DAY: &[u8] =
    include_bytes!("../../../resources/frog_130_180/SnowShowersDay.e6spectra");
const SNOW_SHOWERS_NIGHT: &[u8] =
    include_bytes!("../../../resources/frog_130_180/SnowShowersNight.e6spectra");
const SUN: &[u8] = include_bytes!("../../../resources/frog_130_180/ClearDay.e6spectra");
const THUNDER: &[u8] =
    include_bytes!("../../../resources/frog_130_180/Thunder.e6spectra");
const THUNDER_RAIN: &[u8] =
    include_bytes!("../../../resources/frog_130_180/ThunderRain.e6spectra");
const THUNDER_SHOWERS_DAY: &[u8] =
    include_bytes!("../../../resources/frog_130_180/ThunderShowersDay.e6spectra");
const THUNDER_SHOWERS_NIGHT: &[u8] =
    include_bytes!("../../../resources/frog_130_180/ThunderShowersNight.e6spectra");
const WIND: &[u8] = include_bytes!("../../../resources/frog_130_180/Wind.e6spectra");

#[derive(Clone, Copy)]
#[derive_format_or_debug]
#[repr(u8)]
pub enum Frog130x180 {
    ClearNight,
    Cloudy,
    Fog,
    Hail,
    PartlyCloudyDay,
    PartlyCloudyNight,
    Rain,
    RainSnow,
    RainSnowNight,
    RainSnowShowersDay,
    RainSnowShowersNight,
    ShowersDay,
    ShowersNight,
    Sleet,
    Snow,
    SnowShowersDay,
    SnowShowersNight,
    Sun,
    Thunder,
    ThunderRain,
    ThunderShowersDay,
    ThunderShowersNight,
    Wind,
}

impl E6ImageSource for Frog130x180 {
    fn source_bytes(&self) -> &[u8] {
        match self {
            Frog130x180::ClearNight => CLEAR_NIGHT,
            Frog130x180::Cloudy => CLOUDY,
            Frog130x180::Fog => FOG,
            Frog130x180::Hail => HAIL,
            Frog130x180::PartlyCloudyDay => PARTLY_CLOUDY_DAY,
            Frog130x180::PartlyCloudyNight => PARTLY_CLOUDY_NIGHT,
            Frog130x180::Rain => RAIN,
            Frog130x180::RainSnow => RAIN_SNOW,
            Frog130x180::RainSnowNight => RAIN_SNOW_NIGHT,
            Frog130x180::RainSnowShowersDay => RAIN_SNOW_SHOWERS_DAY,
            Frog130x180::RainSnowShowersNight => RAIN_SNOW_SHOWERS_NIGHT,
            Frog130x180::ShowersDay => SHOWERS_DAY,
            Frog130x180::ShowersNight => SHOWERS_NIGHT,
            Frog130x180::Sleet => SLEET,
            Frog130x180::Snow => SNOW,
            Frog130x180::SnowShowersDay => SNOW_SHOWERS_DAY,
            Frog130x180::SnowShowersNight => SNOW_SHOWERS_NIGHT,
            Frog130x180::Sun => SUN,
            Frog130x180::Thunder => THUNDER,
            Frog130x180::ThunderRain => THUNDER_RAIN,
            Frog130x180::ThunderShowersDay => THUNDER_SHOWERS_DAY,
            Frog130x180::ThunderShowersNight => THUNDER_SHOWERS_NIGHT,
            Frog130x180::Wind => WIND,
        }
    }

    fn size(&self) -> Size {
        (130, 180).into()
    }
}

impl From<Icon64> for Frog130x180 {
    fn from(value: Icon64) -> Self {
        match value {
            Icon64::ClearNight => Frog130x180::ClearNight,
            Icon64::Cloudy => Frog130x180::Cloudy,
            Icon64::Fog => Frog130x180::Fog,
            Icon64::Hail => Frog130x180::Hail,
            Icon64::PartlyCloudyDay => Frog130x180::PartlyCloudyDay,
            Icon64::PartlyCloudyNight => Frog130x180::PartlyCloudyNight,
            Icon64::Rain => Frog130x180::Rain,
            Icon64::RainSnow => Frog130x180::RainSnow,
            Icon64::RainSnowShowersDay => Frog130x180::RainSnowShowersDay,
            Icon64::RainSnowShowersNight => Frog130x180::RainSnowShowersNight,
            Icon64::ShowersDay => Frog130x180::ShowersDay,
            Icon64::ShowersNight => Frog130x180::ShowersNight,
            Icon64::Sleet => Frog130x180::Sleet,
            Icon64::Snow => Frog130x180::Snow,
            Icon64::SnowShowersDay => Frog130x180::SnowShowersDay,
            Icon64::SnowShowersNight => Frog130x180::SnowShowersNight,
            Icon64::ClearDay => Frog130x180::Sun,
            Icon64::Thunder => Frog130x180::Thunder,
            Icon64::ThunderRain => Frog130x180::ThunderRain,
            Icon64::ThunderShowersDay => Frog130x180::ThunderShowersDay,
            Icon64::ThunderShowersNight => Frog130x180::ThunderShowersNight,
            Icon64::Wind => Frog130x180::Wind,
        }
    }
}
