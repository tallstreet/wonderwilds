import { heroFrames, spriteMarkup } from './sprites.ts';

export const heroes: Record<
  string,
  { name: string; species: string; fur: string; cape: string; accent?: string; description: string }
> = {
  rose: {
    name: 'Rosie',
    species: 'kitten',
    fur: '#ffb9cf',
    cape: '#a251e5',
    description: 'A kind heart and a little sparkle.',
  },
  sky: {
    name: 'Skye',
    species: 'kitten',
    fur: '#ade9ed',
    cape: '#377fce',
    description: 'Always ready for a new discovery.',
  },
  gold: {
    name: 'Sunny',
    species: 'kitten',
    fur: '#ffc16c',
    cape: '#27b9b2',
    description: 'Bringing sunshine to every mission.',
  },
  poppy: {
    name: 'Poppy',
    species: 'bird',
    fur: '#ffc180',
    cape: '#31b9b4',
    accent: '#f58fb7',
    description: 'A cheerful songbird with bright ideas.',
  },
  wren: {
    name: 'Wren',
    species: 'bird',
    fur: '#9edeea',
    cape: '#8d64d2',
    accent: '#f7bc70',
    description: 'A curious little bird with big courage.',
  },
  clover: {
    name: 'Clover',
    species: 'bunny',
    fur: '#d9b9ef',
    cape: '#41b9aa',
    accent: '#e98cbe',
    description: 'A quick-thinking bunny who loves to help.',
  },
  mochi: {
    name: 'Mochi',
    species: 'bunny',
    fur: '#ffdfba',
    cape: '#dc81b4',
    accent: '#ac8adf',
    description: 'A gentle bunny who never gives up.',
  },
};

export function heroPortrait(id: string, size = 100) {
  return spriteMarkup(Math.max(0, heroFrames.indexOf(id)), 'hero', size);
}
