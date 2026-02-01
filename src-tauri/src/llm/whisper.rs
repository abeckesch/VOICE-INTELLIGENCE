//! Groq Whisper v3 Turbo API Client
//!
//! Sends audio data to Groq's Whisper API for speech-to-text transcription.

use reqwest::{multipart, Client};
use serde::Deserialize;

/// Response structure from Groq Whisper API
#[derive(Debug, Deserialize)]
pub struct WhisperResponse {
    pub text: String,
}

/// Upload audio bytes to Groq Whisper API and return transcription
pub async fn upload_audio(audio_data: Vec<u8>, language: Option<String>) -> Result<String, String> {
    let api_key = std::env::var("GROQ_API_KEY")
        .map_err(|_| "GROQ_API_KEY Umgebungsvariable nicht gesetzt")?;

    let client = Client::new();

    // Create multipart form with audio file
    let audio_part = multipart::Part::bytes(audio_data)
        .file_name("audio.webm")
        .mime_str("audio/webm")
        .map_err(|e| format!("Fehler beim Erstellen des Audio-Teils: {}", e))?;

    let mut form = multipart::Form::new()
        .text("model", "whisper-large-v3-turbo")
        .text("response_format", "json")
        .part("file", audio_part);

    // Add optional language
    if let Some(lang) = language {
        form = form.text("language", lang);
    }

    // Send request to Groq API
    let response = client
        .post("https://api.groq.com/openai/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
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
    let whisper_response: WhisperResponse = response
        .json()
        .await
        .map_err(|e| format!("Fehler beim Parsen der Antwort: {}", e))?;

    Ok(whisper_response.text)
}
