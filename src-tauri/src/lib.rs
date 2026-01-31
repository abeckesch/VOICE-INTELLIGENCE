use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, PhysicalPosition, WindowEvent,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

mod input;
mod llm;

/// Application state holding loaded skills
pub struct AppState {
    pub is_recording: bool,
}

/// Force window to topmost using Windows native API
/// This is more aggressive than Tauri's set_always_on_top
#[cfg(windows)]
fn force_topmost_native_hwnd(hwnd_raw: isize) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_TOPMOST, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        SWP_SHOWWINDOW,
    };

    let hwnd = HWND(hwnd_raw as *mut _);
    unsafe {
        // Use ASYNCWINDOWPOS for better compatibility with hardware-accelerated windows (browsers)
        let _ = SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_ASYNCWINDOWPOS,
        );
    }
}

#[cfg(windows)]
fn force_topmost_native(window: &tauri::WebviewWindow) {
    if let Ok(hwnd) = window.hwnd() {
        force_topmost_native_hwnd(hwnd.0 as isize);
    }
}

#[cfg(windows)]
fn force_topmost_window(window: &tauri::Window) {
    if let Ok(hwnd) = window.hwnd() {
        force_topmost_native_hwnd(hwnd.0 as isize);
    }
}

#[cfg(not(windows))]
fn force_topmost_native(_window: &tauri::WebviewWindow) {}

#[cfg(not(windows))]
fn force_topmost_window(_window: &tauri::Window) {}

/// Position window at bottom-center of the screen
fn position_window_bottom_center(window: &tauri::WebviewWindow) {
    if let Ok(Some(monitor)) = window.current_monitor() {
        let screen_size = monitor.size();
        let screen_position = monitor.position();

        if let Ok(window_size) = window.outer_size() {
            let x = screen_position.x + ((screen_size.width as i32 - window_size.width as i32) / 2);
            let y =
                screen_position.y + (screen_size.height as i32 - window_size.height as i32 - 80); // 80px margin from bottom to visually clear taskbar

            let _ = window.set_position(PhysicalPosition::new(x, y));
        }
    }
}

/// Toggle window visibility - shows if hidden, hides if visible
fn toggle_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        match window.is_visible() {
            Ok(true) => {
                // Emit event to frontend to stop recording before hiding
                let _ = app.emit("window-hiding", ());
                // Small delay to allow frontend to process, then hide
                std::thread::spawn({
                    let window = window.clone();
                    move || {
                        std::thread::sleep(std::time::Duration::from_millis(50));
                        let _ = window.hide();
                    }
                });
            }
            Ok(false) => {
                position_window_bottom_center(&window);
                // Smart Z-Index enforcement on Show event using native Windows API
                let _ = window.show();
                let _ = window.set_always_on_top(true);
                force_topmost_native(&window); // Native Windows API for aggressive topmost
                let _ = window.set_focus();
                // Emit event to frontend to start recording
                let _ = app.emit("window-shown", ());
            }
            Err(e) => {
                eprintln!("Failed to check window visibility: {}", e);
            }
        }
    }
}

/// Check if audio is silent based on RMS threshold (16-bit PCM)
fn is_silent(audio_samples: &[i16], threshold: f32) -> bool {
    if audio_samples.is_empty() {
        return true;
    }

    let mut sum_squares = 0.0;
    for &sample in audio_samples {
        sum_squares += (sample as f32).powi(2);
    }
    let mean_square = sum_squares / audio_samples.len() as f32;
    let rms = mean_square.sqrt();

    rms < threshold
}

/// Check if text matches known hallucinations
fn is_hallucination(text: &str) -> bool {
    let clean_text = text.trim();
    if clean_text.len() < 2 {
        return true;
    }

    let hallucinations = [
        "Thank you.",
        "Thank you for watching",
        "Subtitles by",
        "Thanks.",
        "MBC",
        "Untertitel der Amara.org-Community",
        "Sous-titres réalisés par",
        "Lädt...",
        "Vielen Dank.",
        "Vielen Dank für Ihre Aufmerksamkeit.",
    ];

    // Case-insensitive check? The user gave specific strings with punctuation.
    // Let's do a fast "contains" or "equals" check.
    // Given Whisper hallucinations are often EXACT lines, we check both.
    for &h in &hallucinations {
        if clean_text.contains(h) {
            return true;
        }
    }

    false
}

