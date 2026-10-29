export type AnswerRecorder = {
  start(): Promise<void>;
  finish(): Promise<string>;
  cancel(): void;
};

export function pcmWav(samples: Float32Array): ArrayBuffer {
  const buffer = new ArrayBuffer(44 + samples.length * 2);
  const view = new DataView(buffer);
  const write = (offset: number, text: string) =>
    [...text].forEach((c, i) => view.setUint8(offset + i, c.charCodeAt(0)));
  write(0, 'RIFF');
  view.setUint32(4, buffer.byteLength - 8, true);
  write(8, 'WAVEfmt ');
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, 1, true);
  view.setUint32(24, 16000, true);
  view.setUint32(28, 32000, true);
  view.setUint16(32, 2, true);
  view.setUint16(34, 16, true);
  write(36, 'data');
  view.setUint32(40, samples.length * 2, true);
  samples.forEach((sample, i) =>
    view.setInt16(
      44 + i * 2,
      Math.round(Math.max(-1, Math.min(1, sample)) * (sample < 0 ? 32768 : 32767)),
      true,
    ),
  );
  return buffer;
}

export function createRecorder(csrf: string, profileId: string): AnswerRecorder {
  let recorder: MediaRecorder | undefined;
  let stream: MediaStream | undefined;
  let chunks: Blob[] = [];
  let stopped: Promise<void> | undefined;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let controller = new AbortController();
  let generation = 0;
  function stop() {
    clearTimeout(timer);
    if (recorder?.state === 'recording') recorder.stop();
    stream?.getTracks().forEach((track) => track.stop());
  }
  function cancel() {
    generation++;
    controller.abort();
    stop();
    chunks = [];
    recorder = undefined;
    stream = undefined;
  }
  return {
    cancel,
    async start() {
      cancel();
      const current = generation;
      controller = new AbortController();
      const acquired = await navigator.mediaDevices.getUserMedia({ audio: true });
      if (current !== generation) {
        acquired.getTracks().forEach((track) => track.stop());
        throw new Error('Cancelled');
      }
      stream = acquired;
      try {
        recorder = new MediaRecorder(stream);
        const activeRecorder = recorder;
        stopped = new Promise<void>((resolve, reject) => {
          activeRecorder.ondataavailable = (event) => {
            if (current === generation && event.data.size) chunks.push(event.data);
          };
          activeRecorder.onstop = () => resolve();
          activeRecorder.onerror = () => {
            stop();
            reject(new Error('Recording failed'));
          };
        });
        // Handle errors even when the child has not pressed Finish yet.
        void stopped.catch(() => {});
        recorder.start();
        timer = setTimeout(stop, 20000);
      } catch (error) {
        cancel();
        throw error;
      }
    },
    async finish() {
      if (!recorder || !stopped) throw new Error('No recording');
      const current = generation;
      const signal = controller.signal;
      stop();
      await stopped;
      const recording = new Blob(chunks, { type: recorder.mimeType });
      chunks = [];
      const context = new AudioContext();
      let decoded: AudioBuffer;
      try {
        decoded = await context.decodeAudioData(await recording.arrayBuffer());
      } finally {
        await context.close();
      }
      if (current !== generation || decoded.duration <= 0) throw new Error('Cancelled');
      const frames = Math.min(320000, Math.ceil(decoded.duration * 16000));
      const offline = new OfflineAudioContext(1, frames, 16000);
      const source = offline.createBufferSource();
      source.buffer = decoded;
      source.connect(offline.destination);
      source.start();
      const audio = await offline.startRendering();
      if (current !== generation) throw new Error('Cancelled');
      const response = await fetch('/__demo/transcribe', {
        method: 'POST',
        signal: AbortSignal.any([signal, AbortSignal.timeout(35000)]),
        headers: {
          'Content-Type': 'audio/wav',
          'X-CSRF-Token': csrf,
          'X-Wonderwilds-Profile': profileId,
        },
        body: pcmWav(audio.getChannelData(0)),
      });
      if (!response.ok || !response.headers.get('content-type')?.startsWith('text/plain'))
        throw new Error('Transcription failed');
      const text = await response.text();
      if (current !== generation || text.length > 300) throw new Error('Invalid transcript');
      return text;
    },
  };
}
