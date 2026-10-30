import type {
  Progress,
  WorldEncounter,
  Point,
  Actor,
  StoryShape,
  WorldDecoration,
} from './types.ts';
import { paintScenery } from './scenery.ts';
import Phaser from 'phaser';
import { maps, mapExit, exitPoint, type MapId } from './maps.ts';
import { encounters } from './content.ts';
import { HERO_ATLAS, PROP_ATLAS, heroFrames, propFrames, registerAtlas } from './sprites.ts';
import { questWaypoint } from './quest-navigation.ts';
import { isBlocked, findRoute, nearestCharacter } from './navigation.ts';
import { heroes } from './art.ts';
import { moveGround, advanceFollower } from './grounding.ts';
import {
  advanceGaitPhase,
  sampleGait,
  preloadWalkCycle,
  registerWalkCycle,
  applyWalkCycle,
  walkFrameOrigin,
} from './locomotion.ts';
import { preloadFacing, registerFacing, resolveFacing } from './facing.ts';
import { availableCharacter, preloadCharacters, registerCharacters } from './character-sprites.ts';
export function createWorld(
  parent: string,
  getState: () => Progress,
  onEncounter: (encounter: WorldEncounter) => void,
  onPortal: () => void,
  onPosition?: (point: Point) => void,
  onMapChange?: (region: MapId, position: Point) => void,
  backend?: {
    encounters(): WorldEncounter[];
    companions(): { sprite: string; name: string; offsetX?: number }[];
    decorations?(): WorldDecoration[];
    allowExits?: boolean;
    blocked?(x: number, y: number): boolean;
    inputLocked?(): boolean;
    observe?(point: Point, followers: { x: number; y: number; sprite: string }[]): void;
  },
) {
  class Island extends Phaser.Scene {
    target: Point | null = null;
    route: Point[] = [];
    pending: WorldEncounter | 'portal' | null = null;
    reducedMotion = false;
    keys!: Record<
      'W' | 'A' | 'S' | 'D' | 'UP' | 'DOWN' | 'LEFT' | 'RIGHT' | 'SPACE',
      Phaser.Input.Keyboard.Key
    >;
    player!: Actor;
    followers: { actor: Actor; trail: Point[]; label?: Phaser.GameObjects.Text }[] = [];
    markers: (Point & { e: WorldEncounter; cat: Phaser.GameObjects.GameObject })[] = [];
    you!: Phaser.GameObjects.Text;
    prompt!: Phaser.GameObjects.Text;
    lastSave = 0;
    paintedRegion = '';
    exitArmed = true;
    arrival?: () => void;

    constructor() {
      super('island');
      this.target = null;
      this.route = [];
      this.reducedMotion = false;
    }
    preload() {
      this.load.image('meadow-map', '/assets/clover-meadow-v2.png');
      this.load.image('orchard-map', '/assets/peachy-orchard-v2.png');
      this.load.image('cove-map', '/assets/starlight-cove-v2.png');
      this.load.image('hero-atlas', HERO_ATLAS);
      this.load.image('prop-atlas', PROP_ATLAS);
      preloadWalkCycle(this);
      preloadFacing(this);
      preloadCharacters(this);
    }
    create() {
      this.reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
      registerWalkCycle(this);
      registerFacing(this);
      registerCharacters(this);
      registerAtlas(this.textures.get('hero-atlas'), heroFrames);
      registerAtlas(this.textures.get('prop-atlas'), propFrames);
      const keyboard = this.input.keyboard;
      if (!keyboard) throw new Error('Keyboard input is required');
      this.keys = keyboard.addKeys('W,A,S,D,UP,DOWN,LEFT,RIGHT,SPACE') as Island['keys'];
      keyboard.addCapture(['UP', 'DOWN', 'LEFT', 'RIGHT', 'SPACE']);
      this.paint();
      this.scale.on('resize', () => this.fitCamera());
      this.input.on(
        'pointerdown',
        (p: Phaser.Input.Pointer, objects: Phaser.GameObjects.GameObject[]) => {
          if (
            !objects.length &&
            !document.querySelector('dialog[open]') &&
            !backend?.inputLocked?.()
          )
            this.walkTo(this.cameras.main.getWorldPoint(p.x, p.y));
        },
      );
      this.game.events.on('refresh', () => this.paint());
      this.game.events.on('authoritative-position', () => this.paint(true));
      this.game.events.on('scripted-walk', (point: Point, arrived: () => void) => {
        this.walkTo(point, null, true);
        this.arrival = arrived;
      });
      this.game.events.on('reset-position', () => this.player.setPosition(160, 490));
      this.game.events.on('visit', (e: WorldEncounter) => {
        if (backend?.inputLocked?.()) return;
        if (getState().region !== e.region) {
          onMapChange?.(e.region, { x: 160, y: 490 });
          this.paint();
        }
        const m = this.markers.find((m) => m.e.id === e.id);
        if (m) this.walkTo({ x: m.x - 30, y: m.y + 25 }, m.e);
      });
      this.game.events.on('guide', (e: WorldEncounter) => {
        if (backend?.inputLocked?.()) return;
        const waypoint = questWaypoint(getState().region, e);
        if (waypoint) this.walkTo(waypoint.point, waypoint.arrived ? e : null);
      });
      keyboard.on('keydown-SPACE', () => this.interact());
    }
    label(x: number, y: number, text: string, size = 16, color = '#6a4c86') {
      return this.add
        .text(x, y, text, {
          fontFamily: 'Nunito, Trebuchet MS, sans-serif',
          fontSize: size,
          fontStyle: 'bold',
          color,
          align: 'center',
          backgroundColor: '#fff9ec',
          padding: { x: 12, y: 7 },
        })
        .setOrigin(0.5)
        .setDepth(y + 50);
    }
    paint(forcePosition = false) {
      const old =
        !forcePosition && this.player?.active && this.paintedRegion === getState().region
          ? { x: this.player.x, y: this.player.y }
          : getState().position;
      this.tweens.killAll();
      this.children.removeAll(true);
      this.target = null;
      this.route = [];
      this.pending = null;
      const s = getState();
      this.paintedRegion = s.region;
      const fixed = s.done.includes('meadow-bridge');
      paintScenery(this, s, Boolean(backend));
      for (const decoration of backend?.decorations?.() ?? []) {
        // Meadow already contains the picnic blanket; its display flags add the food.
        if (decoration.kind === 'picnic' && s.region !== 'meadow')
          this.add
            .image(decoration.x, decoration.y, 'prop-atlas', 'picnic')
            .setDisplaySize(160, 100)
            .setOrigin(0.5, 0.9)
            .setDepth(decoration.y);
        else if (decoration.kind !== 'picnic')
          this.storyShape(decoration.x, decoration.y, decoration.kind, true);
      }
      if (s.region === 'meadow' && backend?.allowExits !== false)
        this.add
          .zone(1030, 355, 85, 110)
          .setInteractive({ useHandCursor: true })
          .on('pointerdown', () => {
            this.walkTo({ x: 1000, y: 375 }, 'portal');
          });
      this.markers = [];
      ((backend?.encounters() ?? encounters) as WorldEncounter[])
        .filter((e) => e.region === s.region)
        .forEach((e, i) => {
          const complete = s.done.includes(e.id),
            x = backend ? e.x : e.x + (i === 0 ? -105 : 50),
            y = backend ? e.y : e.y + (i === 0 ? 45 : 65) + 38;
          this.add.ellipse(x, y, 39, 10, 0x354e44, 0.2).setDepth(y - 0.5);
          const sprite = e.sprite ?? 'pip';
          const character =
            e.prop || e.shape ? undefined : availableCharacter(sprite, this.textures);
          const npcFacing = character
            ? { originX: 0.5, originY: 0.96, scaleX: 1, scaleY: 1 }
            : resolveFacing(heroFrames.includes(sprite) ? sprite : 'pip', 0, 0, 'down');
          const cat = e.shape
            ? this.storyShape(x, y, e.shape, false)
            : this.add
                .image(
                  x,
                  y,
                  e.prop ? 'prop-atlas' : (character?.texture ?? 'hero-atlas'),
                  e.prop ?? character?.frame ?? (heroFrames.includes(sprite) ? sprite : 'pip'),
                )
                .setOrigin(npcFacing.originX, npcFacing.originY)
                .setDisplaySize(
                  e.prop === 'berry' ? 32 : 86 * npcFacing.scaleX,
                  e.prop === 'berry' ? 32 : 96 * npcFacing.scaleY,
                )
                .setDepth(y);
          const badge = this.add
            .circle(x, y - 112, 16, complete ? 0xffffff : 0xfff2be)
            .setStrokeStyle(3, complete ? 0x8abb8d : 0xffffff)
            .setDepth(y + 100);
          this.add
            .text(x, y - 112, complete ? '✓' : '!', {
              fontFamily: 'Trebuchet MS',
              fontSize: 22,
              fontStyle: 'bold',
              color: complete ? '#60a17a' : '#8f6a3d',
            })
            .setOrigin(0.5)
            .setDepth(y + 101);
          const zone = this.add
            .zone(x, y - 43, 88, 144)
            .setDepth(y + 102)
            .setInteractive({ useHandCursor: true });
          zone.on('pointerdown', () => {
            if (document.querySelector('dialog[open]') || backend?.inputLocked?.()) return;
            if (Math.hypot(this.player.x - x, this.player.y - y) <= 110) this.chat(e);
            else this.walkTo({ x: x - 30, y: y + 25 }, e);
          });
          this.markers.push({ e, x, y, cat });
          if (backend) this.label(x, y + 12, e.character, 10);
          if (!complete && !this.reducedMotion)
            this.tweens.add({
              targets: [badge],
              y: y - 118,
              duration: 900,
              yoyo: true,
              repeat: -1,
              ease: 'Sine.easeInOut',
            });
        });
      const pos = old && !this.blocked(old.x, old.y) ? old : { x: 160, y: 490 };
      this.player = this.makeActor(pos.x, pos.y, s.hero, 100, 112);

      this.you = this.add
        .text(pos.x, pos.y - 120, 'YOU', {
          fontSize: 10,
          fontFamily: 'Trebuchet MS',
          fontStyle: 'bold',
          color: '#665381',
          backgroundColor: '#ffffff',
          padding: { x: 7, y: 4 },
        })
        .setOrigin(0.5)
        .setDepth(pos.y + 100);
      this.followers = [];
      const companions: { sprite: string; name: string; offsetX?: number }[] =
        backend?.companions() ?? (s.done.length === 3 ? [{ sprite: 'pip', name: '' }] : []);
      for (const [index, companion] of companions.entries()) {
        const offset = 40 + index * 42;
        const startX = pos.x + (companion.offsetX ?? -offset);
        const start = !this.blocked(startX, pos.y) ? { x: startX, y: pos.y } : pos;
        const actor = this.makeActor(start.x, start.y, companion.sprite, 78, 87);
        const trail = findRoute(start, pos, fixed, s.region, backend?.blocked);
        trail.push({ x: pos.x, y: pos.y });
        this.followers.push({
          actor,
          trail,
          label: companion.name ? this.label(start.x, start.y - 90, companion.name, 10) : undefined,
        });
      }
      this.prompt = this.label(
        560,
        653,
        'TAP TO EXPLORE  •  FIND A FRIEND  •  BE A HERO',
        12,
        '#6e5288',
      ).setDepth(2000);
      for (const side of backend?.allowExits === false ? [] : (['west', 'east'] as const)) {
        const destination = maps[s.region as MapId][side];
        const point = exitPoint(s.region as MapId, side);
        this.label(
          side === 'west' ? 108 : 1012,
          point.y + 46,
          `${side === 'west' ? '← ' : ''}${maps[destination].name}${side === 'east' ? ' →' : ''}`,
          11,
        );
        this.add
          .zone(point.x, point.y, 64, 36)
          .setDepth(2001)
          .setInteractive({ useHandCursor: true })
          .on('pointerdown', () => this.walkTo(point));
      }
      this.addAmbientDetails();
      this.prompt.setVisible(false);
      this.fitCamera();
    }
    storyShape(x: number, y: number, kind: StoryShape, lit: boolean) {
      const shape = this.add.graphics({ x, y }).setDepth(y);
      if (kind === 'stones') {
        shape.fillStyle(0xffee8b, 0.25).fillEllipse(0, -10, 76, 44);
        for (const [dx, dy] of [
          [-20, -9],
          [2, -20],
          [23, -6],
        ]) {
          shape.fillStyle(0xc2dad0).fillEllipse(dx, dy, 25, 21);
          shape.fillStyle(0xfff3a3).fillCircle(dx - 3, dy - 4, 6);
        }
      } else if (kind === 'lantern') {
        if (lit) shape.fillStyle(0xffdf70, 0.2).fillCircle(0, -55, 48);
        shape.lineStyle(6, 0x76543b).lineBetween(0, 0, 0, -90).lineBetween(0, -90, 18, -90);
        shape.fillStyle(lit ? 0xffe382 : 0x8c9d8c).fillRoundedRect(5, -83, 28, 38, 6);
        shape.lineStyle(2, 0x76543b).strokeRoundedRect(5, -83, 28, 38, 6);
        if (lit) shape.fillStyle(0xfffae1).fillCircle(19, -64, 7);
      } else if (kind === 'leaf') {
        shape.fillStyle(0x7eaf54).fillEllipse(0, -18, 43, 24);
        shape.lineStyle(2, 0x416a38).lineBetween(-23, -14, 17, -20);
        shape.lineBetween(-4, -17, -11, -26).lineBetween(5, -19, 12, -10);
      } else if (kind === 'hollow' || kind === 'instrument') {
        shape.fillStyle(0x96643f).fillRoundedRect(-40, -30, 80, 30, 12);
        shape.fillStyle(0x4e3527).fillEllipse(-30, -16, 18, 21);
        if (kind === 'instrument') {
          shape.lineStyle(4, 0xd8ba79).lineBetween(-28, -44, 28, -44);
          for (const [i, height] of [22, 32, 28, 19].entries()) {
            shape.lineStyle(2, 0xe8d1a6).lineBetween(-22 + i * 15, -44, -22 + i * 15, -35);
            shape.fillStyle(0xebbd68).fillRoundedRect(-25 + i * 15, -35, 7, height, 3);
          }
        }
      } else if (kind === 'corner-leaves' || kind === 'corner-stars') {
        shape.fillStyle(0x9a794d).fillRoundedRect(-50, -20, 100, 20, 8);
        shape.lineStyle(5, 0x785837).lineBetween(-43, 0, -43, -75).lineBetween(43, 0, 43, -75);
        if (kind === 'corner-leaves') {
          for (const [dx, dy] of [
            [-30, -65],
            [0, -78],
            [30, -65],
          ])
            shape.fillStyle(0x85bc63).fillEllipse(dx, dy, 62, 32);
        } else {
          shape.fillStyle(0x9e7cc9).fillTriangle(-50, -67, 0, -90, 50, -67);
          for (const dx of [-27, 0, 27]) {
            shape.lineStyle(1, 0xf6de8b).lineBetween(dx, -65, dx, -48);
            shape.fillStyle(0xffedac).fillCircle(dx, -45, 4);
          }
        }
        shape.fillStyle(0xf0b4b4).fillRoundedRect(-28, -26, 56, 14, 6);
      } else if (kind === 'music') {
        for (const [dx, dy] of [
          [-20, 0],
          [15, -14],
        ]) {
          shape.lineStyle(4, 0x8862b0).lineBetween(dx + 7, dy - 7, dx + 7, dy - 47);
          shape.fillStyle(0x8862b0).fillEllipse(dx, dy - 5, 19, 12);
          shape.lineStyle(5, 0x8862b0).lineBetween(dx + 7, dy - 47, dx + 22, dy - 39);
        }
      } else if (kind === 'seating') {
        for (const dx of [-30, 30]) {
          shape.fillStyle(0x765339).fillRoundedRect(dx - 15, -23, 30, 23, 5);
          shape.fillStyle(0xcfb178).fillEllipse(dx, -23, 32, 14);
        }
      } else if (kind === 'bunting') {
        shape.lineStyle(4, 0x88633f).lineBetween(-50, 0, -50, -80).lineBetween(50, 0, 50, -80);
        shape.lineStyle(2, 0xffedb6).lineBetween(-50, -72, 50, -72);
        for (const [i, colour] of [0xf5a6bd, 0x9ccdb1, 0xc8b0ed, 0xf2d38b].entries())
          shape
            .fillStyle(colour)
            .fillTriangle(-43 + i * 23, -71, -23 + i * 23, -71, -33 + i * 23, -49);
      } else if (kind === 'trail') {
        for (const [dx, dy] of [
          [-30, -5],
          [-5, -15],
          [23, -27],
        ])
          shape.fillStyle(lit ? 0xd6bd8e : 0x9d7653).fillEllipse(dx, dy, 24, 12);
      }
      return shape;
    }
    fitCamera() {
      if (!this.player) return;
      const camera = this.cameras.main;
      camera.setZoom(Math.max(this.scale.width / 1120, this.scale.height / 690));
      camera.setBounds(0, 0, 1120, 690);
      camera.startFollow(this.player, true, 0.12, 0.12);
      camera.centerOn(this.player.x, this.player.y);
    }
    addAmbientDetails() {
      const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
      for (let i = 0; i < 10; i++) {
        const x = 90 + i * 101,
          y = 118 + ((i * 89) % 460);
        const star = this.add.star(x, y, 4, 1.5, 5, 0xfff8c7, 0.9).setDepth(1800);
        if (!reduced)
          this.tweens.add({
            targets: star,
            alpha: 0.15,
            scale: 0.4,
            duration: 1400 + i * 110,
            yoyo: true,
            repeat: -1,
          });
      }
      for (const [x, y, color] of [
        [380, 180, 0xffb8de],
        [780, 520, 0xc6abff],
        [200, 390, 0xffd96e],
      ]) {
        const butterfly = this.add.container(x, y).setDepth(1500);
        butterfly.add([
          this.add.ellipse(-4, 0, 9, 13, color),
          this.add.ellipse(4, 0, 9, 13, color),
          this.add.ellipse(0, 1, 2, 10, 0x8b609a),
        ]);
        if (!reduced)
          this.tweens.add({
            targets: butterfly,
            x: x + 35,
            y: y - 17,
            scaleX: 0.6,
            duration: 2300,
            yoyo: true,
            repeat: -1,
            ease: 'Sine.easeInOut',
          });
      }
    }
    flower(x: number, y: number, c: number, scale: number) {
      const g = this.add.graphics().setDepth(y - 20);
      g.fillStyle(c);
      for (let i = 0; i < 5; i++) {
        const a = (i * 6.28) / 5;
        g.fillCircle(x + Math.cos(a) * 4 * scale, y + Math.sin(a) * 4 * scale, 3.5 * scale);
      }
      g.fillStyle(0xffd469);
      g.fillCircle(x, y, 2.5 * scale);
    }
    blocked(x: number, y: number) {
      return (
        isBlocked(x, y, getState().done.includes('meadow-bridge'), getState().region) ||
        Boolean(backend?.blocked?.(x, y))
      );
    }
    walkTo(point: Point, pending: WorldEncounter | 'portal' | null = null, scripted = false) {
      if (document.querySelector('dialog[open]') || (!scripted && backend?.inputLocked?.())) return;
      this.pending = pending;
      this.route = findRoute(
        this.player,
        point,
        getState().done.includes('meadow-bridge'),
        getState().region,
        backend?.blocked,
      );
      this.target = this.route.shift() || null;
      if (!this.target) {
        this.pending = null;
        this.prompt
          .setVisible(true)
          .setPosition(this.player.x, this.player.y - 85)
          .setText('Fix the bridge to cross!');
        this.time.delayedCall(3000, () => {
          if (this.prompt?.active) this.prompt.setVisible(false);
        });
      }
    }
    chat(encounter: WorldEncounter) {
      if (document.querySelector('dialog[open]') || backend?.inputLocked?.()) return;
      // Stop any click-to-walk interaction so it cannot reopen chat after closing.
      this.target = null;
      this.route = [];
      this.pending = null;
      onPosition?.({ x: this.player.x, y: this.player.y });
      onEncounter(encounter);
    }
    interact() {
      if (document.querySelector('dialog[open]') || backend?.inputLocked?.()) return;
      const close = nearestCharacter(this.player, this.markers);
      if (close) this.chat(close.e);
      else if (
        getState().region === 'meadow' &&
        this.player.x > 950 &&
        this.player.y > 300 &&
        this.player.y < 440
      )
        onPortal();
    }
    makeActor(x: number, y: number, heroId: string, width: number, height: number): Actor {
      const ground = this.add.container(x, y).setDepth(y) as Actor;
      const character = availableCharacter(heroId, this.textures);
      ground.art = this.add
        .image(
          0,
          0,
          character?.texture ?? 'hero-atlas',
          character?.frame ?? (heroFrames.includes(heroId) ? heroId : 'pip'),
        )
        .setOrigin(0.5, 0.96)
        .setDisplaySize(width, height);
      ground.add(ground.art);
      ground.shadow = this.add
        .ellipse(x, y, Math.round(width * 0.42), Math.round(width * 0.11), 0x354e44, 1)
        .setAlpha(0.22)
        .setDepth(y - 0.5);
      ground.heroId = heroId;
      ground.baseWidth = width;
      ground.baseHeight = height;
      ground.phase = 0;
      ground.facing = 'down';
      this.poseActor(ground, 0, 0, 0);
      return ground;
    }
    poseActor(actor: Actor, dx: number, dy: number, distance: number) {
      const character = availableCharacter(actor.heroId, this.textures);
      const moving = distance > 0.01,
        species = character?.species ?? heroes[actor.heroId]?.species ?? 'kitten';
      actor.phase = moving ? advanceGaitPhase(actor.phase, distance, species) : 0;
      // These library sheets supply front poses; do not borrow another hero's side/back art.
      const facing = character
        ? {
            texture: character.texture,
            frame: character.frame,
            flipX: false,
            facing: 'down' as const,
            originX: 0.5,
            originY: 0.96,
            scaleX: 1,
            scaleY: 1,
          }
        : resolveFacing(
            heroFrames.includes(actor.heroId) ? actor.heroId : 'pip',
            dx,
            dy,
            actor.facing,
          );
      actor.facing = facing.facing;
      const texture =
        this.textures.exists(facing.texture) && this.textures.get(facing.texture).has(facing.frame)
          ? facing.texture
          : 'hero-atlas';
      actor.art
        .setTexture(texture, facing.frame)
        .setFlipX(facing.flipX)
        .setOrigin(facing.originX ?? 0.5, facing.originY ?? 0.96);
      const spriteWalk =
        !character &&
        applyWalkCycle(actor.art, actor.heroId, actor.phase, moving, {
          reducedMotion: this.reducedMotion,
          facing: actor.facing,
        });
      if (spriteWalk) {
        const origin = walkFrameOrigin(actor.phase);
        actor.art.setOrigin(origin.x, origin.y);
      }
      const gait = sampleGait(species, actor.phase, moving, { reducedMotion: this.reducedMotion });
      // The real walking frames already contain the kitten's weight shift.
      const lift = spriteWalk ? 0 : gait.lift,
        sx = spriteWalk ? 1 : gait.scaleX,
        sy = spriteWalk ? 1 : gait.scaleY;
      actor.art
        .setDisplaySize(
          actor.baseWidth * sx * (spriteWalk ? 1 : (facing.scaleX ?? 1)),
          actor.baseHeight * sy * (spriteWalk ? 1 : (facing.scaleY ?? 1)),
        )
        .setPosition(0, -lift)
        .setAngle(spriteWalk ? 0 : gait.rotation);
      actor.setDepth(actor.y);
      const shadowScale = Math.max(0.65, 1 - lift * 0.023);
      actor.shadow
        .setPosition(actor.x, actor.y + 1)
        .setScale(shadowScale)
        .setAlpha(0.22 * Math.max(0.55, 1 - lift * 0.035))
        .setDepth(actor.y - 0.5);
    }
    update(t: number, delta: number) {
      if (!this.player) return;
      if (document.querySelector('dialog[open]') || (backend?.inputLocked?.() && !this.arrival)) {
        // Do not leave a bunny suspended above its shadow while a modal is open.
        this.poseActor(this.player, 0, 0, 0);
        for (const follower of this.followers) this.poseActor(follower.actor, 0, 0, 0);
        return;
      }
      let dx = 0,
        dy = 0;
      const k = this.keys,
        dt = Math.min(delta, 40);
      if (!backend?.inputLocked?.() && (k.LEFT.isDown || k.A.isDown)) dx--;
      if (!backend?.inputLocked?.() && (k.RIGHT.isDown || k.D.isDown)) dx++;
      if (!backend?.inputLocked?.() && (k.UP.isDown || k.W.isDown)) dy--;
      if (!backend?.inputLocked?.() && (k.DOWN.isDown || k.S.isDown)) dy++;
      if (dx || dy) {
        this.target = null;
        this.route = [];
        this.pending = null;
      } else if (this.target) {
        while (
          this.target &&
          Math.hypot(this.target.x - this.player.x, this.target.y - this.player.y) < 2
        )
          this.target = this.route.shift() || null;
        if (this.target) {
          dx = this.target.x - this.player.x;
          dy = this.target.y - this.player.y;
        } else if (this.pending) {
          const pending = this.pending;
          this.pending = null;
          if (pending === 'portal') onPortal();
          else this.chat(pending);
        }
      }
      if (!this.target && this.arrival) {
        const arrived = this.arrival;
        this.arrival = undefined;
        arrived();
      }
      const oldX = this.player.x,
        oldY = this.player.y;
      const distance = this.target ? Math.min(dt * 0.23, Math.hypot(dx, dy)) : dt * 0.23;
      const motion = moveGround(this.player, dx, dy, distance, (x, y) => this.blocked(x, y));
      this.player.setPosition(motion.x, motion.y);
      this.poseActor(this.player, motion.x - oldX, motion.y - oldY, motion.distance);
      this.you.setPosition(this.player.x, this.player.y - 120).setDepth(this.player.y + 100);
      for (const [index, follower] of this.followers.entries()) {
        follower.trail.push(...motion.path);
        const before = { x: follower.actor.x, y: follower.actor.y };
        const follow = advanceFollower(before, follower.trail, dt * 0.26, 48 + index * 46, (x, y) =>
          this.blocked(x, y),
        );
        follower.actor.setPosition(follow.x, follow.y);
        follower.label?.setPosition(follow.x, follow.y - 90).setDepth(follow.y + 100);
        this.poseActor(follower.actor, follow.x - before.x, follow.y - before.y, follow.distance);
      }
      backend?.observe?.(
        { x: this.player.x, y: this.player.y },
        this.followers.map((follower) => ({
          x: follower.actor.x,
          y: follower.actor.y,
          sprite: follower.actor.heroId,
        })),
      );
      const exit = mapExit(getState().region, this.player);
      if (!exit) this.exitArmed = true;
      if (exit && this.exitArmed && onMapChange && backend?.allowExits !== false) {
        this.exitArmed = false;
        onMapChange(exit.region, exit.position);
        this.paint();
        if (!this.reducedMotion) this.cameras.main.fadeIn(350, 255, 248, 235);
        return;
      }
      if (t - (this.lastSave || 0) > 1000) {
        this.lastSave = t;
        onPosition?.({ x: this.player.x, y: this.player.y });
      }
    }
  }
  return new Phaser.Game({
    type: Phaser.AUTO,
    parent,
    width: 1120,
    height: 690,
    backgroundColor: '#bce0ad',
    scene: Island,
    scale: { mode: Phaser.Scale.RESIZE, autoCenter: Phaser.Scale.CENTER_BOTH },
    render: { antialias: true },
    input: { keyboard: true },
  });
}