/// Process audio bytes from frontend - sends to Groq Whisper API, then to Llama3 for response
#[tauri::command]
async fn process_audio(
    audio_data: Vec<u8>,
    duration_ms: u64,
    privacy_mode: bool,
    whisper_path: String,
    model_path: String,
    ffmpeg_path: String,
    language: String, // New parameter
    skill: String,    // New parameter (auto, cleanup, todo, summary)
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let bytes_received = audio_data.len();

    // Log received audio (Concise)
    println!(
        "🎤 Input: {}ms | Language: {} | Active Skill: {}",
        duration_ms, language, skill
    );

    if bytes_received == 0 {
        return Err("Keine Audiodaten empfangen".to_string());
    }

    // Language Logic
    // Cloud Whisper: None = Auto-Detect
    // Local Whisper: "auto" = Auto-Detect (otherwise defaults to English on some builds)
    let (target_lang_cloud, target_lang_local) = if language == "auto" {
        (None, "auto".to_string())
    } else {
        (Some(language.clone()), language.clone())
    };

    // === PHASE 9: SILENCE GUARD (VAD Lite) ===
    // 1. Write audio to temp file (universally needed for VAD)
    let temp_dir = std::env::temp_dir();
    let audio_path = temp_dir.join("voice_intelligence_temp_audio.webm");
    let wav_path = temp_dir.join("voice_intelligence_temp_audio.wav");

    // std::fs::write(&audio_path, &audio_data)...
    if let Err(e) = std::fs::write(&audio_path, &audio_data) {
        eprintln!("❌ Failed to write temp audio for VAD: {}", e);
        return Err(e.to_string());
    }

    // 2. Decode WebM to 16kHz Mono PCM S16LE via ffmpeg to STDOUT
    // Use configured path or default (we need to be able to run this!)
    let ffmpeg_cmd = if ffmpeg_path.is_empty() {
        "ffmpeg"
    } else {
        &ffmpeg_path
    };

    let vad_output = std::process::Command::new(ffmpeg_cmd)
        .arg("-i")
        .arg(&audio_path)
        .arg("-ar")
        .arg("16000") // 16 kHz
        .arg("-ac")
        .arg("1") // Mono
        .arg("-f")
        .arg("s16le") // Raw PCM
        .arg("-") // Pipe to stdout
        .output();

    match vad_output {
        Ok(output) if output.status.success() => {
            let pcm_bytes = output.stdout;
            // Convert bytes to i16
            let pcm_samples: Vec<i16> = pcm_bytes
                .chunks_exact(2)
                .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
                .collect();

            // Threshold: 100-300 recommended. Let's start with 150.
            if is_silent(&pcm_samples, 150.0) {
                println!("🔇 Silence detected (RMS < 150). Aborting.");
                // Cleanup
                let _ = std::fs::remove_file(&audio_path);
                let _ = std::fs::remove_file(&wav_path);
                // Clean return - frontend ignores empty string? Or we assume so.
                return Ok("".to_string());
            } else {
            }
        }
        Err(e) => {
            eprintln!(
                "⚠ VAD Failed (ffmpeg error): {}. Proceeding without Silence Guard.",
                e
            );
        }
        Ok(_) => {}
    }

    // Step 1: STT
    let transcription = if privacy_mode {
        println!("🛡️ STT (Local): {}", model_path);

        // Validation
        if whisper_path.is_empty() || model_path.is_empty() {
            eprintln!("❌ Error: Local Whisper configuration missing!");
            return Err("Settings error: Local Whisper binaries not configured. Please check 'Local Mode' in settings.".to_string());
        }

        // 1. File is already written at logic start!
        // std::fs::write(&audio_path, &audio_data) ... (Done above)

        // 2. Convert WebM to WAV (16kHz) using ffmpeg
        let ffmpeg_cmd = if ffmpeg_path.is_empty() {
            "ffmpeg"
        } else {
            &ffmpeg_path
        };

        // Quietly convert...
        let ffmpeg_output = std::process::Command::new(ffmpeg_cmd)
            .arg("-y") // Overwrite
            .arg("-i")
            .arg(&audio_path)
            .arg("-ar")
            .arg("16000") // 16 kHz
            .arg("-ac")
            .arg("1") // Mono
            .arg("-c:a")
            .arg("pcm_s16le") // Signed 16-bit little endian
            .arg(&wav_path)
            .output()
            .map_err(|e| format!("Fehler beim Ausführen von ffmpeg: {}", e))?;

        if !ffmpeg_output.status.success() {
            let err_msg = String::from_utf8_lossy(&ffmpeg_output.stderr);
            return Err(format!("FFmpeg Konvertierungsfehler: {}", err_msg));
        }

        // 3. Run Whisper CLI
        let mut whisper_cmd = std::process::Command::new(&whisper_path);
        whisper_cmd
            .arg("-m")
            .arg(&model_path)
            .arg("-f")
            .arg(&wav_path)
            .arg("--no-timestamps")
            .arg("-l")
            .arg(&target_lang_local); // Always pass language (explicit or auto)

        let whisper_output = whisper_cmd
            .output()
            .map_err(|e| format!("Fehler beim Ausführen von {}: {}", whisper_path, e))?;

        if !whisper_output.status.success() {
            let err_msg = String::from_utf8_lossy(&whisper_output.stderr);
            return Err(format!("Whisper CLI Fehler: {}", err_msg));
        }

        let raw_text = String::from_utf8_lossy(&whisper_output.stdout).to_string();
        let clean_text = raw_text.trim().to_string();

        println!("\n✨ TRANSCRIPTION (Local):\n{}\n", clean_text);

        // Cleanup temp files (best effort)
        let _ = std::fs::remove_file(&audio_path);
        let _ = std::fs::remove_file(&wav_path);

        if clean_text.is_empty() {
            // Maybe it outputted to stderr or something else?
            // Without -nt usually it prints to stdout.
            // Let's assume user has a working setup.
            return Err("Lokale Transkription war leer.".to_string());
        }

        clean_text
    } else {
        // Standard Mode: Groq Whisper
        println!("☁️ STT (Cloud): Groq Whisper (whisper-large-v3)");
        match llm::whisper::upload_audio(audio_data, target_lang_cloud.clone()).await {
            Ok(text) => {
                println!("\n✨ TRANSCRIPTION (Cloud):\n{}\n", text);
                text
            }
            Err(e) => {
                eprintln!("❌ Whisper API error: {}", e);
                return Err(e);
            }
        }
    };

    // === PHASE 9: OUTPUT FILTER (Hallucination Check) ===
    if is_hallucination(&transcription) {
        println!(
            "🧠 Hallucination detected ('{}'). Filtering output.",
            transcription.trim()
        );
        // Cleanup temp file if it exists
        let _ = std::fs::remove_file(&audio_path);
        let _ = std::fs::remove_file(&wav_path);
        return Ok("".to_string());
    }

    // Step 2: Build system prompt from loaded skills

    let system_prompt =
        llm::prompt::build_system_prompt(target_lang_cloud.clone(), Some(skill.clone()));

    // Step 3: Configure Backend (Groq vs Ollama)
    let (base_url, model, api_key, mode_label) = if privacy_mode {
        (
            "http://localhost:11434/v1",
            "llama3",
            "ollama".to_string(),
            "🛡️ LLM (Local)",
        )
    } else {
        let key = std::env::var("GROQ_API_KEY").map_err(|_| "GROQ_API_KEY nicht gesetzt")?;
        (
            "https://api.groq.com/openai/v1",
            "llama-3.3-70b-versatile",
            key,
            "☁️ LLM (Cloud)",
        )
    };

    // Step 4: Send to LLM
    println!("{}: {}", mode_label, model);
    println!("🤖 Generiere Antwort...");
    let response_text = match llm::groq::chat_completion(
        &system_prompt,
        &transcription,
        base_url,
        model,
        &api_key,
    )
    .await
    {
        Ok(response) => {
            println!("\n💬 LLM RESPONSE:\n----------------------------------------\n{}\n----------------------------------------\n", response);
            response
        }
        Err(e) => {
            eprintln!("❌ LLM API error: {}", e);
            return Err(e);
        }
    };

    // Step 5: Copy response to clipboard (backup)

    if let Err(e) = input::injector::copy_to_clipboard(&response_text) {
        eprintln!("⚠ Clipboard error: {}", e);
    }

    // Step 6: Hide window to return focus to previous application

    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.hide();
    }

    // Step 7: Wait for focus to return to previous window
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Step 8: Type the response text at cursor position

    if let Err(e) = input::injector::type_text(&response_text) {
        eprintln!("❌ Injection error: {}", e);
        // Response is still in clipboard as backup
    }

    // Step 9: Emit completion event to frontend
    let _ = app_handle.emit("processing-complete", ());

    Ok(response_text)
}

