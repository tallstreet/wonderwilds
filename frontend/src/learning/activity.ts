import type { Answer, EncounterInstance } from '../../../contracts/generated/contracts.ts';
import './activity.css';
import { art, picture } from './visuals.ts';
import { pointerMoves } from './drag.ts';

export type SupportKind = 'hint' | 'worked_example' | 'read_aloud';
export type ActivityFeedback = {
  message: string;
  result?: 'correct' | 'incorrect' | 'not_assessed';
};
export type SupportFeedback = { message: string; narrationText?: string };
export type ActivityPresentation = {
  /** Approved authored instructions, resolved from promptId/instructionId by the content adapter. */
  prompt: string;
  choiceMode?: 'single' | 'multiple' | 'ordered';
  narrationAvailable?: boolean;
};
export type ActivityCallbacks = {
  onAttempt(answer: Answer): Promise<ActivityFeedback>;
  /** Must record support authoritatively before resolving; rejection displays no hint/audio. */
  onSupport(kind: SupportKind): Promise<SupportFeedback>;
  /** Playback is opt-in and supplied by a reviewed local narration adapter, never browser speech here. */
  narrate?(text: string): Promise<void> | void;
  stopNarration?(): void;
  onAbandon?(): Promise<void>;
};

let nextActivityId = 0;

