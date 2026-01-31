//! Groq Llama3 Chat Completion API Client
//!
//! Sends messages to Groq's Chat API for LLM inference with skill-based system prompts.

use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Chat message structure for Groq API
#[derive(Debug, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Request body for Groq Chat Completion API
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

/// Choice in Groq API response
#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

/// Message in response
#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: String,
}

/// Response from Groq Chat Completion API
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

/// Send a chat completion request to generic OpenAI-compatible API (Groq or Ollama)
pub async fn chat_completion(
    system_prompt: &str,
    user_message: &str,
    base_url: &str,
    model: &str,
    api_key: &str,
) -> Result<String, String> {
    let client = Client::new();

    let request = ChatCompletionRequest {
        model: model.to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_message.to_string(),
            },
        ],
        temperature: 0.7,
        max_tokens: 2048,
    };

    let response = client
        .post(format!(
            "{}/chat/completions",
            base_url.trim_end_matches('/')
        ))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Anfrage fehlgeschlagen: {}", e))?;

    // Check for HTTP errors
    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        return Err(format!("API-Fehler {}: {}", status, error_body));
    }

    // Parse response
    let chat_response: ChatCompletionResponse = response
        .json()
        .await
        .map_err(|e| format!("Fehler beim Parsen der Antwort: {}", e))?;

    chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "Keine Antwort vom LLM erhalten".to_string())
}
