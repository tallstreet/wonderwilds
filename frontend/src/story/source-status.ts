import type { StorySnapshot } from '../../../contracts/generated/contracts.ts';

/** Describe the displayed prose; a successful cached scene is not a live health check. */
export function sceneSourceLabel(
  snapshot: Pick<StorySnapshot, 'sceneSource' | 'commandBudgetExhausted'>,
): string {
  if (snapshot.commandBudgetExhausted) return 'Story dialogue unavailable';
  switch (snapshot.sceneSource) {
    case 'llm':
      return 'LLM-generated dialogue';
    case 'authored_fallback':
      return 'Preauthored story · LLM fallback';
    case 'authored':
      return 'Preauthored story';
    default:
      return 'Story source unavailable · refresh to check';
  }
}