/// Test if local configuration is valid (binary runs, model exists)
#[tauri::command]
async fn test_local_configuration(
    whisper_path: String,
    model_path: String,
) -> Result<String, String> {
    // 1. Check Model Path
    let model_pb = std::path::PathBuf::from(&model_path);
    if !model_pb.exists() {
        return Err(format!("Modell-Datei nicht gefunden: {}", model_path));
    }

    // 2. Check Binary Execution (getting help or version)
    // whisper-cli --help usually returns exit code 0 or 1 depending on impl, but stdout/stderr should have content.
    // Most CLIs return success on help, or at least run.
    // If we got here, execution worked (it was found and ran).
    let _output = std::process::Command::new(&whisper_path)
        .arg("--help")
        .output()
        .map_err(|e| format!("Binary kann nicht ausgeführt werden: {}", e))?;

    // Most CLIs return success on help, or at least run.
    // If we got here, execution worked (it was found and ran).

    Ok("Konfiguration erfolgreich verifiziert! ✅".to_string())
}

/// Expand or shrink window for UI overlays (like settings)
#[tauri::command]
async fn hide_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = app.emit("window-hiding", ());
        std::thread::spawn({
            let window = window.clone();
            move || {
                std::thread::sleep(std::time::Duration::from_millis(50));
                let _ = window.hide();
            }
        });
    }
}

