import { memo } from 'react';
import { RecordingState } from '../hooks/useAudioRecorder';

interface AudioVisualizerProps {
    bars: number[];
    state: RecordingState;
}

function AudioVisualizerComponent({ bars, state }: AudioVisualizerProps) {
    // Only show visualizer during recording
    if (state !== 'recording') {
        return null;
    }

    return (
        <div className="flex items-center gap-[3px] h-6">
            {bars.map((height, index) => (
                <div
                    key={index}
                    className="w-[4px] bg-red-500 rounded-full transition-all duration-75 ease-out"
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
