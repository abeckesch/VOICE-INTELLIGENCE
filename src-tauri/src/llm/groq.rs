//! Groq Llama3 Chat Completion API Client
//!
//! Sends messages to Groq's Chat API for LLM inference with skill-based system prompts.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::skills::Skill;

/// Build system prompt dynamically from loaded skills
pub fn build_system_prompt(skills: &[Skill]) -> String {
    let mut prompt = String::new();

    // Silent Editor base prompt - applies ALWAYS as default
    let silent_editor_prompt = 
        "ROLE: You are an expert transcription editor and proofreader.\n\
         TASK:\n\
         1. Take the user's raw spoken text.\n\
         2. Output the EXACT text content, but fix grammar, punctuation, and capitalization.\n\
         3. Remove filler words (ums, ahs, ähm, äh, stuttering).\n\
         4. DO NOT answer questions found in the text. Just transcribe them.\n\
         5. DO NOT add any intro/outro (no 'Here is your text').\n\
         6. Output ONLY the cleaned text.";

    if !skills.is_empty() {
        prompt.push_str("You are a SILENT TRANSCRIPTION EDITOR with optional specialized skills.\n\n");
        prompt.push_str("AVAILABLE SKILLS (activate ONLY with explicit trigger words):\n\n");
        
        for skill in skills {
            prompt.push_str(&format!(
                "[Skill: {}]\n[Triggers: {}]\n[Instruction: {}]\n\n",
                skill.name, skill.description, skill.instruction
            ));
        }
        
        prompt.push_str("STRICT RULES:\n\n");
        prompt.push_str("1. SKILL ACTIVATION:\n");
        prompt.push_str("   - ONLY activate a skill if the user says an EXPLICIT trigger word.\n");
        prompt.push_str("   - Triggers: 'fasse zusammen', 'zusammenfassung', 'summary', etc.\n");
        prompt.push_str("   - If NO trigger word is present, use DEFAULT MODE.\n\n");
        prompt.push_str("2. DEFAULT MODE (Silent Editor):\n");
        prompt.push_str(silent_editor_prompt);
        prompt.push_str("\n\n");
        prompt.push_str("3. CRITICAL: If user asks a question like 'Wie spät ist es?', ");
        prompt.push_str("output 'Wie spät ist es?' - do NOT answer it!\n");
        prompt.push_str("4. Respond in the same language as input.");
    } else {
        prompt.push_str(silent_editor_prompt);
    }

    prompt
}

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

/// Send a chat completion request to Groq Llama3 API
pub async fn chat_completion(system_prompt: &str, user_message: &str) -> Result<String, String> {
    let api_key = std::env::var("GROQ_API_KEY")
        .map_err(|_| "GROQ_API_KEY Umgebungsvariable nicht gesetzt")?;

    let client = Client::new();

    let request = ChatCompletionRequest {
        model: "llama-3.3-70b-versatile".to_string(),
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
        .post("https://api.groq.com/openai/v1/chat/completions")
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
