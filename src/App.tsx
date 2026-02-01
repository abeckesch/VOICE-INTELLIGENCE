import { useEffect, useRef, useState } from "react";
import "./index.css";
import { useAudioRecorder, RecordingState } from "./hooks/useAudioRecorder";
import { useAudioVisualizer } from "./hooks/useAudioVisualizer";
import { AudioVisualizer } from "./components/AudioVisualizer";
import { SettingsOverlay } from "./components/SettingsOverlay";
import { Settings, X, Check } from "lucide-react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

function StatusDot({ state }: { state: RecordingState }) {
  const config: Record<RecordingState, { color: string; pulse: boolean }> = {
    idle: { color: "bg-white/40", pulse: false },
    recording: { color: "bg-red-500", pulse: true },
    processing: { color: "bg-amber-500", pulse: true },
  };

  const { color, pulse } = config[state];

  return (
    <div className={`w-3 h-3 rounded-full ${color} ${pulse ? "animate-pulse" : ""}`} />
  );
}

function StatusLabel({ state, isPrivacyMode, isPaused }: { state: RecordingState; isPrivacyMode: boolean; isPaused: boolean }) {
  if (isPaused) {
    return (
      <div className="w-20 text-center">
        <span className="text-amber-400 text-sm font-medium animate-pulse">Pausiert...</span>
      </div>
    );
  }

  const labels: Record<RecordingState, string> = {
    idle: "Bereit",
    recording: "Höre zu...",
    processing: isPrivacyMode ? "Verarbeite..." : "Verarbeite...",
  };

  return (
    <div className="w-20 text-center">
      <span className="text-white/90 text-sm font-medium">{labels[state]}</span>
    </div>
  );
}

