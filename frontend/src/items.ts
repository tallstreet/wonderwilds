import { propArt } from './sprites.ts';

/** Cosmetic catalogue; legal pickups, recipes and quantities come from Rust. */
export const items: Record<string, { name: string; description: string; prop: string }> = {
  wood: {
    name: 'wood',
    description: 'A small bundle of smooth fallen wooden branches',
    prop: 'plank',
  },
  planks: {
    name: 'planks',
    description: 'Three neatly cut honey-coloured wooden planks',
    prop: 'plank',
  },
  berries: {
    name: 'berries',
    description: 'A little cluster of glossy ripe red berries with green leaves',
    prop: 'berry',
  },
  picnic_basket: {
    name: 'picnic basket',
    description: 'A woven picnic basket with red gingham cloth and berries',
    prop: 'picnic',
  },
  glow_pebbles: {
    name: 'glow pebbles',
    description: 'Three smooth turquoise pebbles glowing gently from within',
    prop: 'flower',
  },
  lanterns: {
    name: 'lanterns',
    description: 'A little wooden lantern holding a glowing turquoise pebble',
    prop: 'arch',
  },
  wind_instrument: {
    name: 'wind instrument',
    description: 'A handmade hollow wooden wind chime with little hanging leaves',
    prop: 'plank',
  },
  lucky_clover: {
    name: 'lucky clover',
    description: 'A vivid emerald four-leaf clover with a short curved stem',
    prop: 'flower',
  },
  ribbon: {
    name: 'golden ribbon',
    description: 'A golden satin ribbon tied in a soft bow',
    prop: 'flower',
  },
  river_crystal: {
    name: 'river crystal',
    description: 'A rounded faceted sky-blue river crystal',
    prop: 'flower',
  },
  seashell: {
    name: 'pearly seashell',
    description: 'A pearly peach and cream scallop seashell',
    prop: 'flower',
  },
  fern_leaf: { name: 'fern leaf', description: 'A fresh curled green fern frond', prop: 'flower' },
  moon_mushroom: {
    name: 'moon mushroom',
    description: 'A tiny whimsical pale blue mushroom with glowing moon spots',
    prop: 'flower',
  },
  feather: {
    name: 'owl feather',
    description: 'A soft cream and lavender owl feather with a golden tip',
    prop: 'flower',
  },
  acorn: {
    name: 'golden acorn',
    description: 'A plump golden-brown acorn with a textured cap and tiny leaf',
    prop: 'flower',
  },
};
const artwork = new Map<string, string>();
export function itemTexture(id: string) {
  return `item:${id}`;
}
export function itemImage(id: string) {
  return artwork.get(id);
}
export function rememberItemImage(id: string, url: string) {
  if (Object.hasOwn(items, id)) artwork.set(id, url);
}
export function itemArt(id: string, size = 64) {
  const url = artwork.get(id);
  return url
    ? `<img class="item-art" src="${url}" width="${size}" height="${size}" alt=""/>`
    : propArt(Object.hasOwn(items, id) ? items[id].prop : 'picnic', size);
}
