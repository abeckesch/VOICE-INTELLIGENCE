# Spezifikation: Voice Intelligence Desktop App (Performance & Value Edition)

## 1. High-Level Ziel
Eine blitzschnelle Desktop-Anwendung ("Voice HUD"), die gesprochene Sprache in perfekt formatierten Text verwandelt. Fokus: Maximale Ergebnisqualität bei minimaler Latenz und geringsten Betriebskosten.

## 2. Core User Loop
1.  **Trigger:** Globaler Hotkey (z. B. `Alt+Space`).
2.  **Input:** Minimalistisches Overlay (HUD). Nutzer spricht.
3.  **Processing (High-Speed Cloud Pipeline):**
    *   **Audio:** Upload zu Whisper v3 Turbo (via Groq).
    *   **Intelligence:** Transkript wird an **Groq Llama3** gesendet.
    *   **Routing & Generation:** Ein einziger LLM-Call entscheidet über den "Skill" (z. B. E-Mail, Code, Notiz) und generiert den Output.
4.  **Output:** Text wird via Keyboard-Simulation an der Cursor-Position eingefügt + Clipboard-Backup.

## 3. Technische Anforderungen & Stack

### A. Frontend / Runtime
*   **Stack:** Next.js (React) + Tauri v2.
*   **Begründung:** Tauri bietet native Performance und globalen Hotkey-Support bei minimalem Speicherbedarf.

### B. AI Pipeline (Groq-Only Strategy)
*   **STT (Speech-to-Text):** `Whisper v3 Turbo` via Groq. Ziel für Latenz: < 500ms.
*   **LLM (Inferenz):** `Llama3` via Groq.
    *   *Warum:* Bietet extreme Geschwindigkeit bei hoher Qualität.
    *   *Modell:* Vorzugsweise Llama3-70B für hohe Qualität oder Llama3-8B für maximale Geschwindigkeit.
*   **Abgrenzung:** Wir nutzen Groq als Single-Provider für die gesamte Pipeline, um die Latenz durch minimierten Netzwerk-Overhead zu optimieren.

### C. Skill-System ("Antigravity" Pattern)
*   **Struktur:** Skills werden als Markdown-Dateien in `/skills` definiert (z.B. `/skills/email/SKILL.md`).
*   **Router-Logik:** Der System-Prompt injiziert die Beschreibungen aller Skills. Das LLM wählt den Skill und führt ihn in einem Durchgang aus (Chain-of-Thought), um Latenz zu sparen.
*   **Fallback-Verhalten ("Silent Editor"):** Wenn kein Skill erkannt wird, fungiert das System als reiner Editor:
    *   Korrektur von Grammatik, Interpunktion und Groß-/Kleinschreibung.
    *   Entfernung von Füllwörtern (ähm, äh, uhm).
    *   Keine Meta-Kommentare oder Chatbot-Antworten.

## 4. User Experience (UX) Goals
*   **Harte Anforderung:** Gesamtlatenz (Sprechende -> Textstart) **< 2 Sekunden**.

### UI Design: Floating Capsule HUD
*   **Grundlayout:**
    *   Always-on-top Overlay, bottom-center positioniert.
    *   Pillenschiff/Capsule Design (rounded-full).
    *   Minimale Größe (ca. 400x60px), blockiert nicht den Arbeitsbereich.
    *   Backdrop-blur Glassmorphism Ästhetik (bg-black/80, backdrop-blur-md).
    *   Dezente Border (border-white/10).

### UI-Zustände (State Machine):
1.  **Idle:** Kleiner, pulsierender weißer Punkt. Text: "Bereit".
2.  **Recording:** 
    *   Roter pulsierender Punkt.
    *   Audio-Visualizer mit 5-7 dynamischen Balken, die auf Lautstärke reagieren.
    *   Text: "Höre zu..."
3.  **Processing:** 
    *   Gelber/Amber pulsierender Punkt.
    *   Optionaler Spinner oder Animation.
    *   Text: "Verarbeite..."
4.  **Success/Error:** 
    *   Kurzes Feedback (✓ grün / ✗ rot) für 500ms vor dem Schließen.

*   **Visuals:** Das Overlay muss "snappy" reagieren. Keine Ladespinner, sondern progressive Zustandsanzeige.

## 5. Abgrenzung
*   Keine lokale Inferenz (zu langsam auf Standard-Hardware).
*   Kein komplexes User-Management (API-Keys in `.env`).

## 6. App-Architektur: Tray-based Application with Overlay HUD

*   **Konzept:** Die App verhält sich wie ein System-Widget/HUD, nicht wie ein traditionelles Fenster.
*   **System Tray:** Ein Tray-Icon ermöglicht schnellen Zugriff (Show/Hide, Quit).
*   **Taskbar-Verhalten:** Das Overlay erscheint nicht in der Windows-Taskbar (`skipTaskbar: true`).
*   **Fokus-Management:** Das Fenster klaut beim Start keinen Fokus und "lauert" im Hintergrund bis der Hotkey (Alt+Space) aktiviert wird.
*   **Windows-Integration:** Durch `skipTaskbar` wird das Fenster als Widget behandelt und bleibt bei "Show Desktop" (Win+D) sichtbar.

