import type Phaser from 'phaser';
import type { Progress } from './types.ts';
import { maps, biomes } from './maps.ts';
import { mapImageUrl, createImageRequests } from './mapImages.ts';
import { paintLandscape } from './landscape.ts';
import { BRIDGE } from './mapGeometry.ts';
// The generated landscape and these props share one shaded 3D art direction.
// Rewards remain data-driven: code counts planks/berries and switches garden state.
export function paintScenery(
  scene: Phaser.Scene & {
    label(x: number, y: number, text: string, size?: number): Phaser.GameObjects.Text;
  },
  state: Progress,
) {
  if (state.region !== 'meadow') {
    paintExplorationMap(scene, state.region);
    return;
  }
  scene.add.image(0, 0, 'meadow-map').setOrigin(0).setDisplaySize(1120, 690).setDepth(-10);
  const fixed = state.done.includes('meadow-bridge');
  const prop = (name: string, x: number, y: number, w: number, h: number, depth = 2) =>
    scene.add.image(x, y, 'prop-atlas', name).setDisplaySize(w, h).setDepth(depth);
  const plankWidth = BRIDGE.width / 10;
  for (let i = 0; i < 10; i++)
    if (i < 6 || fixed)
      prop(
        'plank',
        BRIDGE.x - BRIDGE.width / 2 + (i + 0.5) * plankWidth,
        BRIDGE.y,
        plankWidth,
        62,
        2,
      );
  // Crop the same original art into rail layers: the far rail sits behind
  // crossing heroes, while the near rail correctly hides their lower legs.
  prop('bridge-frame', BRIDGE.x, BRIDGE.y, BRIDGE.width, BRIDGE.height, BRIDGE.deckTop - 1).setCrop(
    0,
    0,
    470,
    135,
  );
  prop(
    'bridge-frame',
    BRIDGE.x,
    BRIDGE.y,
    BRIDGE.width,
    BRIDGE.height,
    BRIDGE.y + BRIDGE.height / 2,
  ).setCrop(0, 135, 470, 135);
  prop(
    state.done.includes('meadow-garden') ? 'garden-blooming' : 'garden-sleeping',
    879,
    193,
    134,
    100,
    3,
  );
  prop('picnic', 359, 272, 136, 100, 3);
  if (state.done.includes('meadow-picnic'))
    for (const [cx, cy] of [
      [334, 293],
      [385, 280],
    ])
      for (let i = 0; i < 3; i++) prop('berry', cx - 7 + i * 7, cy + (i % 2) * 5, 8, 8, 4);
  prop('arch', 1030, 350, 105, 105, 3);
  scene.label(184, 235, 'HERO HIDEOUT', 11);
  scene.label(359, 212, 'BERRY PICNIC', 11);
  scene.label(879, 120, 'WISHING GARDEN', 11);
  scene.label(1030, 418, 'Friendship arch', 13);
}

// Each scene has a single request cache; only visited destinations are downloaded.
const requests = new WeakMap<Phaser.Scene, ReturnType<typeof createImageRequests>>();
function paintExplorationMap(scene: Phaser.Scene, region: string) {
  const key = `${region}-map`;
  const addBackground = () =>
    scene.add.image(0, 0, key).setOrigin(0).setDisplaySize(1120, 690).setDepth(-10);
  if (scene.textures.exists(key)) {
    addBackground();
    return;
  }
  const url = mapImageUrl(region);
  if (!url || maps[region]?.biome === undefined) return;
  const before = new Set(scene.children.list);
  paintLandscape(scene, maps[region], biomes[maps[region].biome!], []);
  const fallback = scene.children.list.filter((child) => !before.has(child));
  // This sentinel is destroyed on travel. A late response may cache the texture,
  // but must never paint over another destination or reset the moving player.
  const sentinel = scene.add.zone(0, 0, 1, 1);
  let request = requests.get(scene);
  if (!request) {
    request = createImageRequests(
      (textureKey, imageUrl) =>
        new Promise<void>((resolve, reject) => {
          const image = new Image();
          image.onload = () => {
            if (!scene.sys.isActive()) {
              reject(new Error('Scene stopped'));
              return;
            }
            if (!scene.textures.exists(textureKey)) scene.textures.addImage(textureKey, image);
            resolve();
          };
          image.onerror = () => reject(new Error('Landscape unavailable'));
          image.src = imageUrl;
        }),
    );
    requests.set(scene, request);
  }
  void request(key, url)
    .then(() => {
      if (!sentinel.active) return;
      fallback.forEach((child) => child.destroy());
      addBackground();
      sentinel.destroy();
    })
    .catch(() => {
      // Keep the fully playable fallback; visiting again retries the artwork.
      if (sentinel.active) sentinel.destroy();
    });
}
