use serde_json::{json, Value};
use std::env;
use tauri_plugin_clipboard_manager::ClipboardExt;
use crate::models::RefinedMessage;

#[tauri::command]
pub async fn refine_message(app: tauri::AppHandle, msg: String) -> Result<RefinedMessage, String> {
    let api_key = env::var("GROQ_API_KEY").map_err(|e| {
        log::error!("Failed to get GROQ_API_KEY: {}", e);
        "GROQ_API_KEY not found in environment".to_string()
    })?;

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

    let client = reqwest::Client::new();
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

    let json_response = response.json::<Value>().await.map_err(|e| {
        log::error!("Failed to parse Groq API response as JSON: {}", e);
        e.to_string()
    })?;

    let content = json_response["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| {
            let err = "Failed to get message content from response";
            log::error!("{}", err);
            err.to_string()
        })?;

    let content_json: RefinedMessage = serde_json::from_str(content).map_err(|e| {
        log::error!("Failed to parse message content as RefinedMessage: {}", e);
        e.to_string()
    })?;

    if let Err(e) = app.clipboard().write_text(content_json.suggested_message_rewrite.clone()) {
        log::error!("Failed to write to clipboard: {}", e);
    }

    Ok(content_json)
} 