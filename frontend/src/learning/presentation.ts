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
    return {
      prompt: encounter.personalQuest
        ? `Add the missing objects to make ${parameters.total} altogether.`
        : 'Help Pip finish the bridge!',
    };
  }
  if (encounter.templateId === 'equal_sharing' && 'recipients' in parameters) {
    return {
      prompt: encounter.personalQuest
        ? `Share ${parameters.total} berries between ${parameters.recipients} friends. Give everyone the same!`
        : `Share ${parameters.total} berries. Give everyone the same!`,
    };
  }
  const promptId =
    'instructionId' in parameters
      ? parameters.instructionId
      : 'promptId' in parameters
        ? parameters.promptId
        : null;
  if (!promptId || !Object.hasOwn(prompts, promptId)) return null;
  const shortPrompts: Record<string, string> = {
    'count-next': 'Which number comes next?',
    'pattern-next': 'What comes next?',
    'sort-round': 'Find all the round things.',
    'place-beside': 'Put it beside the tree.',
    'longest-rope': 'Which rope is longest?',
    'sound-s': 'Which letter makes /s/?',
    'word-cat': 'Build the word for a pet that meows.',
    'word-sun': 'Build the word for our bright daytime star.',
    'word-map': 'Build the word for a drawing of places.',
    'seed-order': 'Help Pip grow a seed. What happens first?',
    'listen-direction': 'Put the cup beside the tree.',
    'read-red-cup': 'Pip needs a red cup. Can you find it?',
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
