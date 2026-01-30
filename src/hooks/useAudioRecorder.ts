import { useState, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export type RecordingState = 'idle' | 'recording' | 'processing';

interface AudioRecorderResult {
    state: RecordingState;
    error: string | null;
    stream: MediaStream | null;
    startRecording: () => Promise<void>;
    stopRecording: () => Promise<void>;
}

export function useAudioRecorder(): AudioRecorderResult {
    const [state, setState] = useState<RecordingState>('idle');
    const [error, setError] = useState<string | null>(null);
    const [stream, setStream] = useState<MediaStream | null>(null);
    const mediaRecorderRef = useRef<MediaRecorder | null>(null);
    const chunksRef = useRef<Blob[]>([]);
    const streamRef = useRef<MediaStream | null>(null);
    const startTimeRef = useRef<number>(0);

    const cleanup = useCallback(() => {
        // Stop all tracks
        if (streamRef.current) {
            streamRef.current.getTracks().forEach(track => track.stop());
            streamRef.current = null;
        }
        // Clear stream state
        setStream(null);
        // Clear recorder reference
        mediaRecorderRef.current = null;
        // Clear chunks
        chunksRef.current = [];
        // Reset start time
        startTimeRef.current = 0;
    }, []);

    const startRecording = useCallback(async () => {
        if (state !== 'idle') {
            console.log('Cannot start recording - state is:', state);
            return;
        }

        try {
            setError(null);

            // Ensure previous recording is fully cleaned up
            cleanup();

            console.log('🎤 Requesting microphone...');
            const mediaStream = await navigator.mediaDevices.getUserMedia({
                audio: {
                    echoCancellation: true,
                    noiseSuppression: true,
                    sampleRate: 16000,
                }
            });
            streamRef.current = mediaStream;
            setStream(mediaStream);

            const mediaRecorder = new MediaRecorder(mediaStream, {
                mimeType: 'audio/webm;codecs=opus',
            });

            mediaRecorder.ondataavailable = (event) => {
                if (event.data.size > 0) {
                    chunksRef.current.push(event.data);
                }
            };

            mediaRecorder.onstop = async () => {
                // Calculate recording duration
                const durationMs = Date.now() - startTimeRef.current;
                console.log('📦 MediaRecorder stopped, chunks:', chunksRef.current.length, 'duration:', durationMs, 'ms');
                setState('processing');

                // Copy chunks before cleanup
                const chunks = [...chunksRef.current];

                if (chunks.length === 0) {
                    console.log('⚠️ No audio chunks recorded');
                    cleanup();
                    setState('idle');
                    return;
                }

                // Combine all chunks into single blob
                const audioBlob = new Blob(chunks, { type: 'audio/webm' });
                console.log('📦 Audio blob size:', audioBlob.size);

                const arrayBuffer = await audioBlob.arrayBuffer();
                const audioData = Array.from(new Uint8Array(arrayBuffer));

                // Cleanup before sending (allows new recording to start)
                cleanup();

                try {
                    // Send to Rust backend with duration
                    await invoke<string>('process_audio', { audioData, durationMs });
                } catch (err) {
                    console.error('❌ Process audio error:', err);
                    setError(err instanceof Error ? err.message : String(err));
                }

                setState('idle');
            };

            mediaRecorderRef.current = mediaRecorder;
            startTimeRef.current = Date.now();
            mediaRecorder.start(); // Collect all data in one single chunk at the end
            setState('recording');
            console.log('🔴 Recording started');
        } catch (err) {
            console.error('❌ Microphone error:', err);
            setError(err instanceof Error ? err.message : 'Mikrofon-Zugriff fehlgeschlagen');
            cleanup();
            setState('idle');
        }
    }, [state, cleanup]);

    const stopRecording = useCallback(async () => {
        if (mediaRecorderRef.current && state === 'recording') {
            console.log('⏹️ Stopping recording...');
            mediaRecorderRef.current.stop();
        }
    }, [state]);

    return { state, error, stream, startRecording, stopRecording };
}
