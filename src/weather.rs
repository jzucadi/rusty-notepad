use serde::Deserialize;
use std::time::Duration;

use crate::constants::HTTP_TIMEOUT_SECS;

#[derive(Debug, Deserialize)]
struct GeoResponse {
    lat: f64,
    lon: f64,
}

#[derive(Debug, Clone)]
pub struct WeatherInfo {
    pub temperature_f: f64,
    pub description: String,
    pub icon: String,
}

pub fn fetch_weather() -> Option<WeatherInfo> {
    let geo_url = "http://ip-api.com/json/";
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .build()
        .ok()?;

    let geo_resp: GeoResponse = client.get(geo_url).send().ok()?.json().ok()?;

    let weather_url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current_weather=true&temperature_unit=fahrenheit",
        geo_resp.lat, geo_resp.lon
    );

    let resp = client.get(&weather_url).send().ok()?;
    let json: serde_json::Value = resp.json().ok()?;

    let current = json.get("current_weather")?;
    let temp = current.get("temperature")?.as_f64()?;
    let weather_code = current.get("weathercode")?.as_i64().unwrap_or(0);

    let (description, icon) = match weather_code {
        0 => ("Clear", "\u{2600}"),
        1..=3 => ("Partly cloudy", "\u{26C5}"),
        45 | 48 => ("Foggy", "\u{1F32B}"),
        51 | 53 | 55 => ("Drizzle", "\u{1F327}"),
        61 | 63 | 65 => ("Rain", "\u{1F327}"),
        71 | 73 | 75 => ("Snow", "\u{2744}"),
        77 => ("Snow grains", "\u{2744}"),
        80..=82 => ("Showers", "\u{1F327}"),
        85 | 86 => ("Snow showers", "\u{1F328}"),
        95 => ("Thunderstorm", "\u{26C8}"),
        96 | 99 => ("Thunderstorm", "\u{26C8}"),
        _ => ("Unknown", "\u{2601}"),
    };

    Some(WeatherInfo {
        temperature_f: temp,
        description: description.to_string(),
        icon: icon.to_string(),
    })
}
