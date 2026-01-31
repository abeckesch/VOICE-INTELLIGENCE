# Implementierungsplan: Voice Intelligence Desktop App (Final Architecture)

> **Ziel:** Erstellung eines hochperformanten Voice HUDs mit Tauri v2, Next.js und **Dual-Engine Strategy** (Cloud speed vs. Local privacy).

---

## 1. Projektstruktur & Architektur

Die Anwendung folgt einer strikten Trennung zwischen Rust-Backend (System-Level Ops, API-Orchestrierung) und Next.js-Frontend (UI/UX, Audio-Capture).

### File Tree (Final Build)

```
/
├── src-tauri/                  # Rust Backend
│   ├── src/
│   │   ├── main.rs             # Entry Point, Setup
│   │   ├── lib.rs              # Tauri Commands & Core Logic
│   │   ├── llm/                # Intelligence Layer
│   │   │   ├── mod.rs
│   │   │   ├── groq.rs         # Cloud Logic (Groq API)
│   │   │   └── prompt.rs       # System Prompts & Skill Defs
│   │   ├── input/              # Keyboard Injection
│   │   │   ├── mod.rs
│   │   │   └── injector.rs     # Enigo wrapper
│   │   └── capabilities/       # Tauri Permissions
│   ├── Cargo.toml
│   └── tauri.conf.json         # Config (Transparent, AlwaysOnTop)
│
├── src/                        # Next.js Frontend (Source)
│   ├── app/                    # Routing (Single Page)
│   ├── components/
│   │   ├── AudioVisualizer.tsx # Canvas/CSS Audio Bars
│   │   └── SettingsOverlay.tsx # Local/Cloud Config UI
│   ├── hooks/
│   │   └── useAudioRecorder.ts # MediaRecorder Logic
│   └── App.tsx                 # Main HUD Controller
│
├── README.md                   # Documentation
└── spec.md                     # Requirements
```

---

## 2. Dependencies & Stack

### Rust Backend (`src-tauri`)
*   **Core:** `tauri = "2.0"`, `tokio` (Async runtime)
*   **HTTP:** `reqwest` (API Calls for Groq & Ollama)
*   **Input:** `enigo` (Keyboard Simulation)
*   **Audio:** `ffmpeg` (via Command Line) & `whisper-cli` (External Binary)
*   **OS Integration:** `tauri-plugin-global-shortcut`, `tauri-plugin-clipboard-manager`

### Frontend (`src`)
*   **Framework:** `vite`, `react`, `typescript`
*   **UI:** `tailwind-merge`, `lucide-react` (Icons)
*   **Styling:** `tailwindcss`

---

## 3. Datenfluss & Pipelines

### A. Cloud Pipeline (Default)
1.  **Audio:** Frontend -> `process_audio` (Rust)
2.  **STT:** Rust POST -> **Groq Whisper v3 Turbo** API
3.  **LLM:** Rust POST -> **Groq Llama3-70B** API
4.  **Output:** Keyboard Injection + Clipboard

### B. Local Pipeline (Privacy / BYOE)
1.  **Audio:** Frontend -> Rust
2.  **STT:** Rust Exec -> `whisper-cli.exe` (Local Binary + .bin Model)
3.  **LLM:** Rust POST -> `localhost:11434` (Ollama Llama3)
4.  **Output:** Keyboard Injection + Clipboard

### C. Skill Selection
*   **Mechanik:** Hartcodierte Prompts in `llm/prompt.rs`.
*   **Trigger:** Frontend sendet `skill_id` (z.B. "summary", "email").
*   **Logic:** Backend wählt passenden System-Prompt vor dem LLM-Call.

---

## 4. Implementierte Features

### Core
*   [x] **Global Hotkey:** `Alt+Space` für HUD Toggle.
*   [x] **Always-On-Top:** Aggressives Z-Index Management für Overlay.
*   [x] **Silence Guard:** RMS-basierte Stille-Erkennung (VAD Lite).

### UI/UX
*   [x] **Floating Capsule:** Glassmorphism Design.
*   [x] **States:** Visualisierung von Idle/Recording/Processing.
*   [x] **Settings:** Konfiguration von Pfaden (Local Mode) und Sprache.

### Local "Sidecar" Support
*   [x] **Whisper CLI:** Unterstützung für lokale `main.exe` von whisper.cpp.
*   [x] **Ollama:** Direkte Integration via HTTP.

---

## 5. Security & Privacy
*   **API Key:** `GROQ_API_KEY` via Environment Variable.
*   **Local Data:** Im "Local Mode" verlassen Audiodaten niemals den RAM/Disk des Users.
