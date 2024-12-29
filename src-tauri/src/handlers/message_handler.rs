use serde_json::{json, Value};
use crate::state::AppState;

#[tauri::command]
pub async fn refine_text(
    state: tauri::State<'_, AppState>,
    token: String,
    text: String,
    context: Option<String>
) -> Result<Value, String> {
    log::info!("Refining text: {}", text);


    // Get the user ID before any async operations
    let user_id: String = {
        let user_guard = state.existing_user.lock().map_err(|e| e.to_string())?;
        user_guard.as_ref()
            .and_then(|u| Some(u.id.clone()))
            .ok_or_else(|| "User not authenticated".to_string())?
    };

    log::info!("User ID: {}", user_id);

    let client = reqwest::Client::new();
    let response = client
        .post("https://jeff-ai-cf-be.mrboutte21.workers.dev/api/refinements")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&json!({
            "originalText": text,
            "additionalContext": context,
            "userId": user_id
        }))
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to create refinement: {}", e);
            e.to_string()
        })?;

    let json_value = response.json::<Value>().await.map_err(|e| {
        log::error!("Failed to parse response as JSON: {}", e);
        e.to_string()
    })?;

    Ok(json_value)
}

#[tauri::command]
pub async fn convert_to_markdown(
    state: tauri::State<'_, AppState>,
    token: String,
    html: String,
) -> Result<Value, String> {
    log::info!("Converting the following into markdown: {}", html);


    // Get the user ID before any async operations
    let user_id: String = {
        let user_guard = state.existing_user.lock().map_err(|e| e.to_string())?;
        user_guard.as_ref()
            .and_then(|u| Some(u.id.clone()))
            .ok_or_else(|| "User not authenticated".to_string())?
    };

    log::info!("User ID: {}", user_id);

    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:8787/api/refinements/convert-to-markdown")
        // .post("https://jeff-ai-cf-be.mrboutte21.workers.dev/api/refinements")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&json!({
            "html": html,
        }))
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to convert to markdown: {}", e);
            e.to_string()
        })?;

    let json_value = response.json::<Value>().await.map_err(|e| {
        log::error!("Failed to parse response as JSON: {}", e);
        e.to_string()
    })?;

    Ok(json_value)
}