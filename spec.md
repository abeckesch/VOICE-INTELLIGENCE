# Spezifikation: Voice Intelligence Desktop App (Final Build)

## 1. High-Level Ziel
Eine blitzschnelle Desktop-Anwendung ("Voice HUD"), die gesprochene Sprache in perfekt formatierten Text verwandelt. 
Fokus: Maximale Geschwindigkeit (Cloud) oder maximale Privatsphäre (Local).

## 2. Core User Loop
1.  **Trigger:** Globaler Hotkey (z. B. `Alt+Space`).
2.  **Input:** Minimalistisches Overlay (HUD). Nutzer spricht.
3.  **Processing (Dual-Engine Strategie):**
    *   **Input Gate:** Audio -> Micro -> VAD (Silence Guard).
    *   **Branch A (Cloud):** Audio -> Groq Whisper -> Groq Llama3 -> UI.
    *   **Branch B (Local):** Audio -> Whisper.cpp -> Ollama (Llama3) -> UI.
    *   **Branch C (Raw):** Audio -> Whisper -> UI (Bypass LLM).
4.  **Output:** Text wird via Keyboard-Simulation an der Cursor-Position eingefügt + Clipboard-Backup.

## 3. Technische Anforderungen & Stack

### A. Frontend / Runtime
*   **Stack:** Next.js (React) + Tauri v2.
*   **Begründung:** Native Performance, kleiner Footprint (~3MB Installer), globale Hotkeys.

### B. Architecture Modes

#### 1. Cloud Mode (Default)
Nutzt die extrem schnelle Infrastruktur von Groq.
*   **STT:** `Whisper v3 Turbo` (Groq API).
*   **LLM:** `Llama3-70B` (Groq API).
*   **Vorteil:** Latenz < 1s, State-of-the-Art Qualität.

#### 2. Local Mode (Privacy / BYOE)
"Bring Your Own Engine" - für User mit hohen Privacy-Anforderungen.
*   **STT:** `whisper-CLI` (lokale Binary, z.B. whisper.cpp).
*   **LLM:** `Ollama` (lokaler Server, z.B. llama3).
*   **Konfiguration:** User muss Pfade zu Binary und Modell in den Settings hinterlegen.
*   **Vorteil:** 100% Offline, keine Daten verlassen das Gerät.

#### 3. Raw Mode (Bypass)
Reines Diktat ohne intelligente Nachbearbeitung.
*   **Flow:** Audio -> STT -> Text Output.
*   **Use Case:** Schnellstmögliche Transkription, Rohdaten.

### C. Skill-System
Skills sind spezialisierte Verarbeitungs-Pipelines.
1.  **Standard (Silent Editor):** Korrektur von Grammatik/Rechtschreibung. Kein Chat-Verhalten.
2.  **Zusammenfassung:** Erstellt Bullet Points aus dem Gesagten.
3.  **Business Polish:** Formuliert Umgangssprache in professionelles Business-Deutsch um.
4.  **Action Items:** Extrahiert To-Dos in eine Markdown-Liste.

### D. Quality Gates
*   **Silence Guard:** VAD (Voice Activity Detection) basierend auf RMS-Amplitude. Verwirft stille Aufnahmen (< 150ms RMS) sofort, um Kosten und Halluzinationen zu vermeiden.
*   **Hallucination Filter:** Filtert bekannte Whisper-Artefakte (z.B. "Thank you", "Subtitles by...") aus kurzen Aufnahmen.

## 4. User Experience (UX)

### UI Design: Floating Capsule HUD
*   **Visuals:** Glassmorphism, Rounded-Full, Bottom-Center.
*   **States:**
    1.  **Idle:** "Bereit" (Grauer Dot)
    2.  **Recording:** "Höre zu..." (Roter Pulse + Visualizer)
    3.  **Processing:** "Verarbeite..." (Amber Pulse)
    4.  **Success:** "Gesendet" (Grüner Check)
    5.  **Paused:** Settings offen (Amber Border, Flatline)

### Privacy Indikator
*   **Cloud Mode:** Cyan/Blaues Theme.
*   **Local Mode:** Violettes/Lila Theme (Incognito).

## 5. Projektstruktur
```
voice-intelligence/
├── src-tauri/              # Rust Backend (Core Logic)
│   ├── src/llm/            # API Clients (Groq, Ollama) & Prompts
│   └── src/input/          # Keyboard Injection
├── src/                    # React Frontend (UI)
│   ├── components/         # HUD, Visualizer, Settings
│   └── hooks/              # Audio Recorder, State Logic
└── .env                    # API Keys
```
