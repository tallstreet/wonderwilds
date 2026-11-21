/** Story presentation follows confirmed campaign state; it never grants progress. */
const challenges: Record<
  string,
  { title: string; goal: string; progress: string; outcome?: string }
> = {
  practice_pip_board: {
    title: 'Find the cart’s trail with Pip',
    progress: 'Trail revealed',
    goal: 'Practise Pip’s revealing light on the chalk marks. We will need a sharp eye to follow the brass cart.',
  },
  practice_tansy_board: {
    title: 'Make room for a seedling',
    progress: 'Seedling supported',
    goal: 'Help Tansy weave the loose stems around a practice hoop without crowding the shoot. Growing things will need our help along the trail.',
  },
  practice_nib_board: {
    title: 'Steady Nib’s wobbly stool',
    progress: 'Stool steadied',
    goal: 'Help Nib place a practice brace. We will need a steady crossing before we can follow the cart.',
  },
  trial_bridge: {
    title: 'Help the last traveller cross',
    progress: 'Crossing secured',
    goal: 'The woodcutter has prepared the planks. Fit them across the gaps and brace both banks before the traveller crosses.',
    outcome:
      'The traveller is safely across! Light the bridge beacon with Hazel. The brass cart went towards the garden; that is where our investigation leads next.',
  },
  trial_bloom: {
    title: 'Shelter the thirsty seedlings',
    progress: 'Seedlings sheltered',
    goal: 'You have reconnected the water channels. Protect the seedlings from the collector’s draught so they can drink.',
    outcome:
      'Water reaches the sheltered seedlings. Finish with Fern, then follow the strange echoes towards the watchtower to learn why the collectors are still running.',
  },
  trial_echo: {
    title: 'Keep the warning shutters open',
    progress: 'Shutters secured',
    goal: 'You have pieced together the chime warning. Hold the tower shutters open so the message can travel.',
    outcome:
      'The warning can be heard again. The collectors were supposed to stop after the storm. Follow the orchard road: the mill and river may show why they did not.',
  },
  trial_harvest: {
    title: 'Turn the mill’s machine around',
    progress: 'Machine redirected',
    goal: 'The missing gear is back. Redirect the mill automaton so supplies reach the neighbours instead of the reserve.',
    outcome:
      'The mill can deliver supplies again. The ledger connects the collections to Vell. The river investigation is the other piece of this mystery.',
  },
  trial_current: {
    title: 'Secure the river landing',
    progress: 'Landing secured',
    goal: 'Rowan’s partner is safe and the bypass is open. Steady the landing so friends can cross the river together.',
    outcome:
      'The landing is safe. Rowan has seen why one exhausted partner cannot do everything. Once the mill and river are both restored, the route to the cove opens.',
  },
  trial_tide: {
    title: 'Protect the animals’ refuge',
    progress: 'Refuge secured',
    goal: 'The tracks and pipes told a different story about the mysterious glow. Stabilise the refuge for the animals sheltering here.',
    outcome:
      'The animals have a safe refuge. The glow was helping them, not harming them. Next, help Sail recover the lighthouse beam so we can reach the Observatory.',
  },
  trial_beacon: {
    title: 'Guide the lighthouse beam',
    progress: 'Lens aligned',
    goal: 'The missing lens has been recovered. Keep it steady and align the beam with the route to the Observatory.',
    outcome:
      'The beam points towards the Observatory. The old beacon map offers another way to share the light. Take that plan and your evidence to Vell.',
  },
  trial_accord: {
    title: 'Return the stored moonlight',
    progress: 'Light returned',
    goal: 'Your evidence and the reconnected beacon lines are ready. Balance the intake, shutters and return flow to share the stored moonlight.',
    outcome:
      'Moonlight flows back to the regions. Finish the repairs with Vell, then return to your friends for the Moonlight Gathering.',
  },
  rowan_first_board: {
    title: 'Try Rowan’s challenge',
    progress: 'Teamwork shown',
    goal: 'Rowan wants to prove that one strong partner is enough. Show what a careful plan can do.',
    outcome:
      'The contest is over. Keep following the clues; Rowan still has more to learn about sharing the work.',
  },
  rowan_shared_board: {
    title: 'Practise sharing the work',
    progress: 'Teamwork shown',
    goal: 'Rowan is learning to work with two friends. Take turns helping and prepare a stronger move for a partner.',
    outcome:
      'You have practised working together. Continue investigating the collectors with your friends.',
  },
  rowan_final_board: {
    title: 'Rowan’s teamwork relay',
    progress: 'Relay progress',
    goal: 'Work through three rounds with different affinities. Two friends must contribute their signatures to finish the relay.',
    outcome:
      'Both friends helped finish the relay. Rowan’s new plan gives everyone a part to play.',
  },
};
/** Welcome practices can be tried before or after packing, and again after leaving the bank. */
const welcomePractice: Record<string, { lead: string; next: string }> = {
  practice_pip_board: {
    lead: 'You and Pip can spot a hidden trail.',
    next: 'investigate the real marks at the bridge',
  },
  practice_tansy_board: {
    lead: 'The little shoot has room to grow.',
    next: 'follow the cart’s trail',
  },
  practice_nib_board: {
    lead: 'The stool is steady.',
    next: 'put your teamwork to use at the broken bridge',
  },
};
function practiceOutcome(id: string, flags: string[]) {
  const practice = welcomePractice[id];
  if (!practice) return undefined;
  if (flags.includes('welcome_complete'))
    return `${practice.lead} Extra practice complete. Your adventure continues where you left it.`;
  if (flags.includes('welcome_supplies_ready'))
    return `${practice.lead} Your bag is already packed with Hazel, so ${practice.next}. Trying the other friends is optional.`;
  return `${practice.lead} Pack with Hazel, then ${practice.next}. Trying the other friends is optional.`;
}
export function challengeStory(id: string, flags: string[] = []) {
  const base = challenges[id.replace(/^postgame_/, 'trial_')];
  if (base && id.startsWith('postgame_'))
    return {
      ...base,
      title: 'Revisit: ' + base.title,
      goal: 'An optional return challenge. Try a different team or plan. ' + base.goal,
      outcome: 'Return challenge complete. Your original story discoveries remain saved.',
    };
  const fallback = 'Challenge complete. Check your mission for the next step.';
  if (base) return { ...base, outcome: practiceOutcome(id, flags) ?? base.outcome ?? fallback };
  return {
    title: 'A challenge together',
    goal: 'Choose a friend and work together to finish this challenge.',
    progress: 'Progress',
    outcome: fallback,
  };
}
const discoveries = [
  [
    'welcome_complete',
    'A brass cart left the dark bridge. Before following it, we need to find out what happened to the crossing.',
  ],
  [
    'bridge_clue_complete',
    'The storm broke boards, but something carefully removed the moonstones. Record both clues, then build a safe crossing before following the cart.',
  ],
  [
    'bridge_project_complete',
    'The new crossing is built. A traveller still needs to get across: keep the braces steady together.',
  ],
  [
    'bridge_seal',
    'You made the crossing safe. The missing moonstones and brass filings point towards the cart’s route through the garden.',
  ],
  [
    'bloom_clue_complete',
    'The garden beds are not getting the same water. Follow the channels and choose a route that reaches the thirsty plants.',
  ],
  [
    'bloom_project_complete',
    'Your garden plan is ready. The collector’s draught still threatens the seedlings; shelter them while the water returns.',
  ],
  [
    'bloom_seal',
    'The garden has water again, but the collector is still taking supplies for an “emergency reserve”. The watchtower’s warning may explain that order.',
  ],
  [
    'echo_clue_complete',
    'The echoes carry pieces of a warning. Reconstruct its order with the watchtower chimes to hear the message.',
  ],
  [
    'echo_project_complete',
    'The chimes are in order. Now hold the shutters open so the warning can carry beyond the tower.',
  ],
  [
    'echo_seal',
    'The watchtower warning says the collectors should have stopped after the storm. Investigate the stopped mill and the reversed river to follow their work.',
  ],
  [
    'harvest_clue_complete',
    'The missing deliveries lead to the mill. Recover its missing gear so supplies can reach the neighbours again.',
  ],
  [
    'harvest_project_complete',
    'The gear fits. Now redirect the mill automaton: a repaired machine still needs to send supplies to the right place.',
  ],
  [
    'harvest_seal',
    'The mill’s ledger points to Vell’s order. Helping the neighbours also gives us evidence to take to the Observatory.',
  ],
  [
    'current_clue_complete',
    'The river’s reversed flow has stranded Rowan and his tired partner. Help them, then work out the pump’s bypass route.',
  ],
  [
    'current_project_complete',
    'The bypass valves are set. Secure the landing before anyone uses the reopened river route.',
  ],
  [
    'current_seal',
    'You rescued Rowan’s tired partner and made the river safe. Sharing the work matters. Restoring both the river and mill opens the way to the cove.',
  ],
  [
    'tide_clue_complete',
    'The tracks and pipes do not support the rumours about the glow. Build shelter for the displaced animals while you investigate.',
  ],
  [
    'tide_project_complete',
    'The shelter is built. Stabilise the refuge so the animals can safely stay here.',
  ],
  [
    'tide_seal',
    'The mysterious glow was protecting displaced animals. The collectors’ pipes caused the trouble. Now we need the lighthouse to reach the source.',
  ],
  [
    'beacon_clue_complete',
    'The old map connects the local beacons. Recover the missing lighthouse lens to reveal the route to the Observatory.',
  ],
  [
    'beacon_project_complete',
    'The lens is fitted. Align and steady it so the beam can guide the way to Vell.',
  ],
  [
    'beacon_seal',
    'The lighthouse beam and old beacon map show the way forward: share the load across the regions instead of keeping everything in one reserve.',
  ],
  [
    'accord_clue_complete',
    'Keeping everything in one reserve has left the habitats without what they need. Connect the beacon lines to share the load.',
  ],
  [
    'accord_project_complete',
    'All eight lines are connected. Balance the central machine so the stored light can return to the regions.',
  ],
  [
    'accord_seal',
    'Vell’s machines can return light instead of taking it. The neighbours can come together for the Moonlight Gathering.',
  ],
  [
    'finale_complete',
    'The beacons shine together, and your friends have gathered. The new constellation points towards more discoveries.',
  ],
] as const;
export function storySoFar(flags: string[]) {
  return discoveries.filter(([flag]) => flags.includes(flag)).at(-1)?.[1];
}
