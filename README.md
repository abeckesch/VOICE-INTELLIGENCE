# 🎙️ Voice Intelligence HUD (Antigravity)

> Eine Desktop-App, die Spracheingaben lokal aufnimmt, via Groq (Whisper) transkribiert und durch ein modulares Skill-System (Llama 3) verarbeitet – für nahtlose Integration in deinen Workflow.

---

## 🎯 Das Problem

Spracheingabe am Desktop ist umständlich: Entweder diktiert man in eine App, kopiert manuell, oder nutzt klobige Assistenten, die den Workflow unterbrechen. **Voice Intelligence** löst das durch ein **immer bereites HUD**, das per Hotkey erscheint und den verarbeiteten Text direkt an der Cursor-Position einfügt.

---

## 🏗️ Architektur

```
┌──────────────────────────────────────────────────────────────┐
│  FRONTEND (React + Vite)                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐  │
│  │ Audio Capture│  │ Visualizer   │  │ Status Indicator   │  │
│  │ (Web Audio)  │  │ (CSS + TS)   │  │ (idle/rec/proc)    │  │
│  └──────────────┘  └──────────────┘  └────────────────────┘  │
└───────────────────────────┬──────────────────────────────────┘
                            │ Tauri IPC
┌───────────────────────────▼──────────────────────────────────┐
│  BACKEND (Rust + Tauri v2)                                   │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐  │
│  │ Global Hotkey│  │ Groq APIs    │  │ Keyboard Injection │  │
│  │ (Alt+Space)  │  │ Whisper+LLM  │  │ (enigo crate)      │  │
│  └──────────────┘  └──────────────┘  └────────────────────┘  │
│                           │                                   │
│              ┌────────────▼────────────┐                      │
│              │  🧠 Antigravity Skills  │                      │
│              │  (Markdown-basiert)     │                      │
│              └─────────────────────────┘                      │
└──────────────────────────────────────────────────────────────┘
```

| Layer | Technologie | Aufgabe |
|-------|-------------|---------|
| **Frontend** | React, Vite, Tailwind CSS | UI, Audio-Visualizer, Status-Anzeige |
| **Backend** | Rust, Tauri v2 | Hotkeys, API-Aufrufe, System-Integration |
| **AI Pipeline** | Groq Whisper v3 + Llama 3.3 70B | Transkription + Intelligente Verarbeitung |
| **Skills** | Markdown-Dateien (`/skills`) | Erweiterbare Logik-Module |

---

## 💡 Design-Entscheidungen

### Tauri statt Electron
- **Binary-Größe:** ~2.8 MB (NSIS) vs. 150+ MB bei Electron
- **Performance:** Native Rust-Backend, kein Chromium-Overhead
- **Sicherheit:** Capabilities-System für granulare Berechtigungen

### Tray-Only Architektur
- Kein permanentes Fenster – das HUD erscheint nur bei Bedarf
- Minimaler Fokus-Verlust: Alt+Space → Sprechen → Text erscheint

### Silent Editor Mode
- **Standard:** Die KI korrigiert nur (Grammatik, Formatierung) – kein Chatbot
- **Skills:** Explizite Trigger wie *"Fasse zusammen"* aktivieren Spezialfunktionen
- **Vorteil:** Vorhersagbares Verhalten, keine unerwarteten Antworten

### Hybrid Language Heuristic
- **Kurze Aufnahmen (<4s):** Nutzt Deutsch (verhindert Whisper-Halluzinationen)
- **Längere Aufnahmen:** Auto-Detect (ermöglicht Mehrsprachigkeit)

---

## 🛠️ Setup & Installation

### Prerequisites
- **Node.js** v22.12+
- **Rust** (via [rustup](https://rustup.rs))
- **C++ Build Tools** (Windows: Visual Studio Build Tools)

### 1. Dependencies installieren
```bash
cd voice-intelligence
npm install
```

### 2. Environment konfigurieren
```bash
# .env erstellen (im src-tauri Ordner)
cd src-tauri
echo "GROQ_API_KEY=gsk_your_key_here" > .env
echo "PREFERRED_LANGUAGE=de" >> .env
cd ..
```

### 3. Development Server starten
```bash
npm run tauri dev
```

---

## 📦 Production Build

Erstellt eine **eigenständige Desktop-App** ohne externe Abhängigkeiten:

```bash
npm run tauri build
```

Die Installer findest du unter:
- **Windows EXE:** `src-tauri/target/release/bundle/nsis/voice-intelligence_*_x64-setup.exe`
- **Windows MSI:** `src-tauri/target/release/bundle/msi/voice-intelligence_*_x64_en-US.msi`

> 💡 **Hinweis:** Die `.env`-Datei muss im selben Verzeichnis wie die EXE liegen, oder `GROQ_API_KEY` als System-Umgebungsvariable gesetzt sein.

---

## 📖 Bedienungsanleitung

| Schritt | Aktion | Ergebnis |
|---------|--------|----------|
| 1️⃣ | App starten | Läuft unsichtbar im System-Tray |
| 2️⃣ | `Alt + Space` drücken | HUD erscheint (transparentes Overlay) |
| 3️⃣ | Sprechen | Audio-Visualizer zeigt Aufnahme |
| 4️⃣ | Pause machen | Verarbeitung startet automatisch |
| 5️⃣ | Warten | Text wird an Cursor-Position getippt |

### 🎯 Skills nutzen

| Trigger-Phrase | Aktion |
|----------------|--------|
| *"Fasse zusammen: [Text]"* | Erstellt Bullet-Point-Zusammenfassung |
| *(ohne Trigger)* | Silent Editor: Korrigiert nur Grammatik/Format |

### Skills erweitern
Erstelle neue Skills als Markdown-Dateien in `/skills/`:

```yaml
---
name: "Mein Skill"
description: "Was der Skill macht"
trigger_keywords: ["aktiviere", "mach"]
---
Hier steht die Anweisung für die KI...
```

---

## 📁 Projektstruktur

```
voice-intelligence/
├── src/                    # React Frontend
│   ├── components/         # UI-Komponenten (Visualizer, Status)
│   └── hooks/              # Audio Recording Hook
├── src-tauri/              # Rust Backend
│   ├── src/
│   │   ├── llm/            # Groq API Clients (Whisper + Llama)
│   │   ├── skills/         # Skill Loader
│   │   └── input/          # Keyboard Injection (enigo)
│   └── tauri.conf.json     # App-Konfiguration
├── skills/                 # Benutzerdefinierte Skills
│   └── summary.md          # Zusammenfassungs-Skill
├── plan.md                 # Implementierungsplan
└── spec.md                 # UI-Spezifikation
```

---

## 📜 Lizenz

MIT © 2026
