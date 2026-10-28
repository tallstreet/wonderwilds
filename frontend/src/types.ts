import type Phaser from 'phaser';

export interface Point {
  x: number;
  y: number;
}
export type Blocked = (x: number, y: number) => boolean;
export type Rect = [number, number, number, number];
export type Facing = 'down' | 'up' | 'left' | 'right';
export interface EncounterBase extends Point {
  id: string;
  region: string;
  skillId: string;
  title: string;
  short: string;
  character: string;
  prompt: string;
  reward: string;
}
export type Encounter = EncounterBase &
  (
    | { type: 'missing'; total: number; given: number }
    | { type: 'share'; total: number; groups: number }
    | { type: 'pattern'; start: number; step: number; length: number }
  );
export interface Evidence {
  encounterId: string;
  skillId: string;
  variant: string;
  representation: string;
  attempts: number;
  hints: number;
  assisted: boolean;
  supported: boolean;
  completedAt: string;
}
export interface Progress {
  version: number;
  region: string;
  hero: string;
  companionName: string;
  done: string[];
  evidence: Evidence[];
  position: Point;
}
export interface Attempt {
  attempts: number;
  hints: number;
  assisted: boolean;
}
interface ActivityBase extends Attempt {
  finished: boolean;
  feedback: string;
  hintText: string;
}
export type Activity = ActivityBase &
  (
    | { kind: 'share'; encounter: Extract<Encounter, { type: 'share' }>; value: number[] }
    | { kind: 'missing'; encounter: Extract<Encounter, { type: 'missing' }>; value: number }
    | { kind: 'pattern'; encounter: Extract<Encounter, { type: 'pattern' }>; value: number }
  );
export type Actor = Phaser.GameObjects.Container & {
  art: Phaser.GameObjects.Image;
  shadow: Phaser.GameObjects.Ellipse;
  heroId: string;
  baseWidth: number;
  baseHeight: number;
  phase: number;
  facing: Facing;
};
export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
