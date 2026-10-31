import { art } from '../learning/visuals.ts';
import { mountReviewControl, type ControlKind } from './controls.ts';
import './style.css';

const examples: {
  kind: ControlKind;
  label: string;
  skill: string;
  title: string;
  prompt: string;
  emphasis?: string;
  hint: string;
}[] = [
  {
    kind: 'single',
    label: 'Choose one',
    skill: 'Counting · M05',
    title: 'One little step',
    prompt: 'What number is one more than 9?',
    emphasis: 'one more',
    hint: 'Start at 9 and count forward one step.',
  },
  {
    kind: 'multiple',
    label: 'Choose a few',
    skill: 'Shapes · M10',
    title: 'Shape detectives',
    prompt: 'Choose all the triangles.',
    emphasis: 'all',
    hint: 'A triangle has three straight sides. It can point in any direction.',
  },
  {
    kind: 'number',
    label: 'Type a number',
    skill: 'Adding · M07',
    title: 'A few more berries',
    prompt: 'What is 3 plus 4?',
    hint: 'Start with 4 and count on three more.',
  },
  {
    kind: 'text',
    label: 'Type a word',
    skill: 'Spelling · EY106',
    title: 'A word for this',
    prompt: 'Spell the word for this picture.',
    hint: 'Say each sound in the word, then write the letters.',
  },
  {
    kind: 'order',
    label: 'Put in order',
    skill: 'Daily sequences · M13',
    title: 'Watch it grow',
    prompt: 'Put the plant pictures in order.',
    emphasis: 'in order',
    hint: 'Think about what you plant first, and what grows from it.',
  },
  {
    kind: 'assign',
    label: 'Match and move',
    skill: 'One for each · M01',
    title: 'A gift for every friend',
    prompt: 'Give one gift to each friend.',
    emphasis: 'one gift',
    hint: 'Each friend needs one gift. Any gift can go to any friend.',
  },
  {
    kind: 'workshop',
    label: 'Make something',
    skill: 'Writing · E13',
    title: 'You’re invited, Pip!',
    prompt: 'Write an invitation to a picnic.',
    hint: 'You could start with “Dear Pip”. Tell Pip where to meet you.',
  },
];

const root = document.querySelector<HTMLElement>('#activity-review')!;
root.innerHTML = `<header class="review-header"><a class="brand" href="/">✳ Wonderwilds</a><span class="review-badge">Activity playground</span></header><main class="review-layout"><aside class="review-sidebar"><p class="eyebrow">A LITTLE LEARNING, A LOT OF WONDER</p><h1>Small questions. <br>Big discoveries.</h1><p class="review-intro">Try each way to play. Pick an example below.</p><nav aria-label="Activity examples"></nav><p class="preview-note">UI preview · Answers are collected here for review. No marking, progress or rewards are saved.</p></aside><div class="stage"><div class="stage-heading"><span id="sample-count"></span><button type="button" id="start-again" class="quiet-button">↶ Start again</button></div><div id="samples"></div><footer class="stage-footer"><span>Take your time. You’ve got this.</span><button type="button" id="next-example" class="next-button">Next example →</button></footer></div></main>`;
const nav = root.querySelector('nav')!;
const samples = root.querySelector('#samples')!;
let selected = 0;
const mounted: ReturnType<typeof build>[] = [];

function build(example: (typeof examples)[number], index: number) {
  const article = document.createElement('article');
  article.className = 'sample-card';
  article.dataset.control = example.kind;
  article.innerHTML = `<div class="sample-welcome"><div class="friend-portrait"></div><div><p class="eyebrow"></p><p class="friend-greeting">Pip has a little challenge for you.</p></div></div><h2 tabindex="-1"></h2><form novalidate><p class="question"></p><div class="sample-control"></div><div class="sample-feedback" role="status" aria-live="polite" aria-atomic="true"></div><p class="hint-content" hidden></p><div class="sample-actions"><button class="check-button" type="submit" disabled></button><button class="hint-button" type="button" aria-expanded="false">A little hint</button></div></form>`;
  article.querySelector('.friend-portrait')!.append(art(document, 'pip', 'hero'));
  article.querySelector('.eyebrow')!.textContent = example.skill;
  article.querySelector('h2')!.textContent = example.title;
  const question = article.querySelector('.question')!;
  if (example.emphasis) {
    const [before, after] = example.prompt.split(example.emphasis);
    const emphasis = document.createElement('strong');
    emphasis.textContent = example.emphasis;
    question.append(before!, emphasis, after!);
  } else question.textContent = example.prompt;
  const feedback = article.querySelector<HTMLElement>('.sample-feedback')!;
  const check = article.querySelector<HTMLButtonElement>('.check-button')!;
  check.textContent = example.kind === 'workshop' ? 'Ready to share' : 'Check answer';
  const control = mountReviewControl(
    article.querySelector('.sample-control')!,
    example.kind,
    () => {
      // Initial arrangement setup can call back before mount returns.
      queueMicrotask(() => {
        check.disabled = !control.answer();
        feedback.textContent = '';
      });
    },
  );
  article.querySelector('form')!.addEventListener('submit', (event) => {
    event.preventDefault();
    const answer = control.answer();
    if (!answer) return;
    feedback.textContent =
      example.kind === 'workshop'
        ? 'Your invitation is ready to read together. It has not been automatically marked.'
        : `Preview answer received: ${
            typeof answer === 'string'
              ? answer
              : Array.isArray(answer)
                ? answer.join(' → ')
                : Object.entries(answer)
                    .map(([gift, friend]) => `${gift} for ${friend}`)
                    .join(', ')
          }. This preview does not mark answers.`;
  });
  const hint = article.querySelector<HTMLButtonElement>('.hint-button')!;
  const hintText = article.querySelector<HTMLElement>('.hint-content')!;
  hintText.id = `hint-${index}`;
  hintText.textContent = example.hint;
  hint.setAttribute('aria-controls', hintText.id);
  hint.addEventListener('click', () => {
    hintText.hidden = !hintText.hidden;
    hint.setAttribute('aria-expanded', String(!hintText.hidden));
  });
  samples.append(article);
  return { article, control };
}

function show(index: number, focus = true) {
  selected = index;
  mounted.forEach(({ article, control }, position) => {
    control.cancel();
    article.hidden = position !== index;
  });
  nav
    .querySelectorAll('button')
    .forEach((button, position) => button.setAttribute('aria-current', String(position === index)));
  root.querySelector('#sample-count')!.textContent =
    `EXAMPLE ${String(index + 1).padStart(2, '0')} / 07`;
  const next = root.querySelector<HTMLButtonElement>('#next-example')!;
  next.textContent = index === examples.length - 1 ? 'Back to the first →' : 'Next example →';
  if (focus) mounted[index]!.article.querySelector('h2')!.focus();
}

examples.forEach((example, index) => {
  const button = document.createElement('button');
  button.type = 'button';
  const number = document.createElement('span');
  number.className = 'nav-number';
  number.textContent = String(index + 1).padStart(2, '0');
  button.append(number, example.label);
  button.addEventListener('click', () => show(index));
  nav.append(button);
  mounted.push(build(example, index));
});
root
  .querySelector('#next-example')!
  .addEventListener('click', () => show((selected + 1) % examples.length));
root.querySelector('#start-again')!.addEventListener('click', () => {
  mounted[selected]!.control.destroy();
  mounted[selected]!.article.remove();
  mounted[selected] = build(examples[selected]!, selected);
  show(selected);
});
window.addEventListener('pagehide', () => mounted.forEach(({ control }) => control.destroy()));
show(0, false);
