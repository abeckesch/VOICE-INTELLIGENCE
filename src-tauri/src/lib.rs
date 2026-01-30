use tauri::{
    Emitter, Manager, PhysicalPosition, WindowEvent,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use std::sync::Mutex;

mod llm;
mod skills;
mod input;

/// Application state holding loaded skills
pub struct AppState {
    pub skills: Vec<skills::Skill>,
}

/// Force window to topmost using Windows native API
/// This is more aggressive than Tauri's set_always_on_top
#[cfg(windows)]
fn force_topmost_native_hwnd(hwnd_raw: isize) {
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE, SWP_NOACTIVATE, SWP_SHOWWINDOW, SWP_ASYNCWINDOWPOS,
    };
    use windows::Win32::Foundation::HWND;
    
    let hwnd = HWND(hwnd_raw as *mut _);
    unsafe {
        // Use ASYNCWINDOWPOS for better compatibility with hardware-accelerated windows (browsers)
        let _ = SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0, 0, 0, 0,
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
            let y = screen_position.y + (screen_size.height as i32 - window_size.height as i32 - 80); // 80px margin from bottom to visually clear taskbar
            
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
                force_topmost_native(&window);  // Native Windows API for aggressive topmost
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

/// Process audio bytes from frontend - sends to Groq Whisper API, then to Llama3 for response
#[tauri::command]
async fn process_audio(
    audio_data: Vec<u8>,
    duration_ms: u64,
    state: tauri::State<'_, Mutex<AppState>>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let bytes_received = audio_data.len();
    
    // Log received audio
    println!("🎤 Audio received: {} bytes, duration: {}ms", bytes_received, duration_ms);
    
    if bytes_received == 0 {
        return Err("Keine Audiodaten empfangen".to_string());
    }
    
    // Hybrid Language Heuristic
    const HYBRID_THRESHOLD_MS: u64 = 4000;
    let preferred_language = std::env::var("PREFERRED_LANGUAGE").unwrap_or_else(|_| "de".to_string());
    
    let language = if duration_ms < HYBRID_THRESHOLD_MS {
        println!("🌐 Using preferred language: {} (duration {}ms < {}ms threshold)", 
            preferred_language, duration_ms, HYBRID_THRESHOLD_MS);
        Some(preferred_language)
    } else {
        println!("🌐 Using auto-detect (duration {}ms >= {}ms threshold)", 
            duration_ms, HYBRID_THRESHOLD_MS);
        None
    };
    
    // Step 1: Send to Groq Whisper API for transcription
    println!("📤 Sending to Groq Whisper API...");
    let transcription = match llm::whisper::upload_audio(audio_data, language).await {
        Ok(text) => {
            println!("📝 Transcription: {}", text);
            text
        }
        Err(e) => {
            eprintln!("❌ Whisper API error: {}", e);
            return Err(e);
        }
    };
    
    // Step 2: Build system prompt from loaded skills
    let system_prompt = {
        let app_state = state.lock().map_err(|e| format!("State lock error: {}", e))?;
        llm::groq::build_system_prompt(&app_state.skills)
    };
    println!("🧠 System prompt built with {} skills", 
        state.lock().map(|s| s.skills.len()).unwrap_or(0));
    
    // Step 3: Send to Groq Llama3 for intelligent response
    println!("🤖 Sending to Groq Llama3...");
    let response_text = match llm::groq::chat_completion(&system_prompt, &transcription).await {
        Ok(response) => {
            println!("✨ LLM Response: {}", response);
            response
        }
        Err(e) => {
            eprintln!("❌ LLM API error: {}", e);
            return Err(e);
        }
    };
    
    // Step 4: Copy response to clipboard (backup)
    println!("📋 Copying to clipboard...");
    if let Err(e) = input::injector::copy_to_clipboard(&response_text) {
        eprintln!("⚠ Clipboard error: {}", e);
    }
    
    // Step 5: Hide window to return focus to previous application
    println!("🚪 Hiding window...");
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.hide();
    }
    
    // Step 6: Wait for focus to return to previous window
    std::thread::sleep(std::time::Duration::from_millis(200));
    
    // Step 7: Type the response text at cursor position
    println!("⌨️ Typing response...");
    if let Err(e) = input::injector::type_text(&response_text) {
        eprintln!("❌ Injection error: {}", e);
        // Response is still in clipboard as backup
    }
    
    // Step 8: Emit completion event to frontend
    let _ = app_handle.emit("processing-complete", ());
    println!("✅ Processing complete!");
    
    Ok(response_text)
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
        .invoke_handler(tauri::generate_handler![process_audio])
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
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        toggle_window(tray.app_handle());
                    }
                })
                .build(app)?;

            println!("✓ System Tray initialized");

            // === SKILL LOADER ===
            let exe_path = std::env::current_exe().unwrap_or_default();
            // Skills dir is 4 levels up from target/debug/voice-intelligence.exe -> skills/
            let skills_dir = exe_path
                .parent() // target/debug/
                .and_then(|p| p.parent()) // target/
                .and_then(|p| p.parent()) // src-tauri/
                .and_then(|p| p.parent()) // voice-intelligence/ (root with skills/)
                .map(|p| p.join("skills"))
                .unwrap_or_else(|| std::path::PathBuf::from("skills"));
            
            let loaded_skills = skills::load_skills(&skills_dir);
            if loaded_skills.is_empty() {
                println!("⚠ No skills loaded from {:?}", skills_dir);
            } else {
                let skill_names: Vec<_> = loaded_skills.iter().map(|s| s.name.as_str()).collect();
                println!("✓ Loaded {} skill(s): {}", loaded_skills.len(), skill_names.join(", "));
            }
            
            // Store skills in managed state
            app.manage(Mutex::new(AppState { skills: loaded_skills }));

            println!("✓ Event-based Z-Index enforcement active (no polling)");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
