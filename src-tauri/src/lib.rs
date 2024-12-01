#[cfg(debug_assertions)]
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::env;
use tauri_plugin_clipboard_manager::ClipboardExt;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Debug, Serialize, Deserialize)]
struct RefinedMessage {
    suggested_message_rewrite: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    id: i32,
    name: String,
    done: bool,
    #[serde(rename = "userId")]
    user_id: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
}

#[tauri::command]
async fn refine_message(app: tauri::AppHandle, msg: String) -> Result<RefinedMessage, String> {
    let api_key = env::var("GROQ_API_KEY").map_err(|e| {
        log::error!("Failed to get GROQ_API_KEY: {}", e);
        "GROQ_API_KEY not found in environment".to_string()
    })?;

    // Construct the request body
    let body = json!({
        "messages": [
            {
                "role": "system",
                "content": r###"You are a message refinement API. Your goal is to suggest a more articulate, polished version of the original message, maintaining a professional tone without sounding overly formal. If the message has a casual flair, keep some of its original character so it still feels like the author.

                Your response should be in JSON format:
                {
                "suggested_message_rewrite": "string (a refined version of the original message, with improved clarity and a tone that balances professionalism with the original casual feel)"
                }

                Guidelines:
                1. Retain the message's intent and main points, improving clarity and flow.
                2. Preserve any casual flair to maintain the author's voice, but ensure the tone is articulate and approachable.
                3. Respond in valid JSON format."###
            },
            {
                "role": "user",
                "content": format!("help me analyze the following:\n\n\"\"\"\n{}\n\"\"\"", msg)
            }
        ],
        "model": "llama-3.2-90b-text-preview",
        "temperature": 1,
        "max_tokens": 1024,
        "top_p": 1,
        "stream": false,
        "response_format": {
            "type": "json_object"
        },
        "stop": null
    });

    // Create the client
    let client = reqwest::Client::new();

    // Make the request
    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to send request to Groq API: {}", e);
            e.to_string()
        })?;

    // Parse the response as JSON
    let json_response = response.json::<Value>().await.map_err(|e| {
        log::error!("Failed to parse Groq API response as JSON: {}", e);
        e.to_string()
    })?;

    // Extract the content string from the first choice
    let content = json_response["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| {
            let err = "Failed to get message content from response";
            log::error!("{}", err);
            err.to_string()
        })?;

    // Parse the content string as JSON to get the suggested_message_rewrite
    let content_json: RefinedMessage = serde_json::from_str(content).map_err(|e| {
        log::error!("Failed to parse message content as RefinedMessage: {}", e);
        e.to_string()
    })?;

    // write the refined message to the clipboard
    if let Err(e) = app.clipboard().write_text(content_json.suggested_message_rewrite.clone()) {
        log::error!("Failed to write to clipboard: {}", e);
    }

    // Return the entire RefinedMessage struct
    Ok(content_json)
}

#[tauri::command]
async fn fetch_tasks(token: String) -> Result<Value, String> {
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

    // Get the raw JSON value
    let json_value = response.json::<Value>().await.map_err(|e| {
        log::error!("Failed to parse response as JSON: {}", e);
        e.to_string()
    })?;

    log::info!("Raw API response: {:?}", json_value);
    Ok(json_value)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    dotenv().ok();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("app.log".to_string()),
                    },
                ))
                .build()
        )
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet, refine_message, fetch_tasks])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
