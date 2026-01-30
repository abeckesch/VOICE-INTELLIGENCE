# Detaillierte Implementierungsaufgaben

## Phase 1: Setup & Skeleton (Projekt-Initialisierung)
- [x] Initialisiere Tauri v2 Projekt mit `create-tauri-app` (Next.js/TypeScript) <!-- id: 100 -->
- [x] Installiere TailwindCSS und konfiguriere `postcss.config.js` für das Styling <!-- id: 101 -->
- [x] Bereinige Standard-Boilerplate-Code (React & Rust) <!-- id: 102 -->
- [x] Konfiguriere `tauri.conf.json` für Support transparenter Fenster <!-- id: 103 -->
- [x] Setze `alwaysOnTop` auf true in der Tauri-Konfiguration <!-- id: 104 -->
- [x] Füge `tauri-plugin-global-shortcut` Abhängigkeit zu `src-tauri/Cargo.toml` hinzu <!-- id: 105 -->
- [x] Registriere globalen Hotkey (`Alt+Space`) in `lib.rs` <!-- id: 106 -->
- [x] Implementiere `toggle_window` Logik im Rust Command Handler <!-- id: 107 -->
- [x] Verifiziere, dass der Hotkey die Fenstersichtbarkeit erfolgreich umschaltet <!-- id: 108 --> ✅ Verifiziert

## Phase 2: Core Loop (Audio & API) - GROQ-ONLY STRATEGIE
- [x] Füge `reqwest` (Features: json, multipart) zu `Cargo.toml` hinzu <!-- id: 200 -->
- [x] Füge `tokio` (Features: full) zu `Cargo.toml` hinzu <!-- id: 201 -->
- [x] Erstelle Verzeichnis `src-tauri/src/llm/` <!-- id: 202 -->
- [x] Erstelle Modul `src-tauri/src/llm/whisper.rs` <!-- id: 203 -->
- [x] Definiere Rust-Struktur für Whisper API Response (text Feld) <!-- id: 204 -->
- [x] Implementiere `upload_audio` Funktion mit `reqwest::Client` <!-- id: 205 -->
- [x] Konfiguriere `upload_audio` für den Groq Whisper API Endpunkt <!-- id: 206 -->
- [x] Implementiere Konstruktion von Multipart Form Data für Audio-Dateien <!-- id: 207 -->
- [x] Erstelle `useAudioRecorder` Hook in React zur Erfassung von Mikrofoneingaben <!-- id: 208 -->
- [x] Implementiere `MediaRecorder` Logik zur Ausgabe von Audio Blobs <!-- id: 209 -->
- [x] Erstelle Tauri Command `process_audio`, um Byte-Arrays zu akzeptieren <!-- id: 210 -->
- [x] Verknüpfe Frontend Audio Blob Übertragung mit dem Backend Command <!-- id: 211 -->
- [x] Rufe `whisper::upload_audio` von `process_audio` auf und logge das Ergebnis <!-- id: 212 --> ✅
- [x] Implementiere Hybrid Language Heuristic (Dauer-basierte Sprachauswahl) <!-- id: 213 --> ✅

## Phase 2.5: System Tray & HUD Architecture
- [x] Setze `skipTaskbar: true` in tauri.conf.json <!-- id: 250 --> ✅
- [x] Füge `tauri-plugin-tray-icon` zu Cargo.toml hinzu <!-- id: 251 --> ✅
- [x] Implementiere System Tray mit Show/Hide und Quit Menü <!-- id: 252 --> ✅
- [x] Konfiguriere Fenster für "Widget-Verhalten" (kein Fokus-Klau beim Start) <!-- id: 253 --> ✅
- [x] Implement Smart Z-Index Enforcement: Re-assert on Focus/Show events & 1s Keep-Alive <!-- id: 254 --> ✅

