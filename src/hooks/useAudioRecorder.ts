import { useState, useRef, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export type RecordingState = 'idle' | 'recording' | 'processing';

interface AudioRecorderResult {
    state: RecordingState;
    error: string | null;
    stream: MediaStream | null;
    startRecording: () => Promise<void>;
    stopRecording: () => Promise<void>;
    cancelRecording: () => Promise<void>;
    isPaused: boolean;
    setIsPaused: (paused: boolean) => void;
}

export function useAudioRecorder(
    isPrivacyMode: boolean,
    currentWhisperPath: string,
    currentModelPath: string,
    currentFfmpegPath: string,
    selectedLanguage: string,
    activeSkill: string // New parameter
): AudioRecorderResult {
    const [state, setState] = useState<RecordingState>('idle');
    const [error, setError] = useState<string | null>(null);
    const [stream, setStream] = useState<MediaStream | null>(null);
    const mediaRecorderRef = useRef<MediaRecorder | null>(null);
    const chunksRef = useRef<Blob[]>([]);
    const streamRef = useRef<MediaStream | null>(null);
    const startTimeRef = useRef<number>(0);

    const [isPaused, setIsPaused] = useState(false);

    // Refs to prevent stale closures in event listeners
    const privacyModeRef = useRef(isPrivacyMode);
    const whisperPathRef = useRef(currentWhisperPath);
    const modelPathRef = useRef(currentModelPath);
    const ffmpegPathRef = useRef(currentFfmpegPath);
    const selectedLanguageRef = useRef(selectedLanguage);
    const activeSkillRef = useRef(activeSkill);
    const isPausedRef = useRef(isPaused);

    // Keep refs in sync
    useEffect(() => {
        privacyModeRef.current = isPrivacyMode;
    }, [isPrivacyMode]);

    useEffect(() => {
        whisperPathRef.current = currentWhisperPath;
    }, [currentWhisperPath]);

    useEffect(() => {
        modelPathRef.current = currentModelPath;
    }, [currentModelPath]);

    useEffect(() => {
        ffmpegPathRef.current = currentFfmpegPath;
    }, [currentFfmpegPath]);

    useEffect(() => {
        selectedLanguageRef.current = selectedLanguage;
    }, [selectedLanguage]);

    useEffect(() => {
        activeSkillRef.current = activeSkill;
    }, [activeSkill]);

    useEffect(() => {
        isPausedRef.current = isPaused;
    }, [isPaused]);


    const isCancelledRef = useRef(false);

    // ... (keep refs in sync) ...

    const startRecording = useCallback(async () => {
        if (state !== 'idle') {

            return;
        }

        try {
            setError(null);
            isCancelledRef.current = false; // Reset cancel flag

            // Ensure previous recording is fully cleaned up
            if (streamRef.current) {
                streamRef.current.getTracks().forEach(track => track.stop());
                streamRef.current = null;
            }
            // Reset other refs
            setStream(null);
            mediaRecorderRef.current = null;
            chunksRef.current = [];
            startTimeRef.current = 0;



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
                if (event.data.size > 0 && !isPausedRef.current) {
                    chunksRef.current.push(event.data);
                }
            };

            mediaRecorder.onstop = async () => {
                // If paused or cancelled, discard audio
                if (isPausedRef.current || isCancelledRef.current) {


                    // Cleanup
                    if (streamRef.current) {
                        streamRef.current.getTracks().forEach(track => track.stop());
                        streamRef.current = null;
                    }
                    setStream(null);
                    mediaRecorderRef.current = null;
                    chunksRef.current = [];
                    startTimeRef.current = 0;
                    setState('idle');
                    return;
                }

                // Calculate recording duration
                const durationMs = Date.now() - startTimeRef.current;
                const currentPrivacyMode = privacyModeRef.current;
                const currentWhisperPath = whisperPathRef.current;
                const currentModelPath = modelPathRef.current;
                const currentSelectedLanguage = selectedLanguageRef.current;
                const currentActiveSkill = activeSkillRef.current;




                setState('processing');

                // Copy chunks before cleanup
                const chunks = [...chunksRef.current];

                if (chunks.length === 0) {

                    // cleanup inline
                    if (streamRef.current) {
                        streamRef.current.getTracks().forEach(track => track.stop());
                        streamRef.current = null;
                    }
                    setStream(null);
                    mediaRecorderRef.current = null;
                    chunksRef.current = [];
                    startTimeRef.current = 0;

                    setState('idle');
                    return;
                }

                // Combine all chunks into single blob
                const audioBlob = new Blob(chunks, { type: 'audio/webm' });


                const arrayBuffer = await audioBlob.arrayBuffer();
                const audioData = Array.from(new Uint8Array(arrayBuffer));

                // Cleanup before sending
                if (streamRef.current) {
                    streamRef.current.getTracks().forEach(track => track.stop());
                    streamRef.current = null;
                }
                setStream(null);
                mediaRecorderRef.current = null;
                chunksRef.current = [];
                startTimeRef.current = 0;

                try {
                    // Send to Rust backend with duration and privacy mode
                    await invoke<string>('process_audio', {
                        audioData,
                        durationMs,
                        privacyMode: currentPrivacyMode,
                        whisperPath: currentWhisperPath,
                        modelPath: currentModelPath,
                        ffmpegPath: ffmpegPathRef.current,
                        language: currentSelectedLanguage,
                        skill: currentActiveSkill // Pass active skill
                    });
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

        } catch (err) {
            console.error('❌ Microphone error:', err);
            setError(err instanceof Error ? err.message : 'Mikrofon-Zugriff fehlgeschlagen');
            setState('idle');
        }
    }, [state]);

    const stopRecording = useCallback(async () => {
        if (mediaRecorderRef.current && state === 'recording') {

            mediaRecorderRef.current.stop();
        }
    }, [state]);

    const cancelRecording = useCallback(async () => {
        if (mediaRecorderRef.current && state === 'recording') {

            isCancelledRef.current = true;
            mediaRecorderRef.current.stop();
        }
    }, [state]);

    const setPaused = useCallback((paused: boolean) => {
        setIsPaused(paused);
        isPausedRef.current = paused;

        if (paused) {
            // Stop recording immediately to release microphone
            if (mediaRecorderRef.current && mediaRecorderRef.current.state === 'recording') {

                mediaRecorderRef.current.stop();
            }
        } else {
            // Resume recording if we are effectively idle (and not processing)

            setTimeout(() => {
                startRecording();
            }, 100);
        }
    }, [state, startRecording]);

    return { state, error, stream, startRecording, stopRecording, cancelRecording, isPaused, setIsPaused: setPaused };
}
