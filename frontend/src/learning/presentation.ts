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
      prompt: `The bridge needs ${parameters.total} planks. ${given} are ready. Add the missing planks.`,
    };
  }
  if (encounter.templateId === 'equal_sharing' && 'recipients' in parameters) {
    return {
      prompt: `Share ${parameters.total} objects fairly between ${parameters.recipients} friends.`,
    };
  }
  const promptId =
    'instructionId' in parameters
      ? parameters.instructionId
      : 'promptId' in parameters
        ? parameters.promptId
        : null;
  if (!promptId || !Object.hasOwn(prompts, promptId)) return null;
  const prompt = prompts[promptId as keyof typeof prompts].instruction;
  const choiceMode =
    promptId === 'sort-round'
      ? 'multiple'
      : ['word-cat', 'word-sun', 'word-map', 'seed-order'].includes(promptId)
        ? 'ordered'
        : 'single';
  return { prompt, choiceMode };
}
