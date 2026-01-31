import { X, CheckCircle2, AlertCircle, Loader2 } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";

interface SettingsOverlayProps {
    isOpen: boolean;
    onClose: () => void;
    isPrivacyMode: boolean;
    onTogglePrivacy: () => void;
    whisperPath: string;
    setWhisperPath: (path: string) => void;
    modelPath: string;
    setModelPath: (path: string) => void;
    ffmpegPath: string;
    setFfmpegPath: (path: string) => void;
    selectedLanguage: string;
    setSelectedLanguage: (lang: string) => void;
    activeSkill: string;
    setActiveSkill: (skill: string) => void;
}

export function SettingsOverlay({
    isOpen,
    onClose,
    isPrivacyMode,
    onTogglePrivacy,
    whisperPath,
    setWhisperPath,
    modelPath,
    setModelPath,
    ffmpegPath,
    setFfmpegPath,
    selectedLanguage,
    setSelectedLanguage,
    activeSkill,
    setActiveSkill,
}: SettingsOverlayProps) {
    const [testStatus, setTestStatus] = useState<'idle' | 'testing' | 'success' | 'error'>('idle');
    const [testMessage, setTestMessage] = useState("");

    const handleTestConnection = async () => {
        setTestStatus('testing');
        setTestMessage("");
        try {
            await invoke("test_local_configuration", { whisperPath, modelPath });
            setTestStatus('success');
            setTestMessage("Ready to rock! 🚀");
            setTimeout(() => setTestStatus('idle'), 3000);
        } catch (err) {
            setTestStatus('error');
            setTestMessage(String(err));
        }
    };

    if (!isOpen) return null;

    return (
        <div className="absolute bottom-20 left-1/2 -translate-x-1/2 w-72 z-50 bg-zinc-900 border border-white/10 rounded-2xl shadow-2xl p-4 flex flex-col gap-3">
            <div className="flex justify-between items-center border-b border-white/10 pb-2">
                <span className="text-xs font-semibold text-white/60 tracking-wider uppercase">
                    Settings
                </span>
                <button
                    onClick={onClose}
                    className="text-white/40 hover:text-white transition-colors"
                >
                    <X size={14} />
                </button>
            </div>
            <div className="flex items-center justify-between">
                <div className="flex flex-col">
                    <span className="text-sm font-medium text-white">Language / Sprache</span>
                    <span className="text-[10px] text-white/40">For translation & prompt</span>
                </div>
                <select
                    value={selectedLanguage}
                    onChange={(e) => setSelectedLanguage(e.target.value)}
                    className="bg-zinc-700 border border-white/10 rounded px-2 py-1 text-xs text-white focus:outline-none focus:border-violet-500/50 w-24"
                >
                    <option value="auto">Auto</option>
                    <option value="de">Deutsch</option>
                    <option value="en">English</option>
                    <option value="es">Español</option>
                    <option value="fr">Français</option>
                </select>
            </div>

            <div className="flex items-center justify-between">
                <div className="flex flex-col">
                    <span className="text-sm font-medium text-white">Skill / Modus</span>
                    <span className="text-[10px] text-white/40">Processing behavior</span>
                </div>
                <select
                    value={activeSkill}
                    onChange={(e) => setActiveSkill(e.target.value)}
                    className="bg-zinc-700 border border-white/10 rounded px-2 py-1 text-xs text-white focus:outline-none focus:border-violet-500/50 w-32"
                >
                    <option value="auto">🛡️ Standard</option>
                    <option value="summary">📝 Zusammenfassung</option>
                    <option value="email">✉️ Email Drafter</option>
                    <option value="todo">✅ To-Do Liste</option>
                </select>
            </div>

            <div className="h-px bg-white/10 my-1" />

            <div className="flex items-center justify-between">
                <div className="flex flex-col">
                    <span className="text-sm font-medium text-white">Local Mode (Offline)</span>
                    <span className="text-[10px] text-white/40">Maximum Privacy. Runs 100% on device via Ollama.</span>
                </div>

                {/* Custom Toggle Switch */}
                <button
                    onClick={onTogglePrivacy}
                    className={`relative w-10 h-6 rounded-full transition-colors duration-200 focus:outline-none ${isPrivacyMode ? "bg-violet-500" : "bg-zinc-700"
                        }`}
                >
                    <span
                        className={`absolute top-1 left-1 bg-white w-4 h-4 rounded-full shadow-sm transition-transform duration-200 ${isPrivacyMode ? "translate-x-4" : "translate-x-0"
                            }`}
                    />
                </button>
            </div>

            {isPrivacyMode && (
                <div className="flex flex-col gap-3 mt-2 animate-in fade-in slide-in-from-top-1 duration-200">
                    <div className="text-[10px] text-violet-300 bg-violet-500/10 p-2 rounded border border-violet-500/20">
                        🛡️ Local Mode active. Data stays on device.
                    </div>

                    <div className="space-y-1">
                        <label className="text-[10px] text-white/60 uppercase font-semibold">Local Whisper Binary Path</label>
                        <input
                            type="text"
                            value={whisperPath}
                            onChange={(e) => setWhisperPath(e.target.value)}
                            placeholder="C:\path\to\whisper-cli.exe"
                            className="w-full bg-black/40 border border-white/10 rounded px-2 py-1 text-xs text-white placeholder-white/20 focus:outline-none focus:border-violet-500/50"
                        />
                    </div>

                    <div className="space-y-1">
                        <label className="text-[10px] text-white/60 uppercase font-semibold">Model Path (.bin)</label>
                        <input
                            type="text"
                            value={modelPath}
                            onChange={(e) => setModelPath(e.target.value)}
                            placeholder="C:\path\to\ggml-base.bin"
                            className="w-full bg-black/40 border border-white/10 rounded px-2 py-1 text-xs text-white placeholder-white/20 focus:outline-none focus:border-violet-500/50"
                        />
                    </div>

                    <div className="space-y-1">
                        <label className="text-[10px] text-white/60 uppercase font-semibold">FFmpeg Path (Optional)</label>
                        <input
                            type="text"
                            value={ffmpegPath}
                            onChange={(e) => setFfmpegPath(e.target.value)}
                            placeholder="C:\path\to\ffmpeg.exe"
                            className="w-full bg-black/40 border border-white/10 rounded px-2 py-1 text-xs text-white placeholder-white/20 focus:outline-none focus:border-violet-500/50"
                        />
                    </div>



                    <button
                        onClick={handleTestConnection}
                        disabled={testStatus === 'testing' || !whisperPath || !modelPath}
                        className="flex items-center justify-center gap-2 w-full bg-white/5 hover:bg-white/10 text-xs text-white py-2 rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                        {testStatus === 'testing' && <Loader2 size={12} className="animate-spin" />}
                        {testStatus === 'success' && <CheckCircle2 size={12} className="text-green-400" />}
                        {testStatus === 'error' && <AlertCircle size={12} className="text-red-400" />}
                        {testStatus === 'idle' ? "Test Connection" : testMessage}
                    </button>

                    {testStatus === 'error' && (
                        <p className="text-[10px] text-red-400 leading-tight">
                            {testMessage}
                        </p>
                    )}
                </div>
            )}
        </div>
    );
}
