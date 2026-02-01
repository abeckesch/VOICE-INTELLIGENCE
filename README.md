# Voice Intelligence Capsule

> **Local-First Voice Dictation & AI Processing**

![Tauri v2](https://img.shields.io/badge/Tauri_v2-FCC019?style=for-the-badge&logo=tauri&logoColor=black)
![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Local AI](https://img.shields.io/badge/AI-Local-blueviolet?style=for-the-badge)
![Privacy Focused](https://img.shields.io/badge/Data-Private-success?style=for-the-badge)

**Problem:** Traditional voice dictation tools send your sensitive audio data to the cloud, creating privacy risks and dependencies on internet connectivity.
**Solution:** The Voice Intelligence Capsule implements a **Dual-Engine Architecture**. Choose between **Groq Cloud** for lightning-fast latency (<1s) or **Local Mode** for absolute data sovereignty (Air-Gapped).

---

## 🏗️ Architecture Decision Record (ADR): The Dual Engine

We built this application on a core philosophy: **User Control**. We do not bundle AI models; we provide the engine to run them. This "Bring Your Own Engine" (BYOE) approach ensures you are never locked into a specific provider or model version.

### A. ⚡ Cloud Mode (Speed)
Leverages the **Groq LPU™ Inference Engine**.
*   **STT:** Whisper v3 Turbo.
*   **LLM:** Llama3-70B.
*   **Result:** Usable text in under 1 second. Ideal for general emails, coding, and rapid tasks where data sensitivity is low.

### B. 🛡️ Local Mode (Privacy)
Runs 100% on your device. **No data leaves your machine.**
*   **STT:** Local `whisper-cli` (C++ implementation via whisper.cpp).
*   **LLM:** Local `Ollama` instance.
*   **Result:** Complete offline capability. Ideal for NDA-protected work, medical/legal dictation, or unstable connections.

---

## ✨ Feature Highlights

*   **Instant Access (Floating HUD):** Press `Alt+Space` to summon the capsule. Smart Z-Index ensures it stays **Always-On-Top**, even over full-screen apps.
*   **True Privacy (Local Mode):** Switch to offline mode with one click (after initial setup). Runs Whisper & Llama3 entirely on your device for maximum data sovereignty.
*   **Smart Silence Guard (VAD):** Innovative RMS-based gatekeeper. Detects silence (< 150ms) and aborts processing instantly. Prevents "Thank you" hallucinations and saves API costs.
*   **Skill-Based Processing:** Dictate with intent. Choose between **Standard** (Polishing), **Email** (Drafting), **To-Do** (Action Items), or **Summary** (Bullet points).
*   **Language Anchoring:** Enforce a specific language (e.g., German) to prevent Whisper from switching to English on short commands.

---

## � Roadmap: Vision 2026

We are building the future of desktop interaction. Here is what's next:

### 🤖 Auto-Skill Router (SLM-based)
Instead of manually selecting "Email" or "To-Do", a lightweight **Small Language Model (SLM)** will analyze the intent of your dictation in real-time.
*   *Use Case:* User says "Schedule a meeting with Peter at 2 PM" -> Router detects "Calendar Intent" -> Activates "Action Item Skill" automatically.

### ⚙️ Custom Skill Generator (Prompt-as-Code)
A "No-Code" interface allowing users to define persistent personas.
*   *Technik:* Define System Prompts (e.g., "Answer like a Pirate", "Format as Jira JSON") that are stored as custom skills.

### 🧠 Local RAG (Context Awareness)
Connect your local LLM to your personal knowledge base (e.g., Obsidian Vault, PDF folder).
*   *Goal:* The AI understands *your* context (project names, acronyms) without ever uploading your documents to a cloud vector store.

---

## 🛠️ Setup Guide (BYOE)

This application requires external engines for the **Local Mode**. Follow these steps to enable full offline capabilities:

### 1. Prerequisites
*   **FFmpeg:** Required for audio processing (WebM -> WAV conversion).
    *   [Download FFmpeg](https://ffmpeg.org/download.html) and add it to your **System PATH**.
    *   *Note:* If you add FFmpeg to your System PATH, you can leave the "FFmpeg Path" in settings **empty**. Only define it if you use a portable version not globally installed.

### 2. Whisper Engine (STT)
1.  Download a `whisper-cli` binary (e.g., from [whisper.cpp releases](https://github.com/ggerganov/whisper.cpp/releases)).
    *   *Windows:* `main.exe` or `whisper-cli.exe`.
2.  Download a Model file (`.bin`) (e.g., `ggml-base.bin` or `ggml-medium.bin` from HuggingFace).
3.  **In App Settings:** Enter the absolute paths to both the binary and the model file.

### 3. Ollama (LLM)
1.  Download and install [Ollama](https://ollama.com).
2.  Run `ollama serve` in a terminal.
3.  Pull the model: `ollama pull llama3`.

### 4. Configuration
Open the Capsule Settings (`Alt+Space` -> Gear Icon):
*   Toggle **Local Mode (Offline)** to ON.
*   Verify the paths are green/accepted.
*   Start speaking!

---

## 💻 Development & Build

### Install Dependencies
```bash
npm install
```

### Run in Development Mode
Starts the React frontend and Rust backend with hot-reloading.
```bash
npm run tauri dev
```

### Build for Production
Creates an optimized executable (`.exe`) in `src-tauri/target/release/bundle/nsis/`.
```bash
npm run tauri build
```

---



## 🧩 Tech Stack (Under the Hood)

| Component | Technologies |
| :--- | :--- |
| **Frontend** | React 19, TypeScript, TailwindCSS v4, Lucide Icons |
| **Backend** | Rust, Tauri v2 (Capabilities, Windowing, Tray) |
| **Audio** | FFmpeg (Processing), cpal/MediaRecorder (Capture) |
| **AI (Cloud)** | Groq API (Whisper v3 Turbo + Llama 3) |
| **AI (Local)** | Whisper.cpp (C++ Bindings), Ollama (Local REST API) |

---

**Developed with ❤️ in Rust & TypeScript.**