## Phase 3: Das "Antigravity" Skill-System
- [x] Erstelle `/skills` Verzeichnis im Projekt-Root <!-- id: 300 --> ✅
- [x] Erstelle `/skills/summary.md` mit YAML Frontmatter & Prompt <!-- id: 302 --> ✅
- [x] Füge `walkdir` Crate zu `Cargo.toml` für das Scannen von Dateien hinzu <!-- id: 303 --> ✅
- [x] Füge `serde_yaml` Crate zu `Cargo.toml` für das Parsen hinzu <!-- id: 304 --> ✅
- [x] Erstelle Verzeichnis `src-tauri/src/skills/` <!-- id: 305 --> ✅
- [x] Erstelle Modul `src-tauri/src/skills/mod.rs` <!-- id: 306 --> ✅
- [x] Erstelle Modul `src-tauri/src/skills/loader.rs` <!-- id: 307 --> ✅
- [x] Definiere `Skill` Struktur (Name, Beschreibung, Prompt-Text) <!-- id: 308 --> ✅
- [x] Implementiere rekursiven File Walk in der `load_skills` Funktion <!-- id: 309 --> ✅
- [x] Implementiere Lesen von Dateien und Splitten von Inhalten (YAML vs Body) <!-- id: 310 --> ✅
- [x] Parse YAML Frontmatter in `Skill`-Struktur Metadaten <!-- id: 311 --> ✅
- [x] Lade Skills beim App-Start und gib via println! aus <!-- id: 312 --> ✅
- [x] Verifiziere, dass Skills beim App-Start korrekt geladen werden <!-- id: 313 --> ✅

## Phase 4: Intelligence & Injection (GROQ LLAMA3)
- [x] Erstelle Modul `src-tauri/src/llm/groq.rs` <!-- id: 400 --> ✅ (Groq-Only Strategie)
- [x] Definiere Strukturen für Groq Chat Completion Request/Response <!-- id: 401 --> ✅
- [x] Implementiere `send_request` Funktion für Groq Llama3 API <!-- id: 402 --> ✅ (`chat_completion()`)
- [x] Konstruiere System Prompt dynamisch aus geladenen Skills <!-- id: 403 --> ✅ (`build_system_prompt()`)
- [x] Füge `enigo` Crate zu `Cargo.toml` für Eingabesimulation hinzu <!-- id: 404 --> ✅
- [x] Erstelle Verzeichnis `src-tauri/src/input/` <!-- id: 405 --> ✅
- [x] Erstelle Modul `src-tauri/src/input/injector.rs` <!-- id: 406 --> ✅
- [x] Implementiere `inject_text` Funktion <!-- id: 407 --> ✅ (`type_text()`)
- [x] Füge `window.hide()` Aufruf vor der Text-Injektion hinzu <!-- id: 408 --> ✅
- [x] Füge `std::thread::sleep` (200ms) nach dem Verstecken hinzu <!-- id: 409 --> ✅
- [x] Führe `enigo` Text-Eingabesequenz aus <!-- id: 410 --> ✅
- [x] Integriere: Audio -> Groq Whisper -> Text -> Groq Llama3 -> Ergebnis -> Injection <!-- id: 411 --> ✅
- [x] Refine Default LLM Behavior to "Silent Editor" Mode <!-- id: 412 --> ✅

## Phase 5: UI & Polish
- [x] Refakturiere das Hauptfenster zum Floating Capsule Design (Wispr Flow-Stil) <!-- id: 510 --> ✅
- [x] Installiere `lucide-react`, `clsx`, `tailwind-merge` <!-- id: 500 --> ✅
- [x] Erstelle `useAudioVisualizer.ts` Hook (Web Audio API) <!-- id: 501 --> ✅
- [x] Erstelle `AudioVisualizer.tsx` Komponente (CSS Bars) <!-- id: 502 --> ✅
- [x] Erstelle `StatusIndicator.tsx` (Idle/Recording/Processing/Done) <!-- id: 503 --> ✅ (StatusDot + StatusLabel)
- [x] Integriere Komponenten in das HUD-Layout (`App.tsx`) <!-- id: 504 --> ✅
- [ ] Füge Anzeige für Fehlerbehandlung (Toast oder Status-Text) hinzu <!-- id: 505 -->
