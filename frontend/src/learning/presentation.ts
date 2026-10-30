import type { EncounterInstance } from '../../../contracts/generated/contracts.ts';
import prompts from '../../../content/activities/prompts-v1.json';
import type { ActivityPresentation } from './activity.ts';

/** Resolve only approved public prompts. No answer keys or server-only fixtures are imported. */
export function authoredPresentation(encounter: EncounterInstance): ActivityPresentation | null {
  const parameters = encounter.parameters;
  if (
    encounter.templateId === 'missing_quantity' &&
    'total' in parameters &&
    !('recipients' in parameters)
  ) {
    const given = parameters.given ?? 0;
    return {
      prompt: `The bridge needs ${parameters.total} planks. ${given === 1 ? '1 plank is' : `${given} planks are`} already in place. Tap planks to add the ones that are missing.`,
    };
  }
  if (encounter.templateId === 'equal_sharing' && 'recipients' in parameters) {
    return {
      prompt: `Share ${parameters.total} berries between ${parameters.recipients} friends. Everyone gets the same number!`,
    };
  }
  const promptId =
    'instructionId' in parameters
      ? parameters.instructionId
      : 'promptId' in parameters
        ? parameters.promptId
        : null;
  if (!promptId || !Object.hasOwn(prompts, promptId)) return null;
  const shown =
    'objects' in parameters
      ? parameters.objects.filter((o) => 'attribute' in o && o.attribute === 'shown')
      : [];
  const shortPrompts: Record<string, string> = {
    'count-next': shown.length
      ? `Count along: ${shown.map((o) => o.label).join(', ')}. Which number comes next?`
      : 'Which number comes next?',
    'pattern-next': 'Look at the pattern. Which picture comes next?',
    'sort-round': 'Tap every round shape.',
    'place-beside': 'Which picture shows the cup beside the tree?',
    'longest-rope': 'Which rope is the longest? Tap it.',
    'sound-s': 'Which letter makes the /s/ sound? Tap it.',
    'word-cat': 'Tap the letters in order to spell the word for a pet that meows.',
    'word-sun': 'Tap the letters in order to spell the word for our bright daytime star.',
    'word-map': 'Tap the letters in order to spell the word for a drawing of places.',
    'seed-order': 'Help Pip grow a seed. Tap the cards in order: first, next, last.',
    'listen-direction': 'Pip says: “Put the cup beside the tree.” Tap the matching picture.',
    'read-red-cup': 'Pip needs a red cup. Tap the message that says so.',
  };
  const prompt = shortPrompts[promptId] ?? prompts[promptId as keyof typeof prompts].instruction;
  const choiceMode =
    promptId === 'sort-round'
      ? 'multiple'
      : ['word-cat', 'word-sun', 'word-map', 'seed-order'].includes(promptId)
        ? 'ordered'
        : 'single';
  return { prompt, choiceMode };
}
