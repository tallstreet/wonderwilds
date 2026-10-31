import { pointerMoves } from '../learning/drag.ts';
import { picture } from '../learning/visuals.ts';

export type ControlKind =
  | 'single'
  | 'multiple'
  | 'number'
  | 'text'
  | 'order'
  | 'assign'
  | 'workshop';
export type PreviewAnswer = string | string[] | Record<string, string>;

/** UI-only samples. These collect answers; no correctness or learning evidence is produced. */
export function mountReviewControl(root: HTMLElement, kind: ControlKind, onChange: () => void) {
  const document = root.ownerDocument;
  const drag = pointerMoves(root, () => !root.closest<HTMLElement>('[hidden]'));
  let answer: PreviewAnswer | undefined;
  const node = <K extends keyof HTMLElementTagNameMap>(tag: K, text = '', className = '') => {
    const element = document.createElement(tag);
    element.textContent = text;
    element.className = className;
    return element;
  };
  const changed = (value: PreviewAnswer | undefined) => {
    answer = value;
    onChange();
  };
  const button = (label: string, action: () => void, className = 'piece') => {
    const result = node('button', label, className);
    result.type = 'button';
    result.addEventListener('click', action);
    return result;
  };

  if (kind === 'single' || kind === 'multiple') {
    const fieldset = node('fieldset', '', 'choice-grid');
    fieldset.append(
      node(
        'legend',
        kind === 'single' ? 'Choose one answer' : 'Choose all the triangles',
        'sr-only',
      ),
    );
    const selected = new Set<string>();
    const options =
      kind === 'single' ? ['8', '10'] : ['triangle', 'circle', 'square', 'rotated triangle'];
    for (const [index, value] of options.entries()) {
      const label = node('label', '', 'choice');
      const input = node('input');
      input.type = kind === 'single' ? 'radio' : 'checkbox';
      input.name = `preview-${kind}`;
      input.value = value;
      input.setAttribute(
        'aria-label',
        kind === 'single'
          ? value
          : `Shape ${index + 1}: ${['three straight sides, pointing up', 'a curved round edge', 'four equal straight sides', 'three straight sides, pointing right'][index]}`,
      );
      const drawing =
        kind === 'single' ? node('span', value, 'big-number') : picture(document, value);
      if (value === 'rotated triangle') drawing.classList.add('rotated');
      const caption = node(
        'span',
        kind === 'single' ? 'Choose' : `Shape ${index + 1}`,
        'choice-caption',
      );
      label.append(input, drawing, caption);
      input.addEventListener('change', () => {
        if (kind === 'single') selected.clear();
        if (input.checked) selected.add(value);
        else selected.delete(value);
        fieldset.querySelectorAll('label').forEach((item) => {
          const checked = item.querySelector('input')!.checked;
          item.querySelector('.choice-caption')!.textContent = checked
            ? '✓ Selected'
            : kind === 'single'
              ? 'Choose'
              : `Shape ${options.indexOf(item.querySelector('input')!.value) + 1}`;
        });
        changed(selected.size ? (kind === 'single' ? value : [...selected]) : undefined);
      });
      fieldset.append(label);
    }
    root.append(fieldset);
  } else if (kind === 'number' || kind === 'text' || kind === 'workshop') {
    if (kind === 'number') root.append(node('div', '3 + 4 = ?', 'equation'));
    if (kind === 'text') {
      const stimulus = node('div', '', 'word-picture');
      stimulus.setAttribute('role', 'img');
      stimulus.setAttribute('aria-label', 'A hat');
      stimulus.append(picture(document, 'hat'));
      root.append(stimulus);
    }
    const label = node(
      'label',
      kind === 'number' ? 'Your number' : kind === 'text' ? 'Your word' : 'Your invitation',
      'entry-label',
    );
    const input = kind === 'workshop' ? node('textarea') : node('input');
    input.id = `preview-${kind}-input`;
    label.htmlFor = input.id;
    input.maxLength = kind === 'workshop' ? 600 : kind === 'text' ? 24 : 2;
    input.placeholder = kind === 'workshop' ? 'Dear Pip, …' : kind === 'text' ? 'Type here' : '?';
    if (input instanceof HTMLInputElement) {
      input.autocomplete = 'off';
      input.spellcheck = false;
      input.inputMode = kind === 'number' ? 'numeric' : 'text';
      if (kind === 'number') input.pattern = '[0-9]{1,2}';
    } else input.rows = 5;
    input.addEventListener('input', () => {
      const value = input.value.trim();
      changed(value && (kind !== 'number' || /^\d{1,2}$/.test(value)) ? value : undefined);
    });
    root.append(label, input);
    if (kind === 'workshop') {
      root.append(
        node('p', 'Include who you are inviting, what you will do, and where to meet.', 'scaffold'),
      );
      root.append(
        node(
          'p',
          'Read your invitation with a grown-up. Writing is shared, not automatically marked. This draft stays on this page and clears when you reload.',
          'workshop-note',
        ),
      );
    }
  } else {
    const ordering = kind === 'order';
    const names = ordering ? ['Flower', 'Seed', 'Sprout'] : ['Berry', 'Leaf', 'Flower'];
    const targets = ordering ? ['First', 'Next', 'Last'] : ['Pip', 'Clover', 'Wren'];
    const bank = node('div', '', 'piece-bank');
    bank.setAttribute('role', 'group');
    bank.setAttribute('aria-label', ordering ? 'Plant cards' : 'Gifts');
    const places = node('div', '', 'places');
    const guidance = node(
      'p',
      'Choose a card, then choose its place. You can drag it too.',
      'gesture-note',
    );
    guidance.setAttribute('role', 'status');
    const assignments = new Map<number, number>();
    const history: [number, number][][] = [];
    let selected: number | undefined;
    const pieces: HTMLButtonElement[] = [];
    const slots: HTMLButtonElement[] = [];
    const update = () => {
      pieces.forEach((piece, index) => {
        piece.setAttribute('aria-pressed', String(selected === index));
        piece.classList.toggle('is-placed', assignments.has(index));
        piece.querySelector('.piece-state')!.textContent = assignments.has(index)
          ? '✓ Placed'
          : selected === index
            ? 'Selected'
            : 'Choose';
      });
      slots.forEach((slot, index) => {
        const piece = [...assignments].find(([, place]) => place === index)?.[0];
        slot.textContent = `${targets[index]}${piece === undefined ? ' · +' : ` · ${names[piece]}`}`;
        slot.classList.toggle('filled', piece !== undefined);
      });
      undo.disabled = history.length === 0;
      guidance.textContent =
        selected === undefined
          ? 'Choose a card, then choose its place. You can drag it too.'
          : `${names[selected]} selected. Choose a place below.`;
      changed(
        assignments.size !== names.length
          ? undefined
          : ordering
            ? targets.map(
                (_, index) => names[[...assignments].find(([, place]) => place === index)![0]]!,
              )
            : Object.fromEntries(
                [...assignments].map(([piece, place]) => [names[piece]!, targets[place]!]),
              ),
      );
    };
    const place = (piece: number, target: number) => {
      if (!Number.isInteger(piece) || piece < 0 || piece >= names.length) return;
      history.push([...assignments]);
      for (const [other, destination] of assignments)
        if (destination === target) assignments.delete(other);
      assignments.set(piece, target);
      selected = undefined;
      update();
    };
    names.forEach((name, index) => {
      const piece = button('', () => {
        selected = selected === index ? undefined : index;
        update();
      });
      piece.setAttribute('aria-label', `Choose ${name.toLowerCase()}`);
      const drawing = picture(document, name.toLowerCase());
      if (name === 'Seed' || name === 'Sprout') {
        drawing.innerHTML =
          name === 'Seed'
            ? '<svg viewBox="0 0 180 140" focusable="false"><ellipse cx="90" cy="82" rx="32" ry="21" transform="rotate(-25 90 82)" fill="#b58a56"/><path d="M74 82q13-17 31-13" stroke="#e5c99d" stroke-width="7" fill="none" stroke-linecap="round"/></svg>'
            : '<svg viewBox="0 0 180 140" focusable="false"><ellipse cx="90" cy="120" rx="45" ry="8" fill="#e1d9bf"/><path d="M90 115V65" stroke="#5d9e68" stroke-width="8" stroke-linecap="round"/><path d="M90 83Q39 76 53 44Q90 44 90 83m0-9q17-44 47-33q9 34-47 33" fill="#92c877"/></svg>';
      }
      piece.append(drawing, node('strong', name), node('span', 'Choose', 'piece-state'));
      drag.source(piece, String(index));
      pieces.push(piece);
      bank.append(piece);
    });
    targets.forEach((target, index) => {
      const slot = button(
        target,
        () => {
          if (selected !== undefined) place(selected, index);
          else {
            const previous = [...assignments].find(([, destination]) => destination === index)?.[0];
            if (previous !== undefined) {
              history.push([...assignments]);
              assignments.delete(previous);
              update();
            }
          }
        },
        'place',
      );
      drag.target(slot, (key) => place(Number(key), index));
      slots.push(slot);
      places.append(slot);
    });
    const undo = button(
      '↶ Undo',
      () => {
        const previous = history.pop();
        if (!previous) return;
        assignments.clear();
        previous.forEach(([piece, target]) => assignments.set(piece, target));
        selected = undefined;
        update();
      },
      'quiet-button',
    );
    root.append(bank, guidance, places, undo);
    update();
  }
  return {
    answer: () => answer,
    cancel: () => drag.cancel(),
    destroy: () => drag.destroy(),
  };
}
