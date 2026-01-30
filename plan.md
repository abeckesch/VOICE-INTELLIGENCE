# Implementierungsplan: Voice Intelligence Desktop App

> **Ziel:** Erstellung eines hochperformanten Voice HUDs mit Tauri v2, Next.js und **Groq-Only Strategy** (Whisper v3 Turbo + Llama3).

---

## 1. Projektstruktur & Architektur

Die Anwendung folgt einer strikten Trennung zwischen Rust-Backend (System-Level Ops, API-Orchestrierung) und Next.js-Frontend (UI/UX, Audio-Capture).

### File Tree

```
/
├── src-tauri/                  # Rust Backend
│   ├── src/
│   │   ├── main.rs             # Entry Point, Setup
│   │   ├── lib.rs              # Tauri Commands exports
│   │   ├── audio/              # Audio processing (optional helper)
│   │   ├── skills/             # Skill Loader & Parser logic
│   │   │   ├── mod.rs
│   │   │   └── loader.rs
│   │   ├── llm/                # API Clients
│   │   │   ├── mod.rs
│   │   │   ├── groq.rs         # Groq API Client (Whisper + Llama3)
│   │   │   └── whisper.rs      # Whisper v3 Turbo via Groq
│   │   └── input/              # Keyboard Injection
│   │       ├── mod.rs
│   │       └── injector.rs
│   ├── Cargo.toml
│   ├── capabilities/           # Tauri ACLs
│   └── tauri.conf.json         # Config
│
├── src/                        # Next.js Frontend (Source)
│   ├── app/
│   │   ├── page.tsx            # HUD UI
│   │   ├── layout.tsx
│   │   └── globals.css
│   ├── components/
│   │   ├── audio-visualizer.tsx
│   │   └── status-indicator.tsx
│   ├── hooks/
│   │   └── use-audio-recorder.ts
│   └── lib/
│       └── tauri-bridge.ts     # Type-safe invokes
│
├── skills/                     # User-defined Skills (Antigravity Pattern)
│   ├── email/
│   │   └── SKILL.md
│   └── coding/
│       └── SKILL.md
│
├── public/
├── package.json
├── next.config.js
└── plan.md                     # This file
```

---

## 2. Dependencies

### Rust Crates (`src-tauri/Cargo.toml`)
*   **Core:** `tauri = "2.0"`, `tokio = { version = "1", features = ["full"] }`
*   **Serialization:** `serde`, `serde_json`, `serde_yaml` (für Skill-Frontmatter)
*   **HTTP Client:** `reqwest = { version = "0.11", features = ["json", "multipart", "stream"] }`
*   **System Input:** `enigo` (oder `rdev`) für Keyboard-Injection.
    *   *Note:* `enigo` ist oft einfacher für Cross-Platform-Typing.
*   **Global Hotkey:** `tauri-plugin-global-shortcut`
*   **Clipboard:** `tauri-plugin-clipboard-manager` (für Backup)
*   **Utilities:** `walkdir` (Filesystem Scanning), `anyhow` (Error Handling), `base64` (Audio encoding internal)

### NPM Packages (`package.json`)
*   **Framework:** `next`, `react`, `react-dom`
*   **Tauri Bridge:** `@tauri-apps/api`, `@tauri-apps/plugin-global-shortcut`
*   **UI:** `lucide-react` (Icons), `clsx`, `tailwind-merge` (Styling utils)
*   **Styling:** `tailwindcss`, `postcss`, `autoprefixer`
*   **State/Logic:** `zustand` (optional, for simple store)

---

## 3. Datenfluss & Pipeline

1.  **Trigger:** `Alt+Space` (Rust) -> Öffnet transparentes Fenster (Frontend).
2.  **Audio:** Frontend nutzt Web Audio API -> `MediaRecorder` -> Blobs.
3.  **Handover:** Frontend sendet Audio-Blob via Command `process_voice_command` an Rust.
4.  **STT:** Rust sendet Audio an **Groq Whisper v3 Turbo**.
    *   *Constraint:* < 500ms.
5.  **Skill-Matching:**
    *   System lädt alle `SKILL.md` Beschreibungen beim Start.
    *   Prompt enthält Liste aller Skills + Beschreibungen.
6.  **Intelligence:**
    *   Rust sendet Transkript + Skill-Kontext an **Groq Llama3** (70B oder 8B).
    *   Prompt-Instruction: "Choose skill, execute immediately per instructions."
7.  **Execution & Output:
    *   LLM Response = Text Payload.
    *   Fenster schließt sich.
    *   Rust nutzt `enigo`, um Text an Cursor-Position zu tippen.
    *   Text zusätzlich ins Clipboard kopieren.

---

## 4. Implementierungsphasen

### Phase 1: Setup & Skeleton (Tag 1)
*   Initialisiere Tauri v2 Projekt mit Next.js Template.
*   Konfiguriere TailwindCSS für "Premium Visuals" (Dark Mode, Glassmorphism).
*   Richte globalen Hotkey (`Alt+Space`) in Rust ein, der das Fenster toggled.
*   Implementiere Window-Transparenz, "Always On Top" und **Bottom-Center Positionierung** für optimales HUD-Feeling.

### Phase 2: Core Loop (Audio & API) (Tag 1-2)
*   **Frontend Audio:** Hook `useAudioRecorder` implementieren (Record -> Blob).
*   **Backend Command:** `process_audio(payload: Vec<u8>)`.
*   **Whisper Integration:** `whisper.rs` Modul für Upload zu API Provider.
    *   *Test:* Sende Audio, logge Text-Output.
*   **Validation:** Latenztest Audio -> Text (Ziel: < 1s).

### Phase 3: The "Antigravity" Skill System (Tag 2)
*   Erstelle `/skills` Ordnerstruktur und Beispiel-Skill (`email/SKILL.md`).
*   **Indexer:** Rust-Modul, das beim Start `/skills` rekursiv scannt.
*   **Parser:** Lese YAML-Frontmatter (Description) und Markdown-Body.
*   **Context Injection:** Baue dynamischen System-Prompt aus geladenen Skills.

### Phase 4: Intelligence & Injection (Tag 3)
*   **Groq Llama3 Integration:** `groq.rs` Client für LLM.
    *   *Flow:* Transkript -> Llama3 -> Response.
*   **Keyboard Injection:** Nutze `enigo` crate.
    *   *Wichtig:* Fenster muss vor dem Tippen versteckt/unfokussiert werden, damit der Text in die vorherige Anwendung fließt.
*   **Clipboard:** Copy-to-Clipboard als Backup.

### Phase 5: Polish & UI (Tag 3+)
*   **Visual Feedback:** Audio-Visualizer im HUD während des Sprechens.
*   **Streaming:** Falls möglich, zeige Zwischenergebnisse im HUD (optional für v1).
*   **Error Handling:** Visuelles Feedback bei API-Fehlern oder No-Match.

---

## 5. Konfigurations-Variablen (`.env`)
*   `GROQ_API_KEY` (für Whisper Turbo + Llama3 - Single Provider Strategy)
*   `HOTKEY_BINDING` (Default: "Alt+Space")

---

## 6. Hybrid Language Heuristic

Um Whisper-Halluzinationen bei kurzen Aufnahmen zu verhindern, aber Mehrsprachigkeit bei längeren Diktaten zu erlauben:

- **Threshold:** 4000ms
- **Logik:**
  - Wenn `duration_ms < 4000`: Nutze `PREFERRED_LANGUAGE` (Default: `de`)
  - Sonst: Nutze Whisper Auto-Detect
- **Konfiguration:** `PREFERRED_LANGUAGE` in `.env` setzen
