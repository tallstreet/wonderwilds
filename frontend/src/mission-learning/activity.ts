import type {
  MissionActivity,
  MissionLearningAnswer,
  MissionLearningSnapshot,
  MissionChoice,
} from '../../../contracts/generated/contracts.ts';
import './activity.css';
import { mountQuestionControl } from './question-control.ts';
import { activityIcon } from './icons.ts';
import { pointerMoves } from '../learning/drag.ts';
import type { AnswerRecorder } from '../learning/voice.ts';
import { voiceEntry } from './voice.ts';
import { art, picture } from '../learning/visuals.ts';
import { ApiFailure } from '../api/client.ts';

export type MissionLearningActivityCallbacks = {
  recorder?: AnswerRecorder;
  onAttempt(answer: MissionLearningAnswer): Promise<MissionLearningSnapshot>;
  onSupport(kind: 'hint' | 'worked_example'): Promise<MissionLearningSnapshot>;
  onRetry(): Promise<MissionLearningSnapshot>;
  onRefresh(): Promise<MissionLearningSnapshot>;
  hasPending(): boolean;
  onAbandon(): Promise<void>;
  onContinue(): void;
  onReplace?(snapshot: MissionLearningSnapshot): void;
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
  const modern = activity.task.kind === 'question';
  if (modern) root.classList.add('ww-question-shell');
  root.dataset.activityVariant = activity.variantId;
  const heading = document.createElement('h2');
  heading.id = container.closest('dialog')
    ? 'dialog-title'
    : `ww-mission-heading-${crypto.randomUUID()}`;
  heading.tabIndex = -1;
  heading.textContent = activity.presentation.title;
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
  else if (!modern) intro.append(objective, lesson, context);
  else intro.append(context);

  const prompt = document.createElement('p');
  prompt.className = 'ww-mission-prompt';
  prompt.textContent = activity.presentation.prompt;
  if (modern) {
    const emphasis = ['one more', 'one less', 'all', 'in order', 'one gift'].find((word) =>
      activity.presentation.prompt.includes(word),
    );
    if (emphasis) {
      const [before, after] = activity.presentation.prompt.split(emphasis);
      const strong = document.createElement('strong');
      strong.textContent = emphasis;
      prompt.replaceChildren(before!, strong, after!);
    }
  }
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
  check.type = modern ? 'submit' : 'button';
  if (modern) {
    form.id = `question-form-${crypto.randomUUID()}`;
    check.setAttribute('form', form.id);
  }
  check.className = 'primary';
  check.textContent = modern ? 'Check answer' : 'Check my idea';
  const hint = document.createElement('button');
  hint.type = 'button';
  hint.className = 'secondary';
  hint.textContent = modern ? 'Hint' : 'Ask for a hint';
  const example = document.createElement('button');
  example.type = 'button';
  example.className = 'secondary';
  example.textContent = modern ? 'Show me' : 'Show a worked example';
  const abandon = document.createElement('button');
  abandon.type = 'button';
  abandon.className = 'text-button';
  abandon.textContent = activity.storyStep ? 'Put this task aside' : 'Leave practice';
  const together = document.createElement('button');
  together.type = 'button';
  together.className = 'secondary';
  together.textContent = 'Let’s do it together';
  together.hidden = !callbacks.onFinishTogether;
  if (modern) {
    check.prepend(activityIcon('check'));
    hint.prepend(activityIcon('hint'));
    example.prepend(activityIcon('example'));
    together.prepend(activityIcon('together'));
  }
  actions.append(check, hint, example, together, abandon);
  const welcome = document.createElement('header');
  welcome.className = 'ww-mission-welcome';
  if (modern) {
    const titles = document.createElement('div');
    const eyebrow = document.createElement('p');
    eyebrow.className = 'ww-question-eyebrow';
    eyebrow.textContent = activity.skillId;
    titles.append(eyebrow, heading);
    welcome.append(art(document, 'pip', 'hero'), titles);
  } else welcome.append(art(document, 'pip', 'hero'), heading);
  root.append(welcome, intro, prompt, form, status, help, actions);
  container.replaceChildren(root);

  let questionControl: ReturnType<typeof mountQuestionControl> | undefined;
  let choiceId: string | undefined;
  const assignments = new Map<string, string>();
  let currentActivity = activity;
  let busy = false;
  let complete = false;
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
  const inputs: (HTMLInputElement | HTMLButtonElement | HTMLTextAreaElement)[] = [];
  let recording = false;
  let voice: ReturnType<typeof voiceEntry> | undefined;
  const editable = () => !busy && !complete && !destroyed && !callbacks.hasPending();
  const drag = pointerMoves(root, () => editable() && !recording);
  const gestureHelp = document.createElement('p');
  gestureHelp.className = 'ww-mission-gesture-help';
  gestureHelp.textContent = 'Drag a piece to its place, or tap a piece and then a place.';
  form.append(gestureHelp);

  const node = <K extends keyof HTMLElementTagNameMap>(tag: K, className: string, text = '') => {
    const element = document.createElement(tag);
    element.className = className;
    element.textContent = text;
    return element;
  };

  function pairAnswer(): MissionLearningAnswer | undefined {
    const task = activity.task;
    if (task.kind !== 'pairing') return undefined;
    if (
      assignments.size !== task.recipients.length ||
      task.recipients.some((row) => !assignments.has(row.id))
    )
      return undefined;
    return {
      kind: 'pairing',
      assignments: task.recipients.map((row) => ({
        objectId: assignments.get(row.id)!,
        recipientId: row.id,
      })),
    };
  }

  function selectedAnswer(): MissionLearningAnswer | undefined {
    if (modern) return questionControl?.answer();
    if (activity.task.kind === 'pairing') return pairAnswer();
    return choiceId ? { kind: 'choice', choiceId } : undefined;
  }

  function update() {
    if (destroyed) return;
    const answer = selectedAnswer();
    const pending = callbacks.hasPending();
    check.disabled = busy || recording || complete || pending || !answer;
    hint.disabled =
      busy || recording || complete || pending || currentActivity.support.hintCount >= 20;
    example.disabled =
      busy || recording || complete || pending || currentActivity.support.workedExample;
    abandon.disabled = busy || recording || complete || pending;
    together.disabled = busy || recording || complete || pending;
    retry.hidden = !retryAvailable;
    retry.disabled = busy || !retryAvailable;
    refresh.hidden = !refreshAvailable;
    refresh.disabled = busy || !refreshAvailable;
    inputs.forEach((input) => (input.disabled = busy || recording || complete || pending));
    questionControl?.update();
    voice?.update();
    if (busy || complete || pending) drag.cancel();
    root.setAttribute('aria-busy', String(busy));
  }

  function renderChoices(choices: MissionChoice[]) {
    gestureHelp.textContent =
      activity.task.kind === 'count_collection'
        ? 'Count the objects, then drag or tap an answer card.'
        : 'Drag an answer card into the tray, or tap a card.';
    const fieldset = node('fieldset', 'ww-mission-options');
    const legend = document.createElement('legend');
    legend.textContent = 'Pick an answer card';
    const tray = node('div', 'ww-mission-answer-tray', 'Drop your answer here, or tap a card');
    tray.setAttribute('role', 'group');
    tray.setAttribute('aria-label', 'Your answer');
    const options = new Map<string, HTMLInputElement>();
    const labels = choices.map((c) => ({ id: c.id, label: c.label ?? String(c.value) }));
    const choose = (id: string) => {
      if (!editable() || recording) return;
      const input = options.get(id);
      const label = labels.find((c) => c.id === id);
      if (!input || !label) return;
      choiceId = id;
      input.checked = true;
      tray.textContent = `Your idea: ${label.label}`;
      update();
    };
    fieldset.append(legend);
    for (const choice of choices) {
      const label = node('label', 'ww-mission-choice');
      const input = document.createElement('input');
      input.type = 'radio';
      input.name = `mission-choice-${activity.id}`;
      input.value = choice.id;
      input.addEventListener('change', () => choose(choice.id));
      options.set(choice.id, input);
      inputs.push(input);
      const text = node(
        'span',
        !choice.label || /^\d+$/.test(choice.label) ? 'ww-mission-sr-only' : '',
        choice.label ?? String(choice.value),
      );
      if (choice.value !== undefined || /^\d+$/.test(choice.label ?? '')) {
        label.append(input, picture(document, String(choice.value ?? choice.label)), text);
      } else label.append(input, text);
      drag.source(label, choice.id);
      fieldset.append(label);
    }
    drag.target(tray, choose);
    form.append(fieldset, tray);
    // Phonics/spelling and decoding must not use transcription as a substitute for the skill.
    const noVoice = new Set([
      'E03',
      'E04',
      'E05',
      'E06',
      'E07',
      'E08',
      'E09',
      'E10',
      'EY101',
      'EY102',
      'EY106',
      'EY201',
      'EY204',
      'EY303',
      'EY403',
    ]);
    if (callbacks.recorder && !noVoice.has(activity.skillId)) {
      voice = voiceEntry(form, callbacks.recorder, labels, editable, choose, (active) => {
        recording = active;
        if (active) drag.cancel();
        update();
      });
    }
  }

  const task = activity.task;
  if (task.kind === 'question') {
    gestureHelp.hidden = true;
    const controlRoot = document.createElement('div');
    form.append(controlRoot);
    questionControl = mountQuestionControl(
      controlRoot,
      task.interaction,
      task.stimuli,
      () => editable() && !recording,
      () => queueMicrotask(update),
    );
    if (task.interaction.kind === 'workshop') {
      check.hidden = true;
      together.replaceChildren(
        activityIcon('together'),
        document.createTextNode('Finish activity'),
      );
    }
    if (
      callbacks.recorder &&
      ((task.interaction.kind === 'number_entry' && task.interaction.fields.length === 1) ||
        (task.interaction.kind === 'single_choice' &&
          ![
            'E03',
            'E04',
            'E05',
            'E06',
            'E07',
            'E08',
            'E09',
            'E10',
            'E12',
            'EY101',
            'EY102',
            'EY106',
            'EY201',
            'EY204',
            'EY303',
            'EY403',
          ].includes(activity.skillId)) ||
        (task.interaction.kind === 'workshop' &&
          !['E01', 'E03', 'E04', 'E05', 'E06', 'E11', 'EY101', 'EY107', 'EY201', 'EY605'].includes(
            activity.skillId,
          )))
    ) {
      voice = voiceEntry(
        form,
        callbacks.recorder,
        task.interaction.kind === 'single_choice' ? task.interaction.options : [],
        editable,
        (id) => questionControl?.selectChoice(id),
        (active) => {
          recording = active;
          update();
        },
        task.interaction.kind === 'single_choice'
          ? undefined
          : (text) => questionControl?.useText(text),
      );
    }
  } else if (task.kind === 'pairing') {
    if (task.objects.length !== task.recipients.length) {
      status.textContent = 'This pairing task is not available right now.';
      check.disabled = true;
    } else {
      const table = node(
        'div',
        `ww-mission-pairing ${task.layout === 'recipient_slots' ? 'ww-mission-recipient-slots' : 'ww-mission-delivery-paths'}`,
      );
      const rows = task.recipients;
      const pieces = task.objects;
      const bank = node('div', 'ww-mission-piece-bank');
      bank.setAttribute('role', 'group');
      bank.setAttribute('aria-label', 'Pieces to match');
      let selected: string | undefined;
      const pieceButtons = new Map<string, HTMLButtonElement>();
      const slots = new Map<string, HTMLButtonElement>();
      const refreshPairs = () => {
        for (const [id, button] of pieceButtons) {
          button.setAttribute('aria-pressed', String(selected === id));
          const used = [...assignments.values()].includes(id);
          button.classList.toggle('is-placed', used);
        }
        for (const row of rows) {
          const piece = pieces.find((p) => p.id === assignments.get(row.id));
          const button = slots.get(row.id)!;
          button.textContent = `${row.label}: ${piece?.label ?? 'tap here'}`;
          button.setAttribute('aria-label', `${row.label}: ${piece?.label ?? 'empty place'}`);
        }
        update();
      };
      const place = (row: string, piece: string) => {
        if (!editable() || recording || !pieces.some((p) => p.id === piece)) return;
        // Moving a piece removes its old match. Every edit remains reversible.
        for (const [previous, value] of assignments)
          if (value === piece) assignments.delete(previous);
        assignments.set(row, piece);
        selected = undefined;
        status.textContent = `${assignments.size} of ${rows.length} places filled.`;
        refreshPairs();
      };
      for (const piece of pieces) {
        const button = node('button', 'ww-mission-piece', piece.label);
        button.type = 'button';
        const token = node(
          'span',
          'ww-mission-piece-token',
          (
            { acorn: '🌰', berry: '🍓', flower: '🌼', shell: '🐚', seed: '🌱' } as Record<
              string,
              string
            >
          )[piece.label.split(' ')[0]!] ?? '◆',
        );
        token.setAttribute('aria-hidden', 'true');
        button.prepend(token);
        button.setAttribute('aria-pressed', 'false');
        button.onclick = () => {
          if (!editable() || recording) return;
          selected = selected === piece.id ? undefined : piece.id;
          refreshPairs();
        };
        pieceButtons.set(piece.id, button);
        inputs.push(button);
        drag.source(button, piece.id);
        bank.append(button);
      }
      for (const row of rows) {
        const button = node('button', 'ww-mission-pair-row');
        button.type = 'button';
        button.onclick = () => {
          if (!editable() || recording) return;
          if (selected) place(row.id, selected);
          else {
            assignments.delete(row.id);
            refreshPairs();
          }
        };
        slots.set(row.id, button);
        inputs.push(button);
        drag.target(button, (piece) => place(row.id, piece));
        table.append(button);
      }
      refreshPairs();
      form.append(bank);
      form.append(table);
    }
  } else if (task.kind === 'practice_cards') {
    const stimulus = node('div', 'ww-mission-stimulus', task.stimulus);
    form.append(stimulus);
    if (task.adultReview) {
      gestureHelp.textContent =
        'Make, write or talk with a grown-up. There is more than one good answer.';
      check.hidden = true;
      const notice = node(
        'p',
        'ww-mission-review-note',
        'A grown-up helps review this activity. Choose Let’s do it together to continue with supported practice.',
      );
      const draftLabel = node(
        'label',
        'ww-mission-draft-label',
        'Your ideas (draw or talk with a grown-up, or write here)',
      );
      const draft = document.createElement('textarea');
      draft.rows = 4;
      draft.maxLength = 4000;
      draft.placeholder = 'Start your idea here…';
      inputs.push(draft);
      draftLabel.append(draft);
      const note = node(
        'p',
        'ww-mission-review-note',
        'This draft stays on this screen until you close it. Review it with a grown-up before continuing.',
      );
      form.append(notice, draftLabel, note);
      if (
        callbacks.recorder &&
        !['E04', 'E05', 'E06', 'EY101', 'EY201'].includes(activity.skillId)
      ) {
        voice = voiceEntry(
          form,
          callbacks.recorder,
          [],
          editable,
          () => {},
          (active) => {
            recording = active;
            update();
          },
          (text) => {
            draft.value = text;
          },
        );
      }
    } else renderChoices(task.choices);
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
      const pieces = new Map<string, HTMLButtonElement>();
      const movePiece = (id: string, intoCounted: boolean) => {
        if (!editable() || recording) return;
        const piece = pieces.get(id);
        if (!piece) return;
        if (intoCounted) placed.add(id);
        else placed.delete(id);
        piece.setAttribute('aria-pressed', String(intoCounted));
        (intoCounted ? counted : source).append(piece);
      };
      drag.target(counted, (id) => movePiece(id, true));
      drag.target(source, (id) => movePiece(id, false));
      for (const item of task.objects) {
        const piece = document.createElement('button');
        piece.type = 'button';
        piece.className = 'ww-mission-movable-object';
        piece.textContent = item.label;
        piece.setAttribute('aria-pressed', 'false');
        inputs.push(piece);
        piece.addEventListener('click', () => movePiece(item.id, !placed.has(item.id)));
        pieces.set(item.id, piece);
        drag.source(piece, item.id);
        source.append(piece);
      }
      source.setAttribute('data-caption', 'Ready to count');
      counted.setAttribute('data-caption', 'Counted');
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

  function finish() {
    if (destroyed || complete) return;
    complete = true;
    root.hidden = true;
    drag.cancel();
    queueMicrotask(() => {
      if (!destroyed) callbacks.onContinue();
    });
  }

  function showSnapshot(snapshot: MissionLearningSnapshot) {
    if (destroyed) return;
    if (
      snapshot.activeActivity?.id !== activity.id &&
      snapshot.skills.some((skill) => skill.skillId === activity.skillId && skill.activityCompleted)
    ) {
      finish();
      return;
    }
    if (
      snapshot.activeActivity &&
      (snapshot.activeActivity.id !== activity.id ||
        snapshot.activeActivity.variantId !== activity.variantId)
    ) {
      if (modern && snapshot.lastFeedback?.result === 'incorrect') {
        status.textContent = snapshot.lastFeedback.explanation ?? snapshot.lastFeedback.message;
        form.hidden = true;
        actions.hidden = true;
        const next = document.createElement('button');
        next.type = 'button';
        next.className = 'primary';
        next.textContent = 'Try another';
        next.addEventListener('click', () => {
          if (destroyed) return;
          root.hidden = true;
          callbacks.onReplace?.(snapshot);
        });
        root.append(next);
        next.focus();
        complete = true;
        return;
      }
      complete = true;
      root.hidden = true;
      callbacks.onReplace?.(snapshot);
      return;
    }
    if (snapshot.activeActivity?.id === activity.id) currentActivity = snapshot.activeActivity;
    const feedback = snapshot.lastFeedback;
    if (!feedback)
      status.textContent = snapshot.activeActivity?.revealedSupport
        ? 'Here’s some help. Try when you are ready.'
        : 'Your idea is ready.';
    if (feedback) {
      status.textContent = feedback.message;
      if (feedback.completed && !snapshot.activeActivity && !complete) {
        finish();
      }
    }
    const revealed = snapshot.activeActivity?.revealedSupport;
    if (revealed) {
      supportText.textContent = revealed.text;
      supportText.hidden = false;
      hint.setAttribute('aria-expanded', 'true');
    }
    if (feedback) retryAvailable = false;
    update();
  }

  async function recover(error: unknown, operation?: () => Promise<MissionLearningSnapshot>) {
    if (destroyed) return;
    retryAvailable = callbacks.hasPending();
    if (retryAvailable) {
      // The outcome is uncertain: only retry the original prepared command.
      status.textContent = 'We could not confirm that save. The same action is ready to retry.';
      return;
    }
    if (modern && error instanceof ApiFailure && error.code === 'INVALID_INPUT') {
      status.textContent = 'Check the format of your answer, then try again.';
      return;
    }
    if (!(error instanceof ApiFailure) || error.code !== 'REVISION_CONFLICT') {
      refreshAvailable = true;
      status.textContent = 'This task could not reconnect. Refresh progress before checking again.';
      return;
    }
    status.textContent = 'Checking your saved progress…';
    try {
      // A definite rejection permits reconciliation; uncertain writes keep their exact command.
      const snapshot = await callbacks.onRefresh();
      if (destroyed) return;
      const completed = snapshot.skills.some(
        (skill) => skill.skillId === activity.skillId && skill.activityCompleted,
      );
      if (snapshot.activeActivity?.id !== activity.id) {
        if (completed || !snapshot.activeActivity) finish();
        else {
          root.hidden = true;
          complete = true;
          callbacks.onContinue();
        }
        return;
      }
      // Keep the existing controls and answer until the retried operation has
      // resolved. The session includes the frozen variant when sending answers.
      showSnapshot(operation ? await operation() : snapshot);
      refreshAvailable = false;
    } catch {
      refreshAvailable = true;
      status.textContent = 'Saved progress could not be loaded. Refresh progress to reconnect.';
    }
  }

  async function perform(operation: () => Promise<MissionLearningSnapshot>) {
    if (busy || destroyed || complete) return;
    busy = true;
    status.textContent = 'Saving your mission task…';
    update();
    try {
      showSnapshot(await operation());
    } catch (error) {
      await recover(error, operation);
    } finally {
      busy = false;
      update();
    }
  }

  form.addEventListener('submit', (event) => {
    event.preventDefault();
    const answer = selectedAnswer();
    if (modern && answer && !check.disabled) void perform(() => callbacks.onAttempt(answer));
  });
  check.addEventListener('click', () => {
    if (modern) return;
    const answer = selectedAnswer();
    if (!answer) return;
    void perform(() => callbacks.onAttempt(answer));
  });
  together.addEventListener('click', () => {
    if (callbacks.onFinishTogether) void perform(callbacks.onFinishTogether);
  });
  hint.setAttribute('aria-controls', 'ww-mission-support');
  supportText.id = 'ww-mission-support';
  hint.setAttribute('aria-expanded', String(!supportText.hidden));
  hint.addEventListener('click', () => {
    if (modern && !supportText.hidden) {
      supportText.hidden = true;
      hint.setAttribute('aria-expanded', 'false');
      return;
    }
    void perform(() => callbacks.onSupport('hint'));
  });
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
      .catch((error: unknown) => recover(error))
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
        if (snapshot.activeActivity) showSnapshot(snapshot);
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
          : 'This task could not be put aside. Refresh progress to reconnect.';
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

  update();

  return {
    destroy() {
      destroyed = true;
      questionControl?.destroy();
      voice?.destroy();
      drag.destroy();
      root.remove();
    },
    focus() {
      heading.focus({ preventScroll: true });
    },
  };
}
