import type { Progress, WorldEncounter, Point, Actor } from './types.ts';
import { paintScenery } from './scenery.ts';
import Phaser from 'phaser';
import { maps, mapExit, exitPoint, type MapId } from './maps.ts';
import { encounters } from './content.ts';
import { HERO_ATLAS, PROP_ATLAS, heroFrames, propFrames, registerAtlas } from './sprites.ts';
import { isBlocked, findRoute } from './navigation.ts';
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
    companion(): { sprite: string; name: string } | undefined;
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
    friend: Actor | null = null;
    friendTrail: Point[] = [];
    markers: (Point & { e: WorldEncounter; cat: Phaser.GameObjects.Image })[] = [];
    friendLabel?: Phaser.GameObjects.Text;
    you!: Phaser.GameObjects.Text;
    prompt!: Phaser.GameObjects.Text;
    lastSave = 0;
    paintedRegion = '';
    exitArmed = true;

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
          if (!objects.length && !document.querySelector('dialog[open]'))
            this.walkTo(this.cameras.main.getWorldPoint(p.x, p.y));
        },
      );
      this.game.events.on('refresh', () => this.paint());
      this.game.events.on('reset-position', () => this.player.setPosition(160, 490));
      this.game.events.on('visit', (e: WorldEncounter) => {
        if (getState().region !== e.region) {
          onMapChange?.('meadow', { x: 160, y: 490 });
          this.paint();
        }
        const m = this.markers.find((m) => m.e.id === e.id);
        if (m) this.walkTo({ x: m.x - 30, y: m.y + 25 }, m.e);
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
    paint() {
      const old =
        this.player?.active && this.paintedRegion === getState().region
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
      paintScenery(this, s);
      if (s.region === 'meadow')
        this.add
          .zone(1030, 355, 85, 110)
          .setInteractive({ useHandCursor: true })
          .on('pointerdown', () => {
            this.walkTo({ x: 1000, y: 375 }, 'portal');
          });
      this.markers = [];
      ((backend?.encounters() ?? encounters) as WorldEncounter[])
        .filter((e) => e.region === s.region && s.region === 'meadow')
        .forEach((e, i) => {
          const complete = s.done.includes(e.id),
            x = backend ? e.x : e.x + (i === 0 ? -105 : 50),
            y = backend ? e.y : e.y + (i === 0 ? 45 : 65) + 38;
          this.add.ellipse(x, y, 39, 10, 0x354e44, 0.2).setDepth(y - 0.5);
          const sprite = e.sprite ?? 'pip';
          const character = availableCharacter(sprite, this.textures);
          const npcFacing = character
            ? { originX: 0.5, originY: 0.96, scaleX: 1, scaleY: 1 }
            : resolveFacing(heroFrames.includes(sprite) ? sprite : 'pip', 0, 0, 'down');
          const cat = this.add
            .image(
              x,
              y,
              character?.texture ?? 'hero-atlas',
              character?.frame ?? (heroFrames.includes(sprite) ? sprite : 'pip'),
            )
            .setOrigin(npcFacing.originX, npcFacing.originY)
            .setDisplaySize(86 * npcFacing.scaleX, 96 * npcFacing.scaleY)
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
          zone.on('pointerdown', () => this.walkTo({ x: x - 30, y: y + 25 }, e));
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
      const pos = old && !isBlocked(old.x, old.y, fixed, s.region) ? old : { x: 160, y: 490 };
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
      this.friend = null;
      this.friendLabel = undefined;
      this.friendTrail = [];
      const companion = backend?.companion();
      if (backend ? companion : s.done.length === 3) {
        const start = !this.blocked(pos.x - 40, pos.y) ? { x: pos.x - 40, y: pos.y } : pos;
        this.friend = this.makeActor(start.x, start.y, companion?.sprite ?? 'pip', 78, 87);
        if (companion) this.friendLabel = this.label(start.x, start.y - 90, companion.name, 10);
        this.friendTrail = findRoute(start, pos, fixed, s.region);
        this.friendTrail.push({ x: pos.x, y: pos.y });
      }
      this.prompt = this.label(
        560,
        653,
        'TAP TO EXPLORE  •  FIND A FRIEND  •  BE A HERO',
        12,
        '#6e5288',
      ).setDepth(2000);
      for (const side of ['west', 'east'] as const) {
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
      return isBlocked(x, y, getState().done.includes('meadow-bridge'), getState().region);
    }
    walkTo(point: Point, pending: WorldEncounter | 'portal' | null = null) {
      if (document.querySelector('dialog[open]')) return;
      this.pending = pending;
      this.route = findRoute(
        this.player,
        point,
        getState().done.includes('meadow-bridge'),
        getState().region,
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
    interact() {
      const close = this.markers.find(
        (m) => Phaser.Math.Distance.Between(this.player.x, this.player.y, m.x, m.y) < 110,
      );
      if (close) onEncounter(close.e);
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
      if (document.querySelector('dialog[open]')) {
        // Do not leave a bunny suspended above its shadow while a modal is open.
        this.poseActor(this.player, 0, 0, 0);
        if (this.friend) this.poseActor(this.friend, 0, 0, 0);
        return;
      }
      let dx = 0,
        dy = 0;
      const k = this.keys,
        dt = Math.min(delta, 40);
      if (k.LEFT.isDown || k.A.isDown) dx--;
      if (k.RIGHT.isDown || k.D.isDown) dx++;
      if (k.UP.isDown || k.W.isDown) dy--;
      if (k.DOWN.isDown || k.S.isDown) dy++;
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
          else onEncounter(pending);
        }
      }
      const oldX = this.player.x,
        oldY = this.player.y;
      const distance = this.target ? Math.min(dt * 0.23, Math.hypot(dx, dy)) : dt * 0.23;
      const motion = moveGround(this.player, dx, dy, distance, (x, y) => this.blocked(x, y));
      this.player.setPosition(motion.x, motion.y);
      this.poseActor(this.player, motion.x - oldX, motion.y - oldY, motion.distance);
      this.you.setPosition(this.player.x, this.player.y - 120).setDepth(this.player.y + 100);
      if (this.friend) {
        this.friendTrail.push(...motion.path);
        const before = { x: this.friend.x, y: this.friend.y };
        const follow = advanceFollower(before, this.friendTrail, dt * 0.26, 48, (x, y) =>
          this.blocked(x, y),
        );
        this.friend.setPosition(follow.x, follow.y);
        this.friendLabel?.setPosition(follow.x, follow.y - 90).setDepth(follow.y + 100);
        this.poseActor(this.friend, follow.x - before.x, follow.y - before.y, follow.distance);
      }
      const exit = mapExit(getState().region, this.player);
      if (!exit) this.exitArmed = true;
      if (exit && this.exitArmed && onMapChange) {
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
