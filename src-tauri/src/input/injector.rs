//! Text Injection Module
//!
//! Injects text at the current cursor position using clipboard + paste.

use enigo::{Enigo, Key, Keyboard, Settings};
use arboard::Clipboard;

/// Copy text to system clipboard
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("Clipboard Initialisierung fehlgeschlagen: {}", e))?;
    
    clipboard.set_text(text)
        .map_err(|e| format!("Clipboard Kopieren fehlgeschlagen: {}", e))?;
    
    Ok(())
}

/// Type text at the current cursor position using clipboard paste
/// This is more reliable than character-by-character for special chars and multi-line text
pub fn type_text(text: &str) -> Result<(), String> {
    // Step 1: Copy text to clipboard
    copy_to_clipboard(text)?;
    
    // Step 2: Small delay to ensure clipboard is ready
    std::thread::sleep(std::time::Duration::from_millis(50));
    
    // Step 3: Simulate Ctrl+V to paste
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Enigo Initialisierung fehlgeschlagen: {}", e))?;
    
    // Press Ctrl+V
    enigo.key(Key::Control, enigo::Direction::Press)
        .map_err(|e| format!("Ctrl press failed: {}", e))?;
    enigo.key(Key::Unicode('v'), enigo::Direction::Click)
        .map_err(|e| format!("V click failed: {}", e))?;
    enigo.key(Key::Control, enigo::Direction::Release)
        .map_err(|e| format!("Ctrl release failed: {}", e))?;
    
    Ok(())
}
