use crate::state::AppState;
use crate::models::User;
use serde_json::{json, Value};
use reqwest;

#[tauri::command]
pub async fn set_user(state: tauri::State<'_, AppState>, user_data: User) -> Result<(), String> {
    let mut user = state.user.lock().map_err(|e| e.to_string())?;
    *user = Some(user_data);
    Ok(())
}

#[tauri::command]
pub async fn capture_user(token: String, state: tauri::State<'_, AppState>) -> Result<Value, String> {
    let client = reqwest::Client::new();
    let user_data = {
        let current_user = state.user.lock().map_err(|e| e.to_string())?;
        current_user.as_ref()
            .map(|u| (u.id.clone(), u.email.clone(), format!("{} {}", u.given_name, u.family_name)))
            .ok_or_else(|| "No user found in state".to_string())?
    };

    let response = client
        .post("https://jeff-ai-cf-be.mrboutte21.workers.dev/api/capture")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&json!({
            "authUserId": user_data.0,
            "email": user_data.1,
            "displayName": user_data.2
        }))
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to create capture: {}", e);
            e.to_string()
        })?;

    let json_value = response.json::<Value>().await.map_err(|e| {
        log::error!("Failed to parse response as JSON: {}", e);
        e.to_string()
    })?;

    Ok(json_value)
}

#[tauri::command]
pub async fn fetch_tasks(token: String) -> Result<Value, String> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://jeff-ai-cf-be.mrboutte21.workers.dev/api/tasks")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to fetch tasks: {}", e);
            e.to_string()
        })?;

    let json_value = response.json::<Value>().await.map_err(|e| {
        log::error!("Failed to parse response as JSON: {}", e);
        e.to_string()
    })?;

    log::info!("Raw API response: {:?}", json_value);
    Ok(json_value)
} 