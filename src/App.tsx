import { useEffect, useRef } from "react";
import "./index.css";
import { useAudioRecorder, RecordingState } from "./hooks/useAudioRecorder";
import { useAudioVisualizer } from "./hooks/useAudioVisualizer";
import { AudioVisualizer } from "./components/AudioVisualizer";
import { listen } from "@tauri-apps/api/event";

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

function StatusLabel({ state }: { state: RecordingState }) {
  const labels: Record<RecordingState, string> = {
    idle: "Bereit",
    recording: "Höre zu...",
    processing: "Verarbeite...",
  };

  return (
    <span className="text-white/90 text-sm font-medium">{labels[state]}</span>
  );
}

function App() {
  const { state, error, stream, startRecording, stopRecording } = useAudioRecorder();
  const { bars, startAnalyzing, stopAnalyzing } = useAudioVisualizer();
  const stateRef = useRef(state);

  // Keep stateRef in sync
  useEffect(() => {
    stateRef.current = state;
  }, [state]);

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
    <main className="w-full h-full flex items-center justify-center">
      {/* Floating Capsule HUD */}
      <div className="bg-black/80 backdrop-blur-md text-white rounded-full px-6 py-3 shadow-2xl flex items-center gap-4 border border-white/10">
        <StatusDot state={state} />

        {/* Audio Visualizer - only visible during recording */}
        <AudioVisualizer bars={bars} state={state} />

        <StatusLabel state={state} />

        {error && (
          <span className="text-red-400 text-xs">!</span>
        )}
      </div>
    </main>
  );
}

export default App;
