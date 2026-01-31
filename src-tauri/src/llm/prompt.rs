pub fn build_system_prompt(
    target_language: Option<String>,
    active_skill_id: Option<String>,
) -> String {
    let mut prompt = String::new();

    // Unified System Persona (Hardened for consistency)
    let silent_editor_rules = r#"
You are a text processing API. You are NOT a chat assistant.
Your goal is to correct grammar, punctuation, and capitalization of the user's text.

EXAMPLES:
Input: "hello this is a test"
Output: "Hello, this is a test."

Input: "das ist ein haus"
Output: "Das ist ein Haus."

Input: "ich bin müde today is a good day"
Output: "Ich bin müde. Today is a good day."

RULES:
1. NO conversational filler (e.g., "Here is the text", "Sure").
2. Output ONLY the processed text.
3. PRESERVE the original language by default (unless overridden below).
4. Do NOT translate.
5. Remove stuttering (uhm, ah).
"#;

    // Hardcoded "Skills"
    // 1. Check if an explicit skill is requested/active
    if let Some(id) = active_skill_id {
        match id.as_str() {
            "email" => {
                prompt.push_str("You are a PROFESSIONAL EMAIL ASSISTANT.\n\n");
                prompt.push_str("INSTRUCTION:\n");
                prompt.push_str("Draft a professional email based on the input text.\n");
                prompt.push_str("Structure: Subject Line, Salutation, Body, Closing.\n");
                prompt.push_str("Keep the tone professional and polite.\n\n");
                prompt.push_str("IMPORTANT: Output ONLY the email draft.");
            }
            "todo" => {
                prompt.push_str("You are a PROJECT MANAGER (Action Item Extraction).\n\n");
                prompt.push_str("INSTRUCTION:\n");
                prompt.push_str("Extract all actionable tasks found in the input text.\n");
                prompt.push_str("Output a Markdown checklist.\n");
                prompt.push_str("Format: '- [ ] Task description'\n\n");
                prompt.push_str("IMPORTANT: Output ONLY the checklist. No intro/outro.");
            }
            "summary" => {
                prompt.push_str("You are a SUMMARIZER.\n\n");
                prompt.push_str("INSTRUCTION:\n");
                prompt.push_str("Summarize the input text into concise bullet points.\n");
                prompt.push_str("Capture the main ideas and key details.\n\n");
                prompt.push_str("IMPORTANT: Output ONLY the summary.");
            }
            _ => {
                // "auto" or unknown -> Default Silent Editor
                prompt.push_str("You are a SILENT TRANSCRIPTION EDITOR.\n\n");
                prompt.push_str(silent_editor_rules);
            }
        }
    } else {
        // Fallback None -> Default
        prompt.push_str("You are a SILENT TRANSCRIPTION EDITOR.\n\n");
        prompt.push_str(silent_editor_rules);
    }

    // Force Language Output if explicit
    if let Some(lang) = target_language {
        prompt.push_str(&format!(
            "\nCRITICAL: You MUST output in language code '{}'. Do NOT switch languages.",
            lang
        ));
    }

    prompt
}