#[tauri::command]
async fn set_window_expand(expand: bool, window: tauri::WebviewWindow) -> Result<(), String> {
    let width = 400.0;
    let height = if expand { 700.0 } else { 60.0 };

    // Resize
    if let Err(e) = window.set_size(tauri::Size::Logical(tauri::LogicalSize { width, height })) {
        return Err(format!("Failed to resize: {}", e));
    }

    // Reposition to keep bottom anchor stable
    // We need to run this on the main thread to ensure coordinates are fresh after resize?
    // Actually set_size is async-ish on some platforms, let's wait a tiny bit or just call position
    position_window_bottom_center(&window);

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    // Only handle key press, not release
                    if event.state() == ShortcutState::Pressed {
                        // Alt+Space hotkey
                        let alt_space = Shortcut::new(Some(Modifiers::ALT), Code::Space);
                        if shortcut == &alt_space {
                            toggle_window(app);
                        }
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            process_audio,
            set_window_expand,
            test_local_configuration,
            hide_window
        ])
        // Smart Z-Index: Re-assert always_on_top on ALL Focus events
        // When we gain focus: ensure we're on top
        // When we lose focus (blur) while visible: immediately re-assert to stay on top
        .on_window_event(|window, event| {
            if let WindowEvent::Focused(focused) = event {
                // Re-assert on both focus gain AND focus loss (blur)
                if let Ok(true) = window.is_visible() {
                    let _ = window.set_always_on_top(true);
                    force_topmost_window(window);

                    // On blur: aggressive re-assertion with longer delays for browser compatibility
                    if !*focused {
                        let win = window.clone();
                        std::thread::spawn(move || {
                            // Longer delays to catch Windows DWM compositor and hardware-accelerated browsers
                            for delay in [20, 50, 150, 300, 500] {
                                std::thread::sleep(std::time::Duration::from_millis(delay));
                                if let Ok(true) = win.is_visible() {
                                    let _ = win.set_always_on_top(true);
                                    force_topmost_window(&win);
                                }
                            }
                        });
                    }
                }
            }
        })
        .setup(|app| {
            // Load .env file for API keys
            if let Err(e) = dotenvy::dotenv() {
                eprintln!("Warning: Could not load .env file: {}", e);
            }

            // Register Alt+Space hotkey on startup
            let alt_space = Shortcut::new(Some(Modifiers::ALT), Code::Space);

            if let Err(e) = app.global_shortcut().register(alt_space) {
                eprintln!("Failed to register Alt+Space hotkey: {}", e);
            } else {
                println!("✓ Registered global hotkey: Alt+Space");
            }

            // === SYSTEM TRAY SETUP ===
            let show_hide = MenuItem::with_id(app, "show_hide", "Show/Hide", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_hide, &quit])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show_hide" => toggle_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_window(tray.app_handle());
                    }
                })
                .build(app)?;

            // Store shared state (currently empty, but keeping pattern)
            app.manage(Mutex::new(AppState {
                is_recording: false,
            }));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
