import type Phaser from 'phaser';
import type { MapDefinition } from './maps.ts';
interface Palette {
  ground: number;
  canopy: number;
  accent: number;
  feature: string;
}
/** Generate only the current landscape, keeping the large library lightweight. */
export function paintLandscape(
  scene: Phaser.Scene,
  map: MapDefinition,
  palette: Palette,
  obstacles: { x: number; y: number; radius: number }[],
) {
  const variant = map.variant ?? 0;
  let seed = (map.biome! + 1) * 971 + variant * 137;
  const random = () => {
    seed = (seed * 1664525 + 1013904223) >>> 0;
    return seed / 4294967296;
  };
  const g = scene.add.graphics().setDepth(-10);
  g.fillStyle(palette.ground);
  g.fillRect(0, 0, 1120, 690);
  for (let i = 0; i < 100; i++) {
    g.fillStyle(palette.canopy, 0.1);
    g.fillEllipse(random() * 1120, random() * 690, 30 + random() * 90, 15 + random() * 40);
  }
  const points = [
    { x: 0, y: map.pathY.west },
    { x: 440 + variant * 44, y: 350 + variant * 6 },
    { x: 1120, y: map.pathY.east },
  ];
  g.lineStyle(68, 0xffffff, 0.25);
  g.strokePoints(points);
  g.lineStyle(52, 0xf2dfbc, 0.95);
  g.strokePoints(points);
  const feature = (x: number, y: number, scale: number) => {
    const a = scene.add.graphics().setPosition(x, y).setScale(scale).setDepth(y);
    a.fillStyle(0x304850, 0.18);
    a.fillEllipse(0, 2, 70, 17);
    a.fillStyle(palette.canopy);
    switch (palette.feature) {
      case 'crystal':
      case 'rock':
        a.fillTriangle(-30, 0, -6, -90, 24, 0);
        a.fillStyle(palette.accent);
        a.fillTriangle(0, 0, 24, -60, 40, 0);
        a.lineStyle(2, 0xffffff, 0.6);
        a.lineBetween(-6, -80, 0, -8);
        break;
      case 'mushroom':
        a.fillStyle(0xffedcf);
        a.fillRoundedRect(-9, -48, 18, 48, 5);
        a.fillStyle(palette.accent);
        a.fillEllipse(0, -51, 84, 45);
        a.fillStyle(0xffffff);
        a.fillCircle(-17, -55, 6);
        a.fillCircle(14, -61, 5);
        break;
      case 'cloud':
        a.fillStyle(0xffffff, 0.9);
        a.fillEllipse(0, -30, 95, 42);
        a.fillCircle(-16, -49, 25);
        a.fillCircle(17, -53, 30);
        break;
      case 'cactus':
        a.fillRoundedRect(-12, -85, 24, 85, 12);
        a.fillRoundedRect(-38, -60, 28, 17, 8);
        a.fillRoundedRect(-38, -80, 14, 34, 7);
        a.fillRoundedRect(9, -43, 30, 17, 8);
        a.fillRoundedRect(26, -68, 14, 38, 7);
        a.fillStyle(palette.accent);
        a.fillCircle(0, -88, 9);
        break;
      case 'flower':
        a.fillRect(-3, -65, 6, 65);
        a.fillEllipse(12, -24, 28, 12);
        a.fillStyle(palette.accent);
        for (let i = 0; i < 8; i++)
          a.fillCircle(
            Math.cos((i * Math.PI) / 4) * 17,
            -68 + Math.sin((i * Math.PI) / 4) * 17,
            12,
          );
        a.fillStyle(0x896443);
        a.fillCircle(0, -68, 11);
        break;
      default:
        a.fillStyle(0x8e7158);
        a.fillRoundedRect(-9, -70, 18, 70, 4);
        a.fillStyle(palette.canopy);
        if (palette.feature === 'pine') {
          a.fillTriangle(-44, -20, 0, -120, 44, -20);
          a.fillStyle(0xffffff, 0.7);
          a.fillTriangle(-24, -77, 0, -120, 24, -77);
        } else if (palette.feature === 'palm') {
          for (let i = -2; i <= 2; i++) a.fillEllipse(i * 17, -83 + Math.abs(i) * 6, 64, 18);
        } else {
          a.fillCircle(0, -84, 42);
          a.fillCircle(-28, -61, 30);
          a.fillCircle(29, -62, 30);
          a.fillStyle(palette.accent, 0.65);
          a.fillCircle(-16, -91, 13);
          a.fillCircle(18, -64, 10);
        }
    }
  };
  for (let i = 0; i < 22; i++)
    feature(
      40 + random() * 1040,
      i < 14 ? 110 + random() * 100 : 650 + random() * 35,
      0.55 + random() * 0.6,
    );
  feature(155, 190, 1.15);
  feature(225, 180, 1.2);
  obstacles.forEach((o) => feature(o.x, o.y, o.radius / 30));
  for (let i = 0; i < 90; i++) {
    g.fillStyle(palette.accent, 0.7);
    g.fillCircle(random() * 1120, 260 + random() * 350, 2 + random() * 2);
  }
  scene.add
    .text(560, 55, map.name, {
      fontFamily: 'Trebuchet MS',
      fontSize: '24px',
      color: '#ffffff',
      stroke: '#465565',
      strokeThickness: 4,
    })
    .setOrigin(0.5)
    .setDepth(1);
}
