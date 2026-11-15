type Move = (label: string, movement: string, disabled?: boolean) => string;

export function projectBoard(id: string, flags: string[], piece: string, move: Move) {
  const has = (part: string) => flags.includes(`project:${id}:${part}`);
  if (id === 'trial_bridge') {
    const ready = ['near', 'middle', 'far', 'brace_near', 'brace_far'].every(has);
    return `<section class="campaign-project" aria-label="Bridge construction"><p>Match the planks to the gaps, add a brace at each bank, then help the traveller cross. You can try pieces without a time limit.</p><div class="campaign-piece-tray" aria-label="Choose a plank">${['short', 'long'].map((size) => `<button data-project-piece="${size}" aria-pressed="${piece === size}"><span class="campaign-plank campaign-plank-${size}" aria-hidden="true"></span>${size === 'short' ? 'Short plank' : 'Long plank'}</button>`).join('')}</div><p>Selected: <strong>${piece === 'short' ? 'short plank' : 'long plank'}</strong>. Tap a gap to place it.</p><div class="bridge-river"><div class="bridge-bank">Near bank</div><div class="bridge-deck">${['near', 'middle', 'far'].map((gap) => `<div class="bridge-gap ${gap === 'middle' ? 'wide' : ''} ${has(gap) ? 'fitted' : ''}">${move(has(gap) ? `${gap} plank fitted ✓` : `${gap} ${gap === 'middle' ? 'long' : 'short'} gap`, `bridge_fit:${gap}:${piece}`, has(gap))}</div>`).join('')}</div><div class="bridge-bank">Far bank</div></div><div class="campaign-actions">${move(has('brace_near') ? 'Near brace secured ✓' : 'Secure near-bank brace', 'bridge_brace:near', !has('near') || has('brace_near'))}${move(has('brace_far') ? 'Far brace secured ✓' : 'Secure far-bank brace', 'bridge_brace:far', !has('far') || has('brace_far'))}</div><p>${ready ? 'All three planks fit and both banks are braced. The traveller can cross!' : 'The long middle gap needs a long plank. Fit the end planks before adding their braces.'}</p><div class="campaign-actions">${move('Help the traveller cross', 'bridge_cross', !ready)}</div></section>`;
  }
  if (id === 'trial_bloom') {
    const ready = [
      'north:beds',
      'south:beds',
      'watered:north',
      'watered:south',
      'mulched:north',
      'mulched:south',
    ].every(has);
    return `<section class="campaign-project" aria-label="Garden irrigation"><p>Send water to both beds, open each watering gate, then cover the damp soil with mulch. Watch where the water goes. There is no timer.</p><div class="garden-source">Water source 💧</div><div class="garden-beds">${['north', 'south'].map((bed) => `<article class="garden-bed ${has(`watered:${bed}`) ? 'watered' : ''}"><h3>${bed === 'north' ? 'North' : 'South'} seedlings</h3><div class="garden-channel ${has(`${bed}:beds`) ? 'flowing' : ''}" aria-hidden="true"></div><p class="garden-growth">${has(`mulched:${bed}`) ? '🌻 🌷 🌻' : has(`watered:${bed}`) ? '🌱 🌱 🌱' : '· 🥀 ·'}</p><p>${has(`mulched:${bed}`) ? 'Watered and protected' : has(`watered:${bed}`) ? 'Damp soil — add mulch' : 'Dry soil — needs water'}</p><div class="campaign-actions">${move(has(`${bed}:beds`) ? 'Water routed to bed ✓' : 'Route water to this bed', `garden_route:${bed}:beds`, has(`${bed}:beds`))}${move(has(`${bed}:collector`) ? 'Water going to collector' : 'Route water to collector', `garden_route:${bed}:collector`, has(`${bed}:collector`))}${move(has(`watered:${bed}`) ? 'Bed watered ✓' : 'Open watering gate', `garden_water:${bed}`, !has(`${bed}:beds`) || has(`watered:${bed}`))}${move(has(`mulched:${bed}`) ? 'Soil protected ✓' : 'Spread leaf mulch', `garden_mulch:${bed}`, !has(`watered:${bed}`) || has(`mulched:${bed}`))}</div></article>`).join('')}</div><p>The collector takes water away from the plants. Keep both channels flowing to the beds.</p><div class="campaign-actions">${move('Invite Fern to check the garden', 'garden_check', !ready)}</div></section>`;
  }
  return undefined;
}
