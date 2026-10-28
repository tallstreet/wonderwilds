import type { Answer, EncounterInstance } from '../../../contracts/generated/contracts.ts';
import './activity.css';

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
  root.setAttribute('aria-labelledby', heading.id);
  const prompt = document.createElement('p');
  prompt.id = `${id}-prompt`;
  prompt.textContent = presentation.prompt;
  root.setAttribute('aria-describedby', prompt.id);
  const live = document.createElement('p');
  live.className = 'ww-activity-status';
  live.setAttribute('role', 'status');
  live.setAttribute('aria-live', 'polite');
  live.setAttribute('aria-atomic', 'true');
  const controls = document.createElement('div');
  const help = document.createElement('p');
  help.textContent =
    'Use the buttons with touch, a mouse, or Tab and Enter. You can change your answer before checking.';
  root.append(heading, prompt, help, controls);
  let destroyed = false;
  let busy = false;
  let answer: () => Answer = () => ({ kind: 'choices', choiceIds: [] });
  let refresh: () => void = () => {};
  const buttons: HTMLButtonElement[] = [];
  const conditions = new Map<HTMLButtonElement, () => boolean>();

  function button(label: string, action: () => void, disabled: () => boolean = () => false) {
    const node = document.createElement('button');
    node.type = 'button';
    node.textContent = label;
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
  function countable(label: string, count: number, target: HTMLElement) {
    target.replaceChildren();
    const text = document.createElement('span');
    text.textContent = `${label}: ${count}`;
    const objects = document.createElement('span');
    objects.className = 'ww-activity-objects';
    objects.setAttribute('aria-hidden', 'true');
    for (let i = 0; i < count; i++) {
      const object = document.createElement('span');
      object.className = 'ww-activity-object';
      object.textContent = '●';
      objects.append(object);
    }
    target.append(text, objects);
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
        !Number.isSafeInteger(('given' in parameters ? parameters.given : undefined) ?? 0) ||
        parameters.total < 1 ||
        parameters.total > 20 ||
        (('given' in parameters ? parameters.given : undefined) ?? 0) < 0 ||
        (('given' in parameters ? parameters.given : undefined) ?? 0) > parameters.total
      )
        return unsupported();
      const given = ('given' in parameters ? parameters.given : undefined) ?? 0;
      let value = 0;
      const existing = document.createElement('p');
      countable(`Ready objects (target ${parameters.total})`, given, existing);
      const current = document.createElement('p');
      const remove = button(
        'Remove one object',
        () => {
          value--;
          update();
        },
        () => value === 0,
      );
      const add = button(
        'Add one object',
        () => {
          value++;
          update();
        },
        () => value >= parameters.total,
      );
      controls.append(existing, current, remove, add);
      refresh = () => countable('Objects you added', value, current);
      answer = () => ({ kind: 'quantity', value });
      return true;
    }
    if (encounter.templateId === 'equal_sharing') {
      if (
        !('recipients' in parameters) ||
        !Number.isSafeInteger(parameters.total) ||
        !Number.isSafeInteger(parameters.recipients) ||
        parameters.total < 1 ||
        parameters.total > 20 ||
        parameters.recipients < 2 ||
        parameters.recipients > 6
      )
        return unsupported();
      const groups = Array<number>(parameters.recipients).fill(0);
      const remaining = document.createElement('p');
      controls.append(remaining);
      const displays: HTMLElement[] = [];
      groups.forEach((_, index) => {
        const group = document.createElement('fieldset');
        const legend = document.createElement('legend');
        legend.textContent = `Friend ${index + 1}`;
        const display = document.createElement('p');
        displays.push(display);
        const add = button(
          `Give one object to friend ${index + 1}`,
          () => {
            groups[index]++;
            update();
          },
          () => groups.reduce((a, b) => a + b, 0) >= parameters.total,
        );
        const remove = button(
          `Take one object back from friend ${index + 1}`,
          () => {
            groups[index]--;
            update();
          },
          () => groups[index] === 0,
        );
        group.append(legend, display, add, remove);
        controls.append(group);
      });
      refresh = () => {
        countable(
          'Objects left to share',
          parameters.total - groups.reduce((a, b) => a + b, 0),
          remaining,
        );
        displays.forEach((display, index) =>
          countable(`Friend ${index + 1} has`, groups[index], display),
        );
      };
      answer = () => ({ kind: 'groups', groups: [...groups] });
      return true;
    }
    const choices: { id: string; label: string }[] = [];
    if (encounter.templateId === 'pattern_sort' || encounter.templateId === 'spatial_measurement') {
      if (
        !('objects' in parameters) ||
        parameters.objects.length < 1 ||
        parameters.objects.length > 20
      )
        return unsupported();
      const shown = parameters.objects.filter(
        (object) => 'attribute' in object && object.attribute === 'shown',
      );
      parameters.objects
        .filter((object) => !('attribute' in object) || object.attribute !== 'shown')
        .forEach((object) =>
          choices.push({
            id: object.id,
            label:
              'length' in object && object.length !== undefined
                ? `${object.label}, length ${object.length} units`
                : object.label,
          }),
        );
      if (shown.length) {
        const pattern = document.createElement('p');
        pattern.textContent = `Pattern so far: ${shown.map((object) => object.label).join(', ')}. What comes next?`;
        controls.append(pattern);
      } else if (
        'unitLength' in parameters &&
        parameters.unitLength &&
        'prefixLength' in parameters &&
        parameters.prefixLength !== undefined
      ) {
        // Earlier contract fixtures describe the repeating unit rather than an explicit prefix.
        if (
          !Number.isSafeInteger(parameters.unitLength) ||
          parameters.unitLength < 1 ||
          parameters.unitLength > choices.length ||
          !Number.isSafeInteger(parameters.prefixLength) ||
          parameters.prefixLength < 0 ||
          parameters.prefixLength > 20
        )
          return unsupported();
        const pattern = document.createElement('p');
        pattern.textContent = `Pattern so far: ${Array.from({ length: parameters.prefixLength }, (_, i) => choices[i % (parameters.unitLength ?? 1)].label).join(', ')}. What comes next?`;
        controls.append(pattern);
      }
      if (!choices.length) return unsupported();
    } else if (encounter.templateId === 'phonics_word_building') {
      if (!('tiles' in parameters) || parameters.tiles.length < 1 || parameters.tiles.length > 20)
        return unsupported();
      parameters.tiles.forEach((tile, index) => choices.push({ id: String(index), label: tile }));
    } else if (encounter.templateId === 'story_message') {
      if (!('cards' in parameters) || parameters.cards.length < 1 || parameters.cards.length > 20)
        return unsupported();
      parameters.cards.forEach((card) => choices.push({ id: card.id, label: card.text }));
    } else return unsupported();
    if (new Set(choices.map((choice) => choice.id)).size !== choices.length) return unsupported();
    const mode =
      presentation.choiceMode ??
      (encounter.templateId === 'phonics_word_building' || encounter.templateId === 'story_message'
        ? 'ordered'
        : 'single');
    let selected: string[] = [];
    const selection = document.createElement('p');
    const list = document.createElement('div');
    list.className = 'ww-activity-choices';
    const choiceButtons = choices.map((choice, index) => {
      const label =
        encounter.templateId === 'phonics_word_building'
          ? `Choose tile ${index + 1}: ${choice.label}`
          : `Choose: ${choice.label}`;
      const node = button(
        label,
        () => {
          if (mode === 'single') selected = [choice.id];
          else if (mode === 'multiple' && selected.includes(choice.id))
            selected = selected.filter((value) => value !== choice.id);
          else selected.push(choice.id);
          update();
          if (mode === 'ordered') {
            const next = [...list.querySelectorAll('button')].find(
              (candidate) => !candidate.disabled,
            );
            (next ?? undo).focus();
          }
        },
        () => mode === 'ordered' && selected.includes(choice.id),
      );
      list.append(node);
      return node;
    });
    const undo = button(
      'Undo last choice',
      () => {
        selected.pop();
        update();
      },
      () => selected.length === 0,
    );
    const clear = button(
      'Clear answer',
      () => {
        selected = [];
        update();
      },
      () => selected.length === 0,
    );
    controls.append(selection, list, undo, clear);
    refresh = () => {
      selection.textContent = selected.length
        ? `Your ${mode === 'ordered' ? 'order' : 'choice'}: ${selected.map((choiceId) => choices.find((c) => c.id === choiceId)?.label).join(', ')}`
        : 'No choices yet.';
      choiceButtons.forEach((node, index) =>
        node.setAttribute('aria-pressed', String(selected.includes(choices[index].id))),
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
      callbacks.stopNarration?.();
      root.remove();
    },
  };
}
