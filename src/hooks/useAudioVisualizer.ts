import { useState, useRef, useEffect, useCallback } from 'react';

interface AudioVisualizerResult {
    bars: number[];  // Normalized values 0-1 for each bar
    startAnalyzing: (stream: MediaStream) => void;
    stopAnalyzing: () => void;
}

const NUM_BARS = 5;
const FFT_SIZE = 256;

export function useAudioVisualizer(): AudioVisualizerResult {
    const [bars, setBars] = useState<number[]>(new Array(NUM_BARS).fill(0));
    const audioContextRef = useRef<AudioContext | null>(null);
    const analyserRef = useRef<AnalyserNode | null>(null);
    const sourceRef = useRef<MediaStreamAudioSourceNode | null>(null);
    const animationFrameRef = useRef<number | null>(null);
    const isRunningRef = useRef(false);

    const analyze = useCallback(() => {
        if (!analyserRef.current || !isRunningRef.current) return;

        const analyser = analyserRef.current;
        const dataArray = new Uint8Array(analyser.frequencyBinCount);
        analyser.getByteFrequencyData(dataArray);

        // Sample from different frequency ranges for visual variety
        const step = Math.floor(dataArray.length / (NUM_BARS + 1));
        const newBars: number[] = [];

        for (let i = 0; i < NUM_BARS; i++) {
            // Take average of a small range for smoother visualization
            const startIdx = (i + 1) * step - 2;
            const endIdx = (i + 1) * step + 2;
            let sum = 0;
            let count = 0;

            for (let j = Math.max(0, startIdx); j < Math.min(dataArray.length, endIdx); j++) {
                sum += dataArray[j];
                count++;
            }

            // Normalize to 0-1 with some amplification for visual effect
            const normalized = Math.min(1, (sum / count / 255) * 1.5);
            newBars.push(normalized);
        }

        setBars(newBars);
        animationFrameRef.current = requestAnimationFrame(analyze);
    }, []);

    const startAnalyzing = useCallback((stream: MediaStream) => {
        // Create audio context
        audioContextRef.current = new AudioContext();
        analyserRef.current = audioContextRef.current.createAnalyser();
        analyserRef.current.fftSize = FFT_SIZE;
        analyserRef.current.smoothingTimeConstant = 0.7;

        // Connect stream to analyser
        sourceRef.current = audioContextRef.current.createMediaStreamSource(stream);
        sourceRef.current.connect(analyserRef.current);

        // Start animation loop
        isRunningRef.current = true;
        analyze();
    }, [analyze]);

    const stopAnalyzing = useCallback(() => {
        isRunningRef.current = false;

        if (animationFrameRef.current) {
            cancelAnimationFrame(animationFrameRef.current);
            animationFrameRef.current = null;
        }

        if (sourceRef.current) {
            sourceRef.current.disconnect();
            sourceRef.current = null;
        }

        if (audioContextRef.current) {
            audioContextRef.current.close();
            audioContextRef.current = null;
        }

        analyserRef.current = null;
        setBars(new Array(NUM_BARS).fill(0));
    }, []);

    // Cleanup on unmount
    useEffect(() => {
        return () => {
            stopAnalyzing();
        };
    }, [stopAnalyzing]);

    return { bars, startAnalyzing, stopAnalyzing };
}