/** Mount an activity without assessing locally or granting rewards. The caller owns focus restoration. */
export function mountActivity(
  container: HTMLElement,
  encounter: EncounterInstance,
  callbacks: ActivityCallbacks,
  presentation: ActivityPresentation,
): { destroy(): void; focus(): void } {
  const document = container.ownerDocument;
  const root = document.createElement('section');
  root.className = 'ww-activity';
  const id = `ww-activity-${++nextActivityId}`;
  const heading = document.createElement('h2');
  heading.id = `${id}-heading`;
  heading.textContent = 'Help with an adventure';
  heading.tabIndex = -1;
  heading.className = 'ww-sr-only';
  root.setAttribute('aria-labelledby', heading.id);
  const prompt = document.createElement('p');
  prompt.id = `${id}-prompt`;
  prompt.textContent = presentation.prompt;
  prompt.className = 'ww-instruction';
  root.setAttribute('aria-describedby', prompt.id);
  const live = document.createElement('p');
  live.className = 'ww-activity-status';
  live.setAttribute('role', 'status');
  live.setAttribute('aria-live', 'polite');
  live.setAttribute('aria-atomic', 'true');
  const controls = document.createElement('div');
  const header = document.createElement('header');
  header.className = 'ww-activity-header';
  header.append(art(document, 'pip', 'hero'), prompt);
  const help = document.createElement('p');
  help.className = 'ww-gesture-help';
  help.textContent = 'Drag to move · Tap to choose';
  controls.className = 'ww-play-area';
  root.append(heading, header, help, controls);
  let destroyed = false;
  let busy = false;
  let answer: () => Answer = () => ({ kind: 'choices', choiceIds: [] });
  let refresh: () => void = () => {};
  const drag = pointerMoves(root, () => !busy && !destroyed);
  const buttons: HTMLButtonElement[] = [];
  const conditions = new Map<HTMLButtonElement, () => boolean>();

  function button(label: string, action: () => void, disabled: () => boolean = () => false) {
    const node = document.createElement('button');
    node.type = 'button';
    node.textContent = label;
    node.setAttribute('aria-label', label);
    node.addEventListener('click', () => {
      if (!busy && !destroyed) action();
    });
    buttons.push(node);
    conditions.set(node, disabled);
    return node;
  }
  function update() {
    if (destroyed) return;
    refresh();
    for (const node of buttons) node.disabled = busy || Boolean(conditions.get(node)?.());
    root.setAttribute('aria-busy', String(busy));
  }
  async function perform(operation: () => Promise<void>) {
    if (busy || destroyed) return;
    const previousFocus = document.activeElement;
    busy = true;
    drag.cancel();
    update();
    try {
      await operation();
    } catch {
      if (!destroyed)
        live.textContent =
          'That did not reach the adventure. Your answer is still here. Please try again.';
    } finally {
      busy = false;
      update();
      if (
        !destroyed &&
        previousFocus instanceof HTMLElement &&
        root.contains(previousFocus) &&
        document.activeElement === document.body
      )
        previousFocus.focus();
    }
  }
  function node(tag: 'div' | 'span' | 'p' | 'fieldset' | 'legend', className: string, text = '') {
    const item = document.createElement(tag);
    item.className = className;
    item.textContent = text;
    return item;
  }
  function iconButton(
    label: string,
    icon: string,
    action: () => void,
    disabled: () => boolean = () => false,
  ) {
    const item = button(label, action, disabled);
    item.textContent = icon;
    item.title = label;
    return item;
  }
  function unsupported() {
    controls.textContent = 'This activity needs an updated content adapter. Your progress is safe.';
    return false;
  }
  function configure(): boolean {
    const parameters = encounter.parameters;
    if (encounter.templateId === 'missing_quantity') {
      if (
        !('total' in parameters) ||
        !Number.isSafeInteger(parameters.total) ||
        parameters.total < 1 ||
        parameters.total > 20
      )
        return unsupported();
      const given = ('given' in parameters ? parameters.given : undefined) ?? 0;
      if (!Number.isSafeInteger(given) || given < 0 || given > parameters.total)
        return unsupported();
      const placed = new Set<number>();
      const supply = node('div', 'ww-supply ww-plank-supply');
      const deck = node('div', 'ww-bridge-deck');
      const river = node('div', 'ww-river');
      river.append(deck);
      const goal = node('p', 'ww-goal', `${parameters.total} planks altogether`);
      const count = node('p', 'ww-small-count');
      const move = (key: string, toDeck: boolean) => {
        const index = Number(key);
        if (!Number.isInteger(index) || index < 0 || index >= parameters.total) return;
        if (toDeck) placed.add(index);
        else placed.delete(index);
        update();
      };
      for (let i = 0; i < given; i++) {
        const fixed = art(document, 'plank');
        fixed.classList.add('ww-fixed-plank');
        deck.append(fixed);
      }
      const loose = Array.from({ length: parameters.total }, (_, i) => {
        const item = button(`Place plank ${i + 1}`, () => move(String(i), !placed.has(i)));
        item.className = 'ww-plank-piece';
        item.replaceChildren(art(document, 'plank'));
        drag.source(item, String(i));
        return item;
      });
      const spaces = node('div', 'ww-bridge-spaces');
      for (let i = 0; i < parameters.total - given; i++)
        spaces.append(node('span', 'ww-plank-space'));
      deck.append(spaces);
      const added = node('div', 'ww-added-planks');
      deck.append(added);
      drag.target(river, (key) => move(key, true));
      drag.target(supply, (key) => move(key, false));
      const tools = node('div', 'ww-small-tools');
      tools.append(
        iconButton(
          'Add one object',
          '＋',
          () => {
            const i = loose.findIndex((_, i) => !placed.has(i));
            if (i >= 0) move(String(i), true);
          },
          () => placed.size >= parameters.total,
        ),
        iconButton(
          'Remove one object',
          '↶',
          () => {
            const i = [...placed].at(-1);
            if (i !== undefined) move(String(i), false);
          },
          () => !placed.size,
        ),
      );
      controls.append(
        goal,
        river,
        node('p', 'ww-tray-label', 'Loose planks'),
        supply,
        tools,
        count,
      );
      refresh = () => {
        loose.forEach((item, i) => {
          const target = placed.has(i) ? added : supply;
          if (item.parentElement !== target) target.append(item);
          item.setAttribute('aria-label', `${placed.has(i) ? 'Return' : 'Place'} plank ${i + 1}`);
        });
        spaces.hidden = placed.size > 0;
        count.textContent = `${given} ready + ${placed.size} added`;
      };
      answer = () => ({ kind: 'quantity', value: placed.size });
      return true;
    }
    if (encounter.templateId === 'equal_sharing') {
      if (
        !('recipients' in parameters) ||
        !Number.isSafeInteger(parameters.total) ||
        parameters.total < 1 ||
        parameters.total > 20 ||
        !Number.isSafeInteger(parameters.recipients) ||
        parameters.recipients < 2 ||
        parameters.recipients > 6
      )
        return unsupported();
      const locations = Array<number>(parameters.total).fill(-1);
      help.textContent = 'Drag to a plate · Or tap a berry, then a plate';
      let selected = -1;
      const supply = node('div', 'ww-supply ww-berry-basket');
      const plates = node('div', 'ww-plates');
      const bowls: HTMLElement[] = [];
      const counts: HTMLElement[] = [];
      const move = (key: string, to: number) => {
        const i = Number(key);
        if (!Number.isInteger(i) || i < 0 || i >= locations.length) return;
        locations[i] = to;
        selected = -1;
        update();
      };
      const berries = locations.map((_, i) => {
        const item = button(`Berry ${i + 1}`, () => {
          if (locations[i] === -1) {
            selected = selected === i ? -1 : i;
            update();
          } else move(String(i), -1);
        });
        item.className = 'ww-berry-piece';
        item.replaceChildren(art(document, 'berry'));
        drag.source(item, String(i));
        return item;
      });
      drag.target(supply, (key) => move(key, -1));
      for (let i = 0; i < parameters.recipients; i++) {
        const plate = node('fieldset', 'ww-friend-plate');
        const legend = node('legend', 'ww-friend-name', i === 0 ? 'Pip' : `Friend ${i + 1}`);
        plate.append(
          legend,
          art(document, ['pip', 'rose', 'clover', 'poppy', 'gold', 'wren'][i], 'hero'),
        );
        const bowl = node('div', 'ww-plate-objects');
        bowls.push(bowl);
        const add = iconButton(
          `Give one object to friend ${i + 1}`,
          '＋',
          () => {
            const pick = selected >= 0 ? selected : locations.indexOf(-1);
            if (pick >= 0) move(String(pick), i);
          },
          () => !locations.includes(-1),
        );
        add.classList.add('ww-plate-add');
        const remove = iconButton(
          `Take one object back from friend ${i + 1}`,
          '↶',
          () => {
            const pick = locations.lastIndexOf(i);
            if (pick >= 0) move(String(pick), -1);
          },
          () => !locations.includes(i),
        );
        const count = node('span', 'ww-plate-count');
        counts.push(count);
        plate.append(bowl, add, remove, count);
        plates.append(plate);
        drag.target(plate, (key) => move(key, i));
        plate.addEventListener('click', (event) => {
          if (
            busy ||
            destroyed ||
            !(event.target instanceof Element) ||
            event.target.closest('button')
          )
            return;
          const pick = selected >= 0 ? selected : locations.indexOf(-1);
          if (pick >= 0) move(String(pick), i);
        });
      }
      const count = node('p', 'ww-small-count');
      controls.append(plates, node('p', 'ww-tray-label', 'Berries to share'), supply, count);
      refresh = () => {
        berries.forEach((item, i) => {
          const target = locations[i] < 0 ? supply : bowls[locations[i]];
          if (item.parentElement !== target) target.append(item);
          item.setAttribute('aria-pressed', String(selected === i));
        });
        counts.forEach((count, i) => {
          count.textContent = String(locations.filter((v) => v === i).length);
        });
        count.textContent = `${locations.filter((v) => v < 0).length} left in the basket`;
      };
      answer = () => ({
        kind: 'groups',
        groups: Array.from(
          { length: parameters.recipients },
          (_, i) => locations.filter((v) => v === i).length,
        ),
      });
      return true;
    }
    const choices: { id: string; label: string; length?: number }[] = [];
    const prefix: string[] = [];
    if (encounter.templateId === 'pattern_sort' || encounter.templateId === 'spatial_measurement') {
      if (
        !('objects' in parameters) ||
        parameters.objects.length < 1 ||
        parameters.objects.length > 20
      )
        return unsupported();
      parameters.objects.forEach((object) => {
        if ('attribute' in object && object.attribute === 'shown') prefix.push(object.label);
        else
          choices.push({
            id: object.id,
            label: object.label,
            length: 'length' in object ? object.length : undefined,
          });
      });
      if (
        !prefix.length &&
        'prefixLength' in parameters &&
        parameters.prefixLength !== undefined &&
        'unitLength' in parameters &&
        parameters.unitLength
      ) {
        if (
          !Number.isSafeInteger(parameters.unitLength) ||
          parameters.unitLength < 1 ||
          parameters.unitLength > choices.length ||
          !Number.isSafeInteger(parameters.prefixLength) ||
          parameters.prefixLength < 0 ||
          parameters.prefixLength > 20
        )
          return unsupported();
        for (let i = 0; i < parameters.prefixLength; i++)
          prefix.push(choices[i % parameters.unitLength].label);
      }
    } else if (encounter.templateId === 'phonics_word_building') {
      if (!('tiles' in parameters) || !parameters.tiles.length || parameters.tiles.length > 20)
        return unsupported();
      parameters.tiles.forEach((label, i) => choices.push({ id: String(i), label }));
    } else if (encounter.templateId === 'story_message') {
      if (!('cards' in parameters) || !parameters.cards.length || parameters.cards.length > 20)
        return unsupported();
      parameters.cards.forEach((card) => choices.push({ id: card.id, label: card.text }));
    } else return unsupported();
    if (!choices.length || new Set(choices.map((c) => c.id)).size !== choices.length)
      return unsupported();
    const maximumLength = Math.max(1, ...choices.map((choice) => choice.length ?? 0));
    const mode =
      presentation.choiceMode ??
      (['phonics_word_building', 'story_message'].includes(encounter.templateId)
        ? 'ordered'
        : 'single');
    let selected: string[] = [];
    const line = node('div', 'ww-pattern-line');
    prefix.forEach((label) => {
      const tile = node('span', 'ww-shown-tile');
      tile.append(picture(document, label));
      line.append(tile);
    });
    if (prefix.length) {
      const description = node(
        'p',
        'ww-sr-only',
        `Pattern so far: ${prefix.join(', ')}. What comes next?`,
      );
      controls.append(description, line);
    }
    const selection = node('p', 'ww-sr-only');
    const tray = node('div', 'ww-answer-tray');
    tray.setAttribute('aria-label', 'Your answer');
    const list = node('div', 'ww-activity-choices');
    const choose = (id: string, index?: number, drop = false) => {
      if (!choices.some((c) => c.id === id)) return;
      if (mode === 'single') selected = [id];
      else if (index !== undefined) {
        selected = selected.filter((value) => value !== id);
        selected.splice(Math.min(index, selected.length), 0, id);
      } else if (selected.includes(id)) {
        if (!drop && mode === 'multiple') selected = selected.filter((value) => value !== id);
      } else selected.push(id);
      update();
    };
    const resolve = (key: string) =>
      key.startsWith('selected:') ? selected[Number(key.slice(9))] : key.slice(7);
    const slots = Array.from({ length: mode === 'single' ? 1 : choices.length }, (_, i) => {
      const slot = button(`Answer position ${i + 1}`, () => {
        if (selected[i]) {
          selected.splice(i, 1);
          update();
        }
      });
      slot.className = 'ww-answer-slot';
      drag.source(slot, `selected:${i}`);
      drag.target(slot, (key) => {
        const id = resolve(key);
        if (id) choose(id, i, true);
      });
      tray.append(slot);
      return slot;
    });
    drag.target(tray, (key) => {
      const id = resolve(key);
      if (id) choose(id, undefined, true);
    });
    const choiceButtons = choices.map((choice, i) => {
      const label =
        encounter.templateId === 'phonics_word_building'
          ? `Choose tile ${i + 1}: ${choice.label}`
          : `Choose: ${choice.label}${choice.length !== undefined ? `, length ${choice.length} units` : ''}`;
      const item = button(
        label,
        () => {
          choose(choice.id);
          if (mode === 'ordered')
            (
              Array.from(list.querySelectorAll('button')).find((b) => !b.disabled) ?? slots[0]
            ).focus();
        },
        () => mode === 'ordered' && selected.includes(choice.id),
      );
      item.className = 'ww-choice-card';
      item.replaceChildren(picture(document, choice.label, choice.length, maximumLength));
      if (encounter.templateId === 'story_message') {
        const caption = node('span', 'ww-card-caption', choice.label);
        item.append(caption);
      }
      drag.source(item, `choice:${choice.id}`);
      list.append(item);
      return item;
    });
    drag.target(list, (key) => {
      if (key.startsWith('selected:')) {
        selected.splice(Number(key.slice(9)), 1);
        update();
      }
    });
    const tools = node('div', 'ww-small-tools');
    tools.append(
      iconButton(
        'Undo last choice',
        '↶',
        () => {
          selected.pop();
          update();
        },
        () => !selected.length,
      ),
      iconButton(
        'Clear answer',
        '↺',
        () => {
          selected = [];
          update();
        },
        () => !selected.length,
      ),
    );
    if (prefix.length) line.append(tray);
    else controls.append(tray);
    controls.append(
      selection,
      node(
        'p',
        'ww-tray-label',
        mode === 'ordered'
          ? 'Put them in order'
          : mode === 'multiple'
            ? 'Choose the ones you need'
            : 'Pick your piece',
      ),
      list,
      tools,
    );
    refresh = () => {
      selection.textContent = selected.length
        ? `Your ${mode === 'ordered' ? 'order' : 'choice'}: ${selected.map((id) => choices.find((c) => c.id === id)?.label).join(', ')}`
        : 'No choices yet.';
      slots.forEach((slot, i) => {
        const choice = choices.find((c) => c.id === selected[i]);
        slot.replaceChildren();
        if (choice) slot.append(picture(document, choice.label, choice.length, maximumLength));
        else slot.textContent = mode === 'ordered' ? String(i + 1) : '?';
        slot.classList.toggle('ww-filled', Boolean(choice));
        slot.setAttribute(
          'aria-label',
          choice ? `Remove ${choice.label} from position ${i + 1}` : `Answer position ${i + 1}`,
        );
      });
      choiceButtons.forEach((item, i) =>
        item.setAttribute('aria-pressed', String(selected.includes(choices[i].id))),
      );
    };
    answer = () => ({ kind: 'choices', choiceIds: [...selected] });
    return true;
  }
  const configured = presentation.prompt.trim().length > 0 && configure();
  if (!configured && !controls.textContent) unsupported();
  if (configured) {
    const check = button(
      'Check my answer',
      () => {
        void perform(async () => {
          const feedback = await callbacks.onAttempt(answer());
          if (!destroyed) live.textContent = feedback.message;
        });
      },
      () => !encounter.permittedActions.includes('attempt'),
    );
    const hint = button(
      'Show a hint',
      () => {
        void perform(async () => {
          const support = await callbacks.onSupport('hint');
          if (!destroyed) live.textContent = support.message;
        });
      },
      () => !encounter.permittedActions.includes('hint'),
    );
    const together = button(
      'Let’s do it together',
      () => {
        void perform(async () => {
          const support = await callbacks.onSupport('worked_example');
          if (!destroyed) live.textContent = support.message;
        });
      },
      () => !encounter.permittedActions.includes('worked_example'),
    );
    check.textContent = 'Check my idea ✓';
    check.classList.add('ww-check');
    hint.textContent = 'A little hint';
    together.textContent = 'Help me';
    const actions = document.createElement('div');
    actions.className = 'ww-activity-actions';
    actions.append(check, hint, together);
    if (presentation.narrationAvailable && callbacks.narrate) {
      actions.append(
        button(
          'Read the activity aloud (with support)',
          () => {
            void perform(async () => {
              const support = await callbacks.onSupport('read_aloud');
              if (destroyed) return;
              live.textContent = `${support.message} Reading aloud is recorded as support.`;
              if (support.narrationText) await callbacks.narrate?.(support.narrationText);
            });
          },
          () => !encounter.permittedActions.includes('hint'),
        ),
      );
      const stop = document.createElement('button');
      stop.type = 'button';
      stop.textContent = 'Stop narration';
      stop.addEventListener('click', () => {
        if (!destroyed) callbacks.stopNarration?.();
      });
      actions.append(stop);
    }
    if (callbacks.onAbandon)
      actions.append(
        button(
          'Return to exploring',
          () => {
            void perform(async () => {
              await callbacks.onAbandon?.();
            });
          },
          () => !encounter.permittedActions.includes('abandon'),
        ),
      );
    const notice = document.createElement('p');
    notice.textContent = ['E13', 'E14', 'E15'].includes(encounter.skillId)
      ? 'An adult can discuss this response with you. Choosing tiles does not certify writing or reasoning.'
      : 'Help always supports your adventure. Hints and reading support are recorded separately from independent practice.';
    notice.className = 'ww-sr-only';
    root.append(actions, notice);
  }
  root.append(live);
  container.append(root);
  update();
  return {
    focus: () => {
      if (!destroyed) heading.focus();
    },
    destroy: () => {
      destroyed = true;
      drag.destroy();
      callbacks.stopNarration?.();
      root.remove();
    },
  };
}
