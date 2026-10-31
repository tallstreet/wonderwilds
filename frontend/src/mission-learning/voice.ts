import type { AnswerRecorder } from '../learning/voice.ts';
import { spokenCard } from './answer.ts';

/** Transcription is an editable input, never an answer check. */
export function voiceEntry(
  root: HTMLElement,
  recorder: AnswerRecorder,
  choices: { id: string; label: string }[],
  enabled: () => boolean,
  select: (id: string) => void,
  freeze: (active: boolean) => void,
  useDraft?: (text: string) => boolean | void,
) {
  const document = root.ownerDocument;
  const panel = document.createElement('fieldset');
  panel.className = 'ww-mission-voice';
  const legend = document.createElement('legend');
  legend.textContent = 'Or say your answer';
  const label = document.createElement('label');
  label.textContent = 'What I heard (you can change it)';
  const input = document.createElement('input');
  input.type = 'text';
  input.maxLength = 300;
  label.append(input);
  const message = document.createElement('p');
  message.setAttribute('role', 'status');
  const button = (text: string) => {
    const node = document.createElement('button');
    node.type = 'button';
    node.textContent = text;
    return node;
  };
  const start = button('Say my answer');
  const finish = button('Finish recording');
  const cancel = button('Cancel recording');
  const use = button('Use this answer');
  const privacy = document.createElement('p');
  privacy.textContent =
    'Recording sends up to 20 seconds of audio to Baseten for transcription. Review the words before using them.';
  panel.append(legend, start, finish, cancel, label, use, message, privacy);
  root.append(panel);
  let alive = true;
  let generation = 0;
  let recording = false;
  let finishing = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  function update() {
    const available = alive && enabled();
    start.disabled = !available || recording;
    input.disabled = !available || recording;
    use.disabled = !available || recording;
    finish.hidden = cancel.hidden = !recording;
    finish.disabled = finishing;
  }
  function reset() {
    generation++;
    clearTimeout(timer);
    recorder.cancel();
    recording = false;
    finishing = false;
    freeze(false);
    update();
  }
  start.onclick = async () => {
    if (!alive || !enabled() || recording) return;
    const current = ++generation;
    recording = true;
    finishing = true;
    freeze(true);
    update();
    message.textContent = 'Allow microphone access, then say your answer.';
    try {
      await recorder.start();
      if (!alive || current !== generation) return;
      finishing = false;
      update();
      finish.focus();
      message.textContent = 'Listening. Press Finish recording when you are ready.';
      timer = setTimeout(() => finish.click(), 20000);
    } catch {
      if (!alive || current !== generation) return;
      reset();
      message.textContent = 'The microphone is unavailable. You can still use the pieces.';
    }
  };
  finish.onclick = async () => {
    if (!alive || !recording || finishing) return;
    finishing = true;
    clearTimeout(timer);
    update();
    const current = generation;
    message.textContent = 'Turning your speech into words…';
    try {
      const text = await recorder.finish();
      if (!alive || current !== generation) return;
      if (text.trim()) input.value = text.slice(0, 300);
      message.textContent = text.trim()
        ? 'Check these words, then choose Use this answer.'
        : 'I could not hear an answer. Try again or use the pieces.';
    } catch {
      if (!alive || current !== generation) return;
      message.textContent =
        'That recording could not be understood. Your previous answer is still here.';
    } finally {
      if (alive && current === generation) {
        reset();
        input.focus();
      }
    }
  };
  cancel.onclick = () => {
    reset();
    message.textContent = 'Recording cancelled.';
    start.focus();
  };
  use.onclick = () => {
    if (!alive || !enabled() || recording) return;
    if (useDraft) {
      if (input.value.trim()) {
        const accepted = useDraft(input.value.trim());
        message.textContent =
          accepted === false
            ? 'I could not match that answer. Your previous answer is still here.'
            : 'Your words are ready. Check them before continuing.';
      } else message.textContent = 'Add some words to your draft first.';
      return;
    }
    const id = spokenCard(input.value, choices);
    if (!id) {
      message.textContent =
        'Say the number or the exact words on one card. You can also use the pieces.';
      return;
    }
    select(id);
    message.textContent = 'Your idea is ready. Choose Check my idea when you are happy with it.';
  };
  update();
  return {
    update,
    destroy() {
      alive = false;
      reset();
      panel.remove();
    },
  };
}
