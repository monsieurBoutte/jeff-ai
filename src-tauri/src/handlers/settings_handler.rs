use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::state::AppState;

// Separate structs for incoming and outgoing data
#[derive(Debug, Serialize, Deserialize)]
pub struct UserSettingsInput {
    lat: Option<f64>,
    lon: Option<f64>,
    city: Option<String>,
    state: Option<String>,
    country: Option<String>,
    units: String,
    language: String,
}

#[derive(Debug, Serialize)]
struct UserSettingsRequest {
    userId: String,
    lat: Option<f64>,
    lon: Option<f64>,
    city: Option<String>,
    state: Option<String>,
    country: Option<String>,
    units: String,
    language: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSettings {
    #[serde(rename = "userId")]
    user_id: String,
    lat: Option<f64>,
    lon: Option<f64>,
    city: Option<String>,
    state: Option<String>,
    country: Option<String>,
    units: String,
    language: String,
}

#[tauri::command]
pub async fn get_user_settings(
    state: tauri::State<'_, AppState>,
    token: String,
) -> Result<Value, String> {
    log::info!("Fetching user settings");

    // Get the user ID before any async operations
    let user_id: String = {
        let user_guard = state.existing_user.lock().map_err(|e| e.to_string())?;
        user_guard
            .as_ref()
            .and_then(|u| Some(u.id.clone()))
            .ok_or_else(|| "User not authenticated".to_string())?
    };

    log::info!("Getting settings for user: {}", user_id);

    let client = reqwest::Client::new();
    let response = client
        .get("https://jeff-ai-cf-be.mrboutte21.workers.dev/api/settings")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to fetch user settings: {}", e);
            e.to_string()
        })?;

    let json_value = response.json::<Value>().await.map_err(|e| {
        log::error!("Failed to parse response as JSON: {}", e);
        e.to_string()
    })?;

    Ok(json_value)
}

#[tauri::command]
pub async fn create_user_settings(
    state: tauri::State<'_, AppState>,
    token: String,
    settings: UserSettingsInput,
) -> Result<Value, String> {
    log::info!("Creating user settings: {:?}", settings);

    // Get the user ID before any async operations
    let user_id: String = {
        let user_guard = state.existing_user.lock().map_err(|e| e.to_string())?;
        user_guard
            .as_ref()
            .and_then(|u| Some(u.id.clone()))
            .ok_or_else(|| "User not authenticated".to_string())?
    };

    // Create the request payload with user_id
    let settings_request = UserSettingsRequest {
        userId: user_id,
        lat: settings.lat,
        lon: settings.lon,
        city: settings.city,
        state: settings.state,
        country: settings.country,
        units: settings.units,
        language: settings.language,
    };

    let client = reqwest::Client::new();
    let response = client
        .post("https://jeff-ai-cf-be.mrboutte21.workers.dev/api/settings")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&settings_request)
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to create user settings: {}", e);
            e.to_string()
        })?;

    let json_value = response.json::<Value>().await.map_err(|e| {
        log::error!("Failed to parse response as JSON: {}", e);
        e.to_string()
    })?;

    Ok(json_value)
}

#[tauri::command]
pub async fn update_user_settings(
    state: tauri::State<'_, AppState>,
    token: String,
    settings: UserSettingsInput,
) -> Result<Value, String> {
    log::info!("Updating user settings: {:?}", settings);

    // Get the user ID before any async operations
    let user_id: String = {
        let user_guard = state.existing_user.lock().map_err(|e| e.to_string())?;
        user_guard
            .as_ref()
            .and_then(|u| Some(u.id.clone()))
            .ok_or_else(|| "User not authenticated".to_string())?
    };

    // Create the request payload with user_id
    let settings_request = UserSettingsRequest {
        userId: user_id,
        lat: settings.lat,
        lon: settings.lon,
        city: settings.city,
        state: settings.state,
        country: settings.country,
        units: settings.units,
        language: settings.language,
    };

    let client = reqwest::Client::new();
    let response = client
        .patch("https://jeff-ai-cf-be.mrboutte21.workers.dev/api/settings")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&settings_request)
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to update user settings: {}", e);
            e.to_string()
        })?;

    let json_value = response.json::<Value>().await.map_err(|e| {
        log::error!("Failed to parse response as JSON: {}", e);
        e.to_string()
    })?;

    Ok(json_value)
}
