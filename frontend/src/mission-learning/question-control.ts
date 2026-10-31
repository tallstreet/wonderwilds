import type {
  MissionLearningAnswer,
  QuestionInteraction,
  QuestionStimulus,
} from '../../../contracts/generated/contracts.ts';
import { pointerMoves } from '../learning/drag.ts';
import { activityIcon, pieceIcon } from './icons.ts';
import { renderStimulus } from './stimulus.ts';
import './question.css';
import { spokenNumber } from './answer.ts';

/** Reusable reviewed controls. Collects typed answers; assessment stays in Rust. */
export function mountQuestionControl(
  root: HTMLElement,
  interaction: QuestionInteraction,
  stimuli: QuestionStimulus[],
  enabled: () => boolean,
  changed: () => void,
) {
  const doc = root.ownerDocument;
  root.classList.add('ww-question');
  let answer: MissionLearningAnswer | undefined;
  const drag = pointerMoves(root, enabled);
  const node = <K extends keyof HTMLElementTagNameMap>(tag: K, text = '', cls = '') => {
    const e = doc.createElement(tag);
    e.textContent = text;
    e.className = cls;
    return e;
  };
  const button = (label: string, action: () => void, cls = 'ww-question-piece') => {
    const b = node('button', label, cls);
    b.type = 'button';
    b.onclick = () => {
      if (enabled()) action();
    };
    return b;
  };
  const emit = (value: MissionLearningAnswer | undefined) => {
    answer = value;
    changed();
  };
  const stimulusRoot = node('div', '', 'ww-question-stimuli');
  stimuli.forEach((stimulus) => stimulusRoot.append(renderStimulus(doc, stimulus)));
  root.append(stimulusRoot);
  let refresh = () => {};
  let canUndo = false;
  const textInputs: HTMLInputElement[] = [];
  let draft: HTMLTextAreaElement | undefined;
  if (interaction.kind === 'single_choice' || interaction.kind === 'multi_choice') {
    const fieldset = node('fieldset', '', 'ww-question-choices');
    fieldset.append(
      node(
        'legend',
        interaction.kind === 'single_choice' ? 'Choose one answer' : 'Choose all that apply',
        'ww-mission-sr-only',
      ),
    );
    const selected = new Set<string>();
    for (const option of interaction.options) {
      const label = node('label', '', 'ww-question-choice');
      const input = node('input');
      input.type = interaction.kind === 'single_choice' ? 'radio' : 'checkbox';
      input.name = `question-${crypto.randomUUID()}`;
      // One radio group per mounted activity.
      if (interaction.kind === 'single_choice')
        input.name = `question-${root.dataset.group ?? (root.dataset.group = crypto.randomUUID())}`;
      input.value = option.id;
      input.setAttribute('aria-label', option.label);
      const caption = node('span', 'Choose', 'ww-question-caption');
      if (option.picture) label.append(activityIcon(option.picture));
      else
        label.append(
          node(
            'strong',
            option.label,
            /^-?\d+(\.\d+)?$/.test(option.label)
              ? 'ww-question-option-label is-number'
              : 'ww-question-option-label',
          ),
        );
      if (option.picture) label.append(node('span', option.label));
      label.prepend(input);
      label.append(caption);
      input.onchange = () => {
        if (!enabled()) return;
        if (interaction.kind === 'single_choice') selected.clear();
        if (input.checked) selected.add(option.id);
        else selected.delete(option.id);
        fieldset.querySelectorAll('label').forEach((l) => {
          l.querySelector('.ww-question-caption')!.textContent = l.querySelector('input')!.checked
            ? '✓ Selected'
            : 'Choose';
        });
        emit(
          selected.size
            ? interaction.kind === 'single_choice'
              ? { kind: 'choice', choiceId: option.id }
              : { kind: 'choices', choiceIds: [...selected] }
            : undefined,
        );
      };
      fieldset.append(label);
    }
    root.append(fieldset);
  } else if (interaction.kind === 'number_entry' || interaction.kind === 'text_entry') {
    const fields =
      interaction.kind === 'number_entry'
        ? interaction.fields
        : [{ id: 'word', label: interaction.label, format: 'text' }];
    const entries = new Map<string, HTMLInputElement>();
    const read = () => {
      if (interaction.kind === 'text_entry') {
        const v = textInputs[0]!.value.trim();
        emit(v ? { kind: 'text', value: v } : undefined);
        return;
      }
      const valid = interaction.fields.every((field) => {
        const text = entries.get(field.id)!.value.trim();
        return (
          text.length <= 32 &&
          (field.format === 'integer'
            ? /^-?\d+$/.test(text)
            : field.format === 'decimal'
              ? /^-?\d+(\.\d{1,6})?$/.test(text)
              : /^-?\d+(\.\d{1,6})?$|^-?\d+\/[1-9]\d*$/.test(text))
        );
      });
      emit(
        valid
          ? {
              kind: 'numbers',
              values: interaction.fields.map((field) => ({
                fieldId: field.id,
                value: entries.get(field.id)!.value.trim(),
              })),
            }
          : undefined,
      );
    };
    for (const field of fields) {
      const label = node('label', field.label, 'ww-question-entry');
      const input = node('input');
      input.autocomplete = 'off';
      input.spellcheck = false;
      input.maxLength = interaction.kind === 'text_entry' ? interaction.maxLength : 32;
      input.inputMode =
        field.format === 'integer' ? 'numeric' : field.format === 'decimal' ? 'decimal' : 'text';
      input.placeholder = field.format === 'text' ? 'Type here' : '?';
      input.oninput = read;
      label.append(input);
      if ('unit' in field && field.unit) label.append(node('span', field.unit));
      root.append(label);
      textInputs.push(input);
      entries.set(field.id, input);
    }
  } else if (interaction.kind === 'workshop') {
    root.append(node('p', interaction.scaffold, 'ww-question-scaffold'));
    const label = node(
      'label',
      'Your ideas (write, draw or talk with a grown-up)',
      'ww-question-entry',
    );
    draft = node('textarea');
    draft.rows = 5;
    draft.maxLength = 16000;
    draft.placeholder = 'Start your idea here…';
    label.append(draft);
    root.append(label);
    const rubric = node('ul', '', 'ww-question-rubric');
    interaction.rubric.forEach((rule) => rubric.append(node('li', rule)));
    root.append(
      rubric,
      node(
        'p',
        'A grown-up reviews this work. It is not automatically marked. This draft stays on this screen and is discarded when you close it.',
        'ww-question-note',
      ),
    );
  } else {
    const ordering = interaction.kind === 'order';
    const items = interaction.items;
    const targets = ordering
      ? items.map((_, index) => ({
          id: `place-${index}`,
          label: ['First', 'Next', 'Then', 'After that', 'Next', 'Then', 'Next', 'Last'][index]!,
        }))
      : interaction.targets;
    const assignments = new Map<string, string>();
    const history: [string, string][][] = [];
    let selected: string | undefined;
    const bank = node('div', '', 'ww-question-bank');
    const places = node('div', '', 'ww-question-places');
    const guidance = node(
      'p',
      'Choose a piece, then a place. You can drag too.',
      'ww-question-gesture',
    );
    guidance.setAttribute('role', 'status');
    const pieces = new Map<string, HTMLButtonElement>();
    const slots = new Map<string, HTMLButtonElement>();
    const capacity = ordering ? 1 : interaction.capacity;
    const undo = button(
      'Undo',
      () => {
        const previous = history.pop();
        if (!previous) return;
        assignments.clear();
        previous.forEach(([a, b]) => assignments.set(a, b));
        selected = undefined;
        refresh();
      },
      'ww-question-undo',
    );
    undo.prepend(activityIcon('undo'));
    refresh = () => {
      pieces.forEach((piece, id) => {
        piece.setAttribute('aria-pressed', String(selected === id));
        piece.classList.toggle('is-placed', assignments.has(id));
        piece.querySelector('.ww-question-caption')!.textContent = assignments.has(id)
          ? '✓ Placed'
          : selected === id
            ? 'Selected'
            : 'Choose';
      });
      targets.forEach((target) => {
        const content = [...assignments]
          .filter(([, place]) => place === target.id)
          .map(([id]) => items.find((v) => v.id === id)!.label);
        const slot = slots.get(target.id)!;
        slot.textContent = `${target.label} · ${content.length ? content.join(', ') : '+'}`;
        slot.classList.toggle('filled', content.length > 0);
      });
      canUndo = history.length > 0;
      undo.disabled = !enabled() || !canUndo;
      guidance.textContent = selected
        ? `${items.find((i) => i.id === selected)!.label} selected. Choose a place.`
        : 'Choose a piece, then a place. You can drag too.';
      if (assignments.size !== items.length) {
        emit(undefined);
        return;
      }
      if (ordering)
        emit({
          kind: 'order',
          itemIds: targets.map((t) => [...assignments].find(([, place]) => place === t.id)![0]),
        });
      else
        emit({
          kind: 'assign',
          assignments: [...assignments].map(([objectId, targetId]) => ({ objectId, targetId })),
        });
    };
    const place = (id: string, target: string) => {
      if (!enabled() || !pieces.has(id)) return;
      history.push([...assignments]);
      const occupants = [...assignments].filter(([key, value]) => value === target && key !== id);
      if (occupants.length >= capacity) assignments.delete(occupants[0]![0]);
      assignments.set(id, target);
      selected = undefined;
      refresh();
    };
    items.forEach((item) => {
      const piece = button('', () => {
        selected = selected === item.id ? undefined : item.id;
        refresh();
      });
      piece.setAttribute('aria-label', `Choose ${item.label}`);
      piece.append(
        ...(item.label.length === 1
          ? []
          : [item.picture ? activityIcon(item.picture) : pieceIcon(item.label)]),
        node('strong', item.label),
        node('span', 'Choose', 'ww-question-caption'),
      );
      pieces.set(item.id, piece);
      bank.append(piece);
      drag.source(piece, item.id);
    });
    targets.forEach((target) => {
      const slot = button(
        target.label,
        () => {
          if (selected) place(selected, target.id);
          else {
            const last = [...assignments].filter(([, place]) => place === target.id).at(-1)?.[0];
            if (last) {
              history.push([...assignments]);
              assignments.delete(last);
              refresh();
            }
          }
        },
        'ww-question-place',
      );
      slots.set(target.id, slot);
      places.append(slot);
      drag.target(slot, (id) => place(id, target.id));
    });
    root.append(bank, guidance, places, undo);
    refresh();
  }
  let wasDisabled = false;
  return {
    answer: () => answer,
    update() {
      const disabled = !enabled();
      root
        .querySelectorAll<HTMLInputElement | HTMLButtonElement | HTMLTextAreaElement>(
          'input,button,textarea',
        )
        .forEach((input) => (input.disabled = disabled));
      if (disabled) {
        drag.cancel();
        wasDisabled = true;
      } else if (wasDisabled) {
        wasDisabled = false;
        refresh();
      } else {
        const undo = root.querySelector<HTMLButtonElement>('.ww-question-undo');
        if (undo) undo.disabled = !canUndo;
      }
    },
    selectChoice(id: string) {
      if (interaction.kind !== 'single_choice' || !enabled()) return;
      const input = [...root.querySelectorAll<HTMLInputElement>('input[type="radio"]')].find(
        (input) => input.value === id,
      );
      if (input) {
        input.checked = true;
        input.dispatchEvent(new Event('change'));
      }
    },
    useText(value: string) {
      if (draft) {
        draft.value = value.slice(0, 16000);
        return true;
      }
      if (textInputs.length !== 1) return false;
      if (interaction.kind === 'number_entry') {
        const numeric = spokenNumber(value);
        if (numeric === undefined) return false;
        const format = interaction.fields[0]!.format;
        if (
          (format === 'integer' && !/^-?\d+$/.test(numeric)) ||
          (format === 'decimal' && numeric.includes('/'))
        )
          return false;
        value = numeric;
      }
      textInputs[0]!.value = value;
      textInputs[0]!.dispatchEvent(new Event('input'));
      return true;
    },
    destroy() {
      root.querySelectorAll('audio').forEach((audio) => {
        audio.pause();
        audio.removeAttribute('src');
        audio.load();
      });
      drag.destroy();
      root.remove();
    },
  };
}
