use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherLocation {
    city: String,
    state: String,
    country: String,
}

#[tauri::command]
pub async fn get_weather_location(
    token: String,
    location: WeatherLocation,
) -> Result<Value, String> {
    log::info!("Fetching weather location for: {:?}", location);

    let query = format!("{},{},{}", location.city, location.state, location.country);
    let url = format!(
        "https://jeff-ai-cf-be.mrboutte21.workers.dev/api/weather/geocode?q={}",
        query
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to fetch weather location: {}", e);
            e.to_string()
        })?;

    let json_value = response.json::<Value>().await.map_err(|e| {
        log::error!("Failed to parse response as JSON: {}", e);
        e.to_string()
    })?;

    Ok(json_value)
} 