function App() {
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isPrivacyMode, setIsPrivacyMode] = useState(false);
  const [whisperPath, setWhisperPath] = useState(() => localStorage.getItem("whisperPath") || "");
  const [modelPath, setModelPath] = useState(() => localStorage.getItem("modelPath") || "");
  const [ffmpegPath, setFfmpegPath] = useState(() => localStorage.getItem("ffmpegPath") || "");
  const [selectedLanguage, setSelectedLanguage] = useState(() => localStorage.getItem("selectedLanguage") || "auto");
  const [activeSkill, setActiveSkill] = useState(() => localStorage.getItem("activeSkill") || "auto");

  // Persist settings
  useEffect(() => {
    localStorage.setItem("whisperPath", whisperPath);
  }, [whisperPath]);

  useEffect(() => {
    localStorage.setItem("modelPath", modelPath);
  }, [modelPath]);

  useEffect(() => {
    localStorage.setItem("ffmpegPath", ffmpegPath);
  }, [ffmpegPath]);

  useEffect(() => {
    localStorage.setItem("selectedLanguage", selectedLanguage);
  }, [selectedLanguage]);

  useEffect(() => {
    localStorage.setItem("activeSkill", activeSkill);
  }, [activeSkill]);

  // Audio Recorder Hook - now aware of privacy mode and language
  const { state, error, stream, startRecording, stopRecording, cancelRecording, isPaused, setIsPaused } = useAudioRecorder(isPrivacyMode, whisperPath, modelPath, ffmpegPath, selectedLanguage, activeSkill);

  // Sync Pause state with Settings Open state
  useEffect(() => {
    setIsPaused(isSettingsOpen);
  }, [isSettingsOpen, setIsPaused]);

  const { bars, startAnalyzing, stopAnalyzing } = useAudioVisualizer();
  const stateRef = useRef(state);

  // Keep stateRef in sync
  useEffect(() => {
    stateRef.current = state;
  }, [state]);

  // Handle Window Resize for Settings
  useEffect(() => {
    const updateWindowSize = async () => {
      try {
        await invoke("set_window_expand", { expand: isSettingsOpen });
      } catch (err) {
        console.error("Failed to resize window:", err);
      }
    };
    updateWindowSize();
  }, [isSettingsOpen]);

  // Start/stop visualizer when stream changes
  useEffect(() => {
    if (stream && state === "recording") {
      startAnalyzing(stream);
    } else {
      stopAnalyzing();
    }
  }, [stream, state, startAnalyzing, stopAnalyzing]);

  // Listen for Tauri events from backend
  useEffect(() => {
    let unlistenShown: (() => void) | undefined;
    let unlistenHiding: (() => void) | undefined;

    const setupListeners = async () => {
      // When window is shown, start recording
      unlistenShown = await listen("window-shown", () => {
        if (stateRef.current === "idle") {
          startRecording();
          // Close settings when window re-opens to be clean
          setIsSettingsOpen(false);
        }
      });

      // When window is about to hide, stop recording
      unlistenHiding = await listen("window-hiding", () => {
        if (stateRef.current === "recording") {
          stopRecording();
        }
      });
    };

    setupListeners();

    return () => {
      unlistenShown?.();
      unlistenHiding?.();
    };
  }, [startRecording, stopRecording]);

  return (
    // justify-end keeps the capsule at the bottom when window expands
    <main className="w-full h-full flex flex-col items-center justify-end relative pb-1">
      {/* Settings Overlay */}
      {isSettingsOpen && (
        <SettingsOverlay
          isOpen={isSettingsOpen}
          onClose={() => setIsSettingsOpen(false)}
          isPrivacyMode={isPrivacyMode}
          onTogglePrivacy={() => setIsPrivacyMode(!isPrivacyMode)}
          whisperPath={whisperPath}
          setWhisperPath={setWhisperPath}
          modelPath={modelPath}
          setModelPath={setModelPath}
          ffmpegPath={ffmpegPath}
          setFfmpegPath={setFfmpegPath}
          selectedLanguage={selectedLanguage}
          setSelectedLanguage={setSelectedLanguage}
          activeSkill={activeSkill}
          setActiveSkill={setActiveSkill}
        />
      )}

      {/* Floating Capsule HUD */}
      <div className={`bg-black/80 backdrop-blur-md text-white rounded-full px-3 py-2 shadow-2xl flex items-center gap-2 transition-colors duration-300 relative z-10 ${isSettingsOpen ? "border border-amber-500/50 shadow-[0_0_15px_rgba(245,158,11,0.2)]" : "border border-white/10"
        }`}>

        {/* Settings Button (Left) */}
        <button
          onClick={() => setIsSettingsOpen(!isSettingsOpen)}
          className={`p-1.5 rounded-full transition-colors ${isSettingsOpen ? "bg-white/10 text-white" : "text-white/40 hover:text-white"}`}
        >
          <Settings size={16} />
        </button>

        {/* Separator */}
        <div className="w-px h-4 bg-white/10 mx-1"></div>

        <StatusDot state={state} />

        {/* Audio Visualizer - handles its own visibility based on state/paused */}
        <AudioVisualizer bars={bars} state={state} isPrivacyMode={isPrivacyMode} isPaused={isPaused} />

        <StatusLabel state={state} isPrivacyMode={isPrivacyMode} isPaused={isPaused} />

        {error && (
          <span className="text-red-400 text-xs">!</span>
        )}

        {/* Separator */}
        <div className="w-px h-4 bg-white/10 mx-1"></div>

        {/* Action Buttons (Right) */}
        <div className="flex items-center gap-1">
          {/* Confirm/Send Button */}
          <button
            onClick={() => stopRecording()}
            disabled={state !== "recording"}
            className={`p-1.5 rounded-full transition-colors ${state === "recording" ? "text-white/40 hover:text-green-400" : "text-white/10 cursor-not-allowed"}`}
            title="Finish & Send"
          >
            <Check size={16} />
          </button>

          {/* Close/Cancel Button */}
          <button
            onClick={() => {
              if (state === "recording") {
                cancelRecording();
              }
              invoke("hide_window");
            }}
            className="p-1.5 rounded-full text-white/40 hover:text-red-400 transition-colors"
            title="Cancel Recording"
          >
            <X size={16} />
          </button>
        </div>
      </div>
    </main>
  );
}

export default App;
