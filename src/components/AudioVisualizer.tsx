import { memo } from 'react';
import { RecordingState } from '../hooks/useAudioRecorder';

interface AudioVisualizerProps {
    bars: number[];
    state: RecordingState;
    isPrivacyMode: boolean;
    isPaused?: boolean;
}

function AudioVisualizerComponent({ bars, state, isPrivacyMode, isPaused = false }: AudioVisualizerProps) {
    // Show visualizer if recording OR if paused (suspended state)
    // If state is 'recording' but we are 'isPaused', we show the suspended UI.
    if (state !== 'recording' && !isPaused) {
        return null;
    }

    if (isPaused) {
        return (
            <div className="flex items-center justify-center gap-[3px] h-6 w-24">
                {/* Breathing Amber Line */}
                <div className="w-full h-[2px] bg-amber-500 rounded-full animate-pulse opacity-75 shadow-[0_0_10px_rgba(245,158,11,0.5)]" />
            </div>
        );
    }

    const barColor = isPrivacyMode ? "bg-violet-500" : "bg-cyan-400";

    return (
        <div className="flex items-center justify-center gap-[3px] h-6 w-24">
            {bars.map((height, index) => (
                <div
                    key={index}
                    className={`w-[4px] rounded-full transition-all duration-75 ease-out ${barColor}`}
                    style={{
                        height: `${Math.max(4, height * 24)}px`,
                        opacity: 0.7 + height * 0.3,
                    }}
                />
            ))}
        </div>
    );
}

// Memoize to prevent unnecessary re-renders
export const AudioVisualizer = memo(AudioVisualizerComponent);
