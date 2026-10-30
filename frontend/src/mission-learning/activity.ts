import type {
  MissionActivity,
  MissionLearningAnswer,
  MissionLearningSnapshot,
  MissionChoice,
} from '../../../contracts/generated/contracts.ts';
import './activity.css';

export type MissionLearningActivityCallbacks = {
  onAttempt(answer: MissionLearningAnswer): Promise<MissionLearningSnapshot>;
  onSupport(kind: 'hint' | 'worked_example'): Promise<MissionLearningSnapshot>;
  onRetry(): Promise<MissionLearningSnapshot>;
  onRefresh(): Promise<MissionLearningSnapshot>;
  hasPending(): boolean;
  onAbandon(): Promise<void>;
  onContinue(): void;
  onFinishTogether?(): Promise<MissionLearningSnapshot>;
};

/** Render the server-authored lesson and typed task without assessing or revealing answers locally. */
export function mountMissionActivity(
  container: HTMLElement,
  activity: MissionActivity,
  initialSnapshot: MissionLearningSnapshot,
  callbacks: MissionLearningActivityCallbacks,
): { destroy(): void; focus(): void } {
  const document = container.ownerDocument;
  const root = document.createElement('section');
  root.className = 'ww-mission-activity';
  const heading = document.createElement('h2');
  heading.id = container.closest('dialog')
    ? 'dialog-title'
    : `ww-mission-heading-${crypto.randomUUID()}`;
  heading.tabIndex = -1;
  heading.textContent = activity.storyStep?.label ?? activity.presentation.title;
  root.setAttribute('aria-labelledby', heading.id);

  const intro = document.createElement('div');
  intro.className = 'ww-mission-intro';
  const objective = document.createElement('p');
  objective.className = 'ww-mission-objective';
  objective.textContent = activity.presentation.objective;
  const lesson = document.createElement('p');
  lesson.textContent = activity.presentation.lessonText;
  const context = document.createElement('p');
  context.className = 'ww-mission-context';
  context.textContent = activity.storyStep?.introduction ?? activity.presentation.context;
  if (activity.storyStep) intro.append(context);
  else intro.append(objective, lesson, context);

  const prompt = document.createElement('p');
  prompt.className = 'ww-mission-prompt';
  prompt.textContent = activity.presentation.prompt;
  const form = document.createElement('form');
  form.className = 'ww-mission-task';
  form.addEventListener('submit', (event) => event.preventDefault());
  const status = document.createElement('p');
  status.className = 'ww-mission-status';
  status.setAttribute('role', 'status');
  status.setAttribute('aria-live', 'polite');
  status.setAttribute('aria-atomic', 'true');
  const help = document.createElement('section');
  help.className = 'ww-mission-help';
  help.setAttribute('aria-label', 'Optional help');
  const supportText = document.createElement('p');
  supportText.setAttribute('role', 'note');
  supportText.className = 'ww-mission-support-text';
  supportText.hidden = true;
  const actions = document.createElement('div');
  actions.className = 'ww-mission-actions';
  const check = document.createElement('button');
  check.type = 'button';
  check.className = 'primary';
  check.textContent = 'Check my idea';
  const hint = document.createElement('button');
  hint.type = 'button';
  hint.className = 'secondary';
  hint.textContent = 'Ask for a hint';
  const example = document.createElement('button');
  example.type = 'button';
  example.className = 'secondary';
  example.textContent = 'Show a worked example';
  const abandon = document.createElement('button');
  abandon.type = 'button';
  abandon.className = 'text-button';
  abandon.textContent = activity.storyStep ? 'Put this task aside' : 'Leave practice';
  const together = document.createElement('button');
  together.type = 'button';
  together.className = 'secondary';
  together.textContent = 'Let’s do it together';
  together.hidden = !callbacks.onFinishTogether;
  actions.append(check, hint, example, together, abandon);
  root.append(heading, intro, prompt, form, status, help, actions);
  container.replaceChildren(root);

  let choiceId: string | undefined;
  let assignments = new Map<string, string>();
  let currentActivity = activity;
  let busy = false;
  let complete =
    !initialSnapshot.activeActivity && initialSnapshot.lastFeedback?.completed === true;
  let destroyed = false;
  let retryAvailable = false;
  let refreshAvailable = false;
  const retry = document.createElement('button');
  retry.type = 'button';
  retry.className = 'primary';
  retry.textContent = 'Retry the same saved action';
  retry.hidden = true;
  help.append(supportText, retry);
  const refresh = document.createElement('button');
  refresh.type = 'button';
  refresh.className = 'secondary';
  refresh.textContent = 'Refresh practice progress';
  refresh.hidden = true;
  help.append(refresh);
  const inputs: (HTMLInputElement | HTMLSelectElement)[] = [];

  const node = <K extends keyof HTMLElementTagNameMap>(tag: K, className: string, text = '') => {
    const element = document.createElement(tag);
    element.className = className;
    element.textContent = text;
    return element;
  };

  function pairAnswer(): MissionLearningAnswer | undefined {
    const task = activity.task;
    if (task.kind !== 'pairing') return undefined;
    const objectFirst = task.layout === 'object_cards';
    const rows = objectFirst ? task.objects : task.recipients;
    if (assignments.size !== rows.length || rows.some((row) => !assignments.has(row.id)))
      return undefined;
    return {
      kind: 'pairing',
      assignments: rows.map((row) => ({
        objectId: objectFirst ? row.id : assignments.get(row.id)!,
        recipientId: objectFirst ? assignments.get(row.id)! : row.id,
      })),
    };
  }

  function selectedAnswer(): MissionLearningAnswer | undefined {
    if (activity.task.kind === 'pairing') return pairAnswer();
    return choiceId ? { kind: 'choice', choiceId } : undefined;
  }

  function update() {
    if (destroyed) return;
    const answer = selectedAnswer();
    const pending = callbacks.hasPending();
    check.disabled = busy || complete || pending || !answer;
    hint.disabled = busy || complete || pending || currentActivity.support.hintCount >= 20;
    example.disabled = busy || complete || pending || currentActivity.support.workedExample;
    abandon.disabled = busy || complete || pending;
    together.disabled = busy || complete || pending;
    retry.hidden = !retryAvailable;
    retry.disabled = busy || !retryAvailable;
    refresh.hidden = !refreshAvailable;
    refresh.disabled = busy || !refreshAvailable;
    inputs.forEach((input) => (input.disabled = busy || complete || pending));
    root.setAttribute('aria-busy', String(busy));
  }

  function renderChoices(choices: MissionChoice[]) {
    const fieldset = node('fieldset', 'ww-mission-options');
    const legend = document.createElement('legend');
    legend.textContent = 'Choose one answer';
    fieldset.append(legend);
    for (const choice of choices) {
      const label = node('label', 'ww-mission-choice');
      const input = document.createElement('input');
      input.type = 'radio';
      input.name = 'mission-choice';
      input.value = choice.id;
      input.addEventListener('change', () => {
        choiceId = choice.id;
        update();
      });
      inputs.push(input);
      const text = document.createElement('span');
      const title =
        choice.label ?? (choice.value === undefined ? 'Choice' : `Number ${choice.value}`);
      text.textContent = choice.value === undefined ? title : `${title} (${choice.value})`;
      label.append(input, text);
      fieldset.append(label);
    }
    form.append(fieldset);
  }

  const task = activity.task;
  if (task.kind === 'pairing') {
    if (task.objects.length !== task.recipients.length) {
      status.textContent = 'This pairing task is not available right now.';
      check.disabled = true;
    } else {
      const table = node('div', 'ww-mission-pairing');
      const objectFirst = task.layout === 'object_cards';
      const rows = objectFirst ? task.objects : task.recipients;
      for (const row of rows) {
        const label = node('label', 'ww-mission-pair-row');
        const caption = document.createElement('span');
        caption.textContent = objectFirst ? `Give ${row.label} to` : `${row.label} receives`;
        const select = document.createElement('select');
        select.setAttribute('aria-label', caption.textContent);
        const blank = document.createElement('option');
        blank.value = '';
        blank.textContent = 'Choose a match';
        select.append(blank);
        const options = objectFirst ? task.recipients : task.objects;
        for (const option of options) {
          const item = document.createElement('option');
          item.value = option.id;
          item.textContent = option.label;
          select.append(item);
        }
        select.addEventListener('change', () => {
          if (objectFirst) {
            if (select.value) assignments.set(row.id, select.value);
            else assignments.delete(row.id);
          } else {
            if (select.value) assignments.set(row.id, select.value);
            else assignments.delete(row.id);
          }
          update();
        });
        inputs.push(select);
        label.append(caption, select);
        table.append(label);
      }
      form.append(table);
    }
  } else if (task.kind === 'count_collection') {
    if (activity.representation === 'ten_frame') {
      const frame = node('div', 'ww-mission-ten-frames');
      frame.setAttribute('role', 'group');
      frame.setAttribute('aria-label', `Count ${task.objects.length} objects in a ten frame`);
      for (let row = 0; row < Math.ceil(task.objects.length / 10); row++) {
        const cells = node('div', 'ww-mission-ten-row');
        for (let column = 0; column < 10; column++) {
          const item = task.objects[row * 10 + column];
          const cell = node(
            'span',
            item ? 'ww-mission-ten-cell is-filled' : 'ww-mission-ten-cell',
            item?.label ?? '',
          );
          if (!item) cell.setAttribute('aria-label', 'empty space');
          cells.append(cell);
        }
        frame.append(cells);
      }
      form.append(frame);
    } else {
      const source = node('div', 'ww-mission-movable-source');
      source.setAttribute('role', 'group');
      source.setAttribute('aria-label', 'Objects to count');
      const counted = node('div', 'ww-mission-movable-count');
      counted.setAttribute('role', 'group');
      counted.setAttribute('aria-label', 'Objects counted so far');
      const placed = new Set<string>();
      for (const item of task.objects) {
        const piece = document.createElement('button');
        piece.type = 'button';
        piece.className = 'ww-mission-movable-object';
        piece.textContent = item.label;
        piece.setAttribute('aria-pressed', 'false');
        piece.addEventListener('click', () => {
          if (placed.has(item.id)) placed.delete(item.id);
          else placed.add(item.id);
          piece.setAttribute('aria-pressed', String(placed.has(item.id)));
          (placed.has(item.id) ? counted : source).append(piece);
        });
        source.append(piece);
      }
      form.append(source, counted);
    }
    renderChoices(task.choices);
  } else if (task.kind === 'compare_collections') {
    const collections = node(
      'div',
      activity.representation === 'aligned_rows'
        ? 'ww-mission-aligned-collections'
        : 'ww-mission-collections',
    );
    for (const group of task.collections) {
      const section = node('section', 'ww-mission-collection');
      const title = document.createElement('h3');
      title.textContent = group.label;
      const items = node(
        'ul',
        activity.representation === 'aligned_rows'
          ? 'ww-mission-objects ww-mission-aligned-items'
          : 'ww-mission-objects',
      );
      if (activity.representation === 'aligned_rows')
        items.style.gridTemplateRows = `repeat(${Math.max(...task.collections.map((entry) => entry.items.length))}, minmax(34px, auto))`;
      for (const item of group.items) items.append(node('li', 'ww-mission-object', item.label));
      section.append(title, items);
      collections.append(section);
    }
    form.append(collections);
    renderChoices(task.choices);
  } else if (task.kind === 'number_path') {
    const flowerSequence = activity.representation === 'countable_flower_sequence';
    const path = node('div', flowerSequence ? 'ww-mission-flower-path' : 'ww-mission-number-path');
    path.setAttribute('role', 'group');
    path.setAttribute(
      'aria-label',
      `Continue counting ${task.direction === 'forward' ? 'forwards' : 'backwards'}`,
    );
    for (const value of task.shown) {
      const item = node(
        'span',
        flowerSequence ? 'ww-mission-flower-group' : 'ww-mission-path-number',
      );
      item.textContent = flowerSequence ? '' : String(value);
      if (flowerSequence) {
        item.setAttribute('aria-label', `${value} ${activity.storyStep ? 'markers' : 'flowers'}`);
        for (let index = 0; index < value; index++)
          item.append(node('span', 'ww-mission-flower', activity.storyStep ? '◆' : '✿'));
        item.append(node('span', 'ww-mission-flower-count', String(value)));
      }
      path.append(item);
    }
    const missingStep = node(
      'span',
      flowerSequence ? 'ww-mission-flower-group' : 'ww-mission-path-number',
      '?',
    );
    if (flowerSequence)
      missingStep.setAttribute(
        'aria-label',
        activity.storyStep ? 'Next group of markers' : 'Next group of flowers',
      );
    path.append(missingStep);
    form.append(path);
    renderChoices(task.choices);
  } else {
    const countable = activity.representation === 'countable_planks';
    const material = activity.storyStep ? 'discovery markers' : 'planks';
    const equation = node('div', countable ? 'ww-mission-plank-part' : 'ww-mission-part-equation');
    if (countable) {
      equation.setAttribute('role', 'group');
      equation.setAttribute(
        'aria-label',
        `${task.shownPart} of ${task.whole} ${material} are ready. Choose how many are missing.`,
      );
      for (let index = 0; index < task.whole; index++)
        equation.append(
          node(
            'span',
            index < task.shownPart ? 'ww-mission-plank is-ready' : 'ww-mission-plank',
            index < task.shownPart ? '▰' : '□',
          ),
        );
    } else {
      equation.setAttribute('role', 'group');
      equation.setAttribute(
        'aria-label',
        `${task.shownPart} objects and some more make ${task.whole} objects altogether.`,
      );
      const known = node('span', 'ww-mission-whole-group');
      known.setAttribute('aria-label', `${task.shownPart} objects in the first part`);
      for (let index = 0; index < task.shownPart; index++)
        known.append(node('span', 'ww-mission-part-object', '●'));
      const missing = node('span', 'ww-mission-part-missing', '?');
      missing.setAttribute('aria-label', 'Objects in the missing part');
      const whole = node('span', 'ww-mission-whole-group');
      whole.setAttribute('aria-label', `${task.whole} objects altogether`);
      for (let index = 0; index < task.whole; index++)
        whole.append(node('span', 'ww-mission-part-object is-total', '○'));
      equation.append(
        known,
        node('span', 'ww-mission-part-symbol', '+'),
        missing,
        node('span', 'ww-mission-part-symbol', '='),
        whole,
      );
    }
    form.append(equation);
    renderChoices(task.choices);
  }

  function showSnapshot(snapshot: MissionLearningSnapshot) {
    if (destroyed) return;
    if (snapshot.activeActivity?.id === activity.id) currentActivity = snapshot.activeActivity;
    const feedback = snapshot.lastFeedback;
    if (feedback) {
      status.textContent = feedback.message;
      if (feedback.completed && !snapshot.activeActivity) {
        complete = true;
        const next = document.createElement('button');
        next.type = 'button';
        next.className = 'primary';
        next.textContent = 'Continue the story';
        next.addEventListener('click', callbacks.onContinue);
        help.replaceChildren(next);
      }
    }
    const revealed = snapshot.activeActivity?.revealedSupport;
    if (revealed) {
      supportText.textContent = revealed.text;
      supportText.hidden = false;
    }
    if (feedback) retryAvailable = false;
    update();
  }

  async function perform(operation: () => Promise<MissionLearningSnapshot>) {
    if (busy || destroyed || complete) return;
    busy = true;
    status.textContent = 'Saving your mission task…';
    update();
    try {
      showSnapshot(await operation());
    } catch {
      retryAvailable = callbacks.hasPending();
      refreshAvailable = !retryAvailable;
      status.textContent = retryAvailable
        ? 'We could not confirm that save. The same action is ready to retry.'
        : 'Progress changed. Refresh it before choosing another action.';
    } finally {
      busy = false;
      update();
    }
  }

  check.addEventListener('click', () => {
    const answer = selectedAnswer();
    if (!answer) return;
    void perform(() => callbacks.onAttempt(answer));
  });
  together.addEventListener('click', () => {
    if (callbacks.onFinishTogether) void perform(callbacks.onFinishTogether);
  });
  hint.addEventListener('click', () => void perform(() => callbacks.onSupport('hint')));
  example.addEventListener(
    'click',
    () => void perform(() => callbacks.onSupport('worked_example')),
  );
  retry.addEventListener('click', () => {
    if (!retryAvailable || busy || destroyed) return;
    busy = true;
    update();
    void callbacks
      .onRetry()
      .then((snapshot) => {
        retryAvailable = false;
        if (!snapshot.activeActivity) callbacks.onContinue();
        else showSnapshot(snapshot);
      })
      .catch(() => {
        retryAvailable = callbacks.hasPending();
        refreshAvailable = !retryAvailable;
        status.textContent = retryAvailable
          ? 'The same practice save could not be confirmed. Try again.'
          : 'Progress changed. Refresh it before choosing another action.';
      })
      .finally(() => {
        busy = false;
        update();
      });
  });
  refresh.addEventListener('click', () => {
    if (!refreshAvailable || busy || destroyed) return;
    busy = true;
    update();
    void callbacks
      .onRefresh()
      .then((snapshot) => {
        refreshAvailable = false;
        if (snapshot.activeActivity?.id === activity.id) showSnapshot(snapshot);
        else callbacks.onContinue();
      })
      .catch(() => {
        status.textContent =
          'Practice progress could not refresh. You can keep going with the story.';
      })
      .finally(() => {
        busy = false;
        update();
      });
  });
  abandon.addEventListener('click', () => {
    if (busy || complete) return;
    busy = true;
    update();
    void callbacks
      .onAbandon()
      .then(callbacks.onContinue)
      .catch(() => {
        retryAvailable = callbacks.hasPending();
        refreshAvailable = !retryAvailable;
        status.textContent = retryAvailable
          ? 'The leave action could not be confirmed. Retry the same save.'
          : 'Progress changed. Refresh it before choosing another action.';
      })
      .finally(() => {
        busy = false;
        update();
      });
  });
  if (initialSnapshot.activeActivity?.revealedSupport)
    supportText.textContent = initialSnapshot.activeActivity.revealedSupport.text;
  if (initialSnapshot.activeActivity?.revealedSupport) supportText.hidden = false;
  if (initialSnapshot.lastFeedback) showSnapshot(initialSnapshot);
  if (complete) {
    const next = document.createElement('button');
    next.type = 'button';
    next.className = 'primary';
    next.textContent = 'Continue the story';
    next.addEventListener('click', callbacks.onContinue);
    help.append(next);
  }
  update();

  return {
    destroy() {
      destroyed = true;
      root.remove();
    },
    focus() {
      heading.focus({ preventScroll: true });
    },
  };
}
