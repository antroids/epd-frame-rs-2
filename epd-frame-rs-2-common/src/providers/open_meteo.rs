use crate::display::weather::Weather;
use crate::errors::DeviceError;
use crate::http;
use alloc::string::String;
use alloc::{format, vec};

pub mod models;

pub fn request_url(latitude: f32, longitude: f32) -> String {
    format!(
        "http://api.open-meteo.com/v1/forecast?latitude={}&longitude={}\
        &daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_sum,\
        precipitation_hours,precipitation_probability_max,wind_speed_10m_max,wind_gusts_10m_max,\
        wind_direction_10m_dominant,apparent_temperature_max,apparent_temperature_min\
        &hourly=temperature_2m,relative_humidity_2m,apparent_temperature,precipitation_probability,\
        precipitation,weather_code,wind_speed_10m,wind_direction_10m,wind_gusts_10m,cloud_cover,is_day\
        &current=temperature_2m,is_day,relative_humidity_2m,apparent_temperature,\
        precipitation,weather_code,wind_speed_10m,wind_direction_10m,wind_gusts_10m,\
        cloud_cover&forecast_hours=24&temporal_resolution=hourly_6&timezone=auto",
        latitude, longitude
    )
}

pub async fn get_weather(
    client: &mut http::client::HttpClient,
    latitude: f32,
    longitude: f32,
) -> Result<Weather, DeviceError> {
    let mut response_buffer = vec![0; 1024 * 8];
    let headers = [
        http::Header::default_user_agent().into(),
        http::Header::accept("application/json").into(),
    ];
    let url = request_url(latitude, longitude);
    let response: models::Response = client.get(&url, &headers, &mut response_buffer).await?;

    Ok(response.into())
}
