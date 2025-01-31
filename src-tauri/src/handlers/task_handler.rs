use crate::state::AppState;
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    id: String,
    content: String,
    day: String,
    completed: bool,
    #[serde(rename = "userId")]
    user_id: String,
    #[serde(rename = "createdAt")]
    created_at: Option<String>,
    #[serde(rename = "updatedAt")]
    updated_at: Option<String>,
}

#[tauri::command]
pub async fn fetch_tasks(
    state: tauri::State<'_, AppState>,
    token: String,
) -> Result<Value, String> {
    log::info!("Fetching all tasks");

    // Get the user ID before any async operations
    let user_id: String = {
        let user_guard = state.existing_user.lock().map_err(|e| e.to_string())?;
        user_guard
            .as_ref()
            .and_then(|u| Some(u.id.clone()))
            .ok_or_else(|| "User not authenticated".to_string())?
    };

    log::info!("User ID: {}", user_id);

    let client = reqwest::Client::new();
    let response = client
        // .get("http://localhost:8787/api/tasks")
        .get("https://jeff-ai-cf-be.mrboutte21.workers.dev/api/tasks")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
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

    Ok(json_value)
}

#[tauri::command]
pub async fn create_task(
    state: tauri::State<'_, AppState>,
    token: String,
    content: String,
    assigned_date: String,
) -> Result<Value, String> {
    log::info!("Creating new task with content: {}", content);

    // Get the user ID before any async operations
    let user_id: String = {
        let user_guard = state.existing_user.lock().map_err(|e| e.to_string())?;
        user_guard
            .as_ref()
            .and_then(|u| Some(u.id.clone()))
            .ok_or_else(|| "User not authenticated".to_string())?
    };

    // Create the task payload matching the API's expected format
    let task_payload = json!({
        "task": content,
        "done": false,
        "userId": user_id,
        "assignedDate": assigned_date
    });

    let client = reqwest::Client::new();
    let response = client
        // .post("http://localhost:8787/api/tasks")
        .post("https://jeff-ai-cf-be.mrboutte21.workers.dev/api/tasks")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&task_payload)
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to create task: {}", e);
            format!("Failed to create task: {}", e)
        })?;

    // Check if the response is successful
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        log::error!("Server returned error: {}", error_text);
        return Err(format!("Server error: {}", error_text));
    }

    let json_value = response.json::<Value>().await.map_err(|e| {
        log::error!("Failed to parse response as JSON: {}", e);
        format!("Failed to parse response: {}", e)
    })?;

    Ok(json_value)
}

#[tauri::command]
pub async fn update_task(
    state: tauri::State<'_, AppState>,
    token: String,
    task_id: u32,
    content: Option<String>,
    completed: Option<bool>,
    day: Option<String>,
) -> Result<Value, String> {
    log::info!("Updating task: {}", task_id);

    let mut update_data = json!({});

    if let Some(content) = content {
        update_data.as_object_mut().unwrap().insert("task".to_string(), json!(content));
    }
    if let Some(completed) = completed {
        update_data.as_object_mut().unwrap().insert("done".to_string(), json!(completed));
    }
    if let Some(day) = day {
        update_data.as_object_mut().unwrap().insert("day".to_string(), json!(day));
    }

    let client = reqwest::Client::new();
    let response = client
        .patch(&format!(
            "https://jeff-ai-cf-be.mrboutte21.workers.dev/api/tasks/{}",
            task_id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&update_data)
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to update task: {}", e);
            e.to_string()
        })?;

    let json_value = response.json::<Value>().await.map_err(|e| {
        log::error!("Failed to parse response as JSON: {}", e);
        e.to_string()
    })?;

    Ok(json_value)
}

#[tauri::command]
pub async fn delete_task(
    state: tauri::State<'_, AppState>,
    token: String,
    task_id: String,
) -> Result<Value, String> {
    log::info!("Deleting task: {}", task_id);

    let client = reqwest::Client::new();
    let response = client
        .delete(&format!(
            "https://jeff-ai-cf-be.mrboutte21.workers.dev/api/tasks/{}",
            task_id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to delete task: {}", e);
            e.to_string()
        })?;

    let json_value = response.json::<Value>().await.map_err(|e| {
        log::error!("Failed to parse response as JSON: {}", e);
        e.to_string()
    })?;

    Ok(json_value)
} 