import {
  characterCard,
  charactersForRegion,
  libraryRegions,
  type LibraryRegion,
} from './character-library-model.ts';
import './character-art.css';
import './character-library.css';

const root = document.querySelector<HTMLElement>('#character-library')!;
root.innerHTML = `<header class="library-header"><a class="back-link" href="/">← Back to the adventure</a><p class="library-eyebrow">THE WONDERWILDS FIELD GUIDE</p><h1>A little world of friends.</h1><p>Meet twelve curious neighbours. Every friend has a favourite place, a helping idea and a story to share.</p></header><main><nav class="region-filters" aria-label="Choose a region">${libraryRegions.map((region) => `<button type="button" data-region="${region.id}" aria-pressed="${region.id === 'all'}">${region.name}</button>`).join('')}</nav><p id="friend-count" class="friend-count" role="status" aria-live="polite"></p><div class="friend-grid" id="friend-grid"></div><aside class="library-note"><strong>A growing field guide</strong><p>Every friendship begins with a little help. Explore Clover Meadow in the adventure; Peachy Orchard and Starlight Cove are future destinations.</p></aside></main><footer>Wonderwilds · Small acts of kindness. Wonderful discoveries.</footer>`;

function showRegion(region: LibraryRegion) {
  const friends = charactersForRegion(region);
  document.querySelector('#friend-grid')!.innerHTML = friends
    .map((friend) => characterCard(friend.id))
    .join('');
  document.querySelector('#friend-count')!.textContent = `${friends.length} friends to meet`;
  document.querySelectorAll<HTMLButtonElement>('[data-region]').forEach((button) => {
    button.setAttribute('aria-pressed', String(button.dataset.region === region));
  });
}
root.querySelectorAll<HTMLButtonElement>('[data-region]').forEach((button) => {
  button.onclick = () => showRegion(button.dataset.region as LibraryRegion);
});
showRegion('all');
