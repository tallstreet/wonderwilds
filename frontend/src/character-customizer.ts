import type Phaser from 'phaser';
import {
  CharacterSheetError,
  CELL,
  CUSTOM_TEXTURE,
  SHEET_WIDTH,
  SHEET_HEIGHT,
  loadCharacterImage,
  prepareCharacter,
  validSavedCharacter,
} from './custom-character.ts';
import './character-customizer.css';

type Hooks = { open(html: string): void; close(): void; toast(message: string): void };
export class CharacterCustomizer {
  private saved?: string;
  private pending?: string;
  private request?: AbortController;
  private storageKey: string;
  private disposed = false;
  private debug?: { url: string; keyed?: string; report?: string };
  private releaseDebug() {
    if (this.debug) URL.revokeObjectURL(this.debug.url);
    this.debug = undefined;
  }
  private showDebug(root: HTMLElement, expanded = false) {
    const panel = root.querySelector<HTMLDetailsElement>('#character-debug')!;
    panel.hidden = !this.debug;
    if (!this.debug) return;
    panel.open = expanded;
    root.querySelector<HTMLImageElement>('[data-character-raw]')!.src = this.debug.url;
    root.querySelector<HTMLAnchorElement>('[data-character-download]')!.href = this.debug.url;
    const keyed = root.querySelector<HTMLImageElement>('[data-character-keyed]')!;
    keyed.hidden = !this.debug.keyed;
    keyed.parentElement!.parentElement!.hidden = !this.debug.keyed;
    if (this.debug.keyed) keyed.src = this.debug.keyed;
    root.querySelector<HTMLElement>('[data-character-report]')!.textContent =
      this.debug.report ?? 'No cell diagnostics are available for this image.';
  }
  constructor(
    private game: Phaser.Game,
    private profile: string,
    private csrf: string,
    private enabled: boolean,
    private hooks: Hooks,
  ) {
    this.storageKey = `wonderwilds.character.v1:${profile}`;
    document.querySelector('dialog')!.addEventListener('close', () => this.request?.abort());
    window.addEventListener('pagehide', () => {
      this.disposed = true;
      this.releaseDebug();
      this.request?.abort();
    });
  }
  async restore() {
    try {
      const value: unknown = localStorage.getItem(this.storageKey);
      if (validSavedCharacter(value)) {
        await this.install(value);
        this.saved = value;
      }
    } catch {
      /* A missing or damaged cosmetic never prevents play. */
    }
  }
  private async install(url: string, current = () => true) {
    const image = await loadCharacterImage(url);
    if (this.disposed || !current()) throw new Error('Character selection closed');
    if (image.width !== SHEET_WIDTH || image.height !== SHEET_HEIGHT)
      throw new Error('Invalid saved character');
    if (this.game.textures.exists(CUSTOM_TEXTURE)) this.game.textures.remove(CUSTOM_TEXTURE);
    this.game.textures.addSpriteSheet(CUSTOM_TEXTURE, image, {
      frameWidth: CELL,
      frameHeight: CELL,
    });
    this.game.events.emit('appearance');
  }
  open() {
    this.pending = undefined;
    this.hooks.open(
      `<section class="character-customizer"><span class="pill">YOUR LOOK</span><h2 id="dialog-title">Create your character</h2><p>Imagine a friendly adventurer for Wonderwilds.</p><form id="character-form"><label for="character-description">Describe your character</label><textarea id="character-description" maxlength="400" rows="3" required placeholder="A fluffy blue fox with a yellow scarf and tiny boots"></textarea><p class="character-note">Your description goes to Meta to make the art. Use an imaginary character and leave out personal details.</p><button class="primary" type="submit" ${this.enabled ? '' : 'disabled'}>Generate character</button></form><p id="character-status" role="status">${this.enabled ? 'Preview your character before choosing it.' : 'Character generation is not configured for this demo.'}</p><div id="character-preview" hidden><div class="character-walk-preview" role="img" aria-label="Your character walking"></div><div class="character-directions" aria-label="Preview direction"><button class="secondary" type="button" data-character-row="0" aria-pressed="true">Front</button><button class="secondary" type="button" data-character-row="1" aria-pressed="false">Left</button><button class="secondary" type="button" data-character-row="2" aria-pressed="false">Right</button><button class="secondary" type="button" data-character-row="3" aria-pressed="false">Back</button></div><button class="primary" id="character-use">Use this character</button></div><details id="character-debug" hidden><summary>Inspect returned image</summary><p>The lines show the expected 3 columns and 4 rows. They are an overlay, not part of Meta’s image.</p><h3>Original response</h3><div class="character-debug-sheet"><img data-character-raw alt="Original image returned by Meta"/><span aria-hidden="true"></span></div><a data-character-download download="wonderwilds-character-source.png">Download original PNG</a><div hidden><h3>After background removal</h3><div class="character-debug-sheet"><img data-character-keyed alt="Returned image after magenta background removal"/><span aria-hidden="true"></span></div></div><h3>Sheet diagnostics</h3><pre data-character-report></pre></details><button class="secondary" id="character-reset" ${this.saved ? '' : 'hidden'}>Use original character</button><p class="character-note">Your chosen look is saved in this browser for this profile.</p></section>`,
    );
    const root = document.querySelector<HTMLElement>('.character-customizer')!;
    this.showDebug(root);
    root.querySelectorAll<HTMLButtonElement>('[data-character-row]').forEach((button) => {
      button.onclick = () => {
        root
          .querySelectorAll('[data-character-row]')
          .forEach((other) => other.setAttribute('aria-pressed', String(other === button)));
        root
          .querySelector<HTMLElement>('.character-walk-preview')!
          .style.setProperty(
            '--character-row',
            `${(Number(button.dataset.characterRow) * 100) / 3}%`,
          );
      };
    });
    root.querySelector('form')!.onsubmit = (event) => {
      event.preventDefault();
      void this.generate(root);
    };
    root.querySelector<HTMLButtonElement>('#character-use')!.onclick = () => void this.use(root);
    root.querySelector<HTMLButtonElement>('#character-reset')!.onclick = () => {
      try {
        localStorage.removeItem(this.storageKey);
      } catch {
        this.hooks.toast(
          'Browser storage is unavailable; the original look is restored for this visit.',
        );
      }
      this.saved = undefined;
      this.game.textures.remove(CUSTOM_TEXTURE);
      this.game.events.emit('appearance');
      this.hooks.close();
    };
  }
  private async generate(root: HTMLElement) {
    const description = root.querySelector<HTMLTextAreaElement>('textarea')!.value.trim();
    if (!description || description.length > 400 || this.request) return;
    const controller = new AbortController();
    this.request = controller;
    const status = root.querySelector<HTMLElement>('#character-status')!;
    const button = root.querySelector<HTMLButtonElement>('[type=submit]')!;
    button.disabled = true;
    this.pending = undefined;
    root.querySelector<HTMLElement>('#character-preview')!.hidden = true;
    status.textContent = 'Creating your walking poses… This may take a minute.';
    let objectUrl: string | undefined;
    try {
      const response = await fetch('/__demo/character', {
        method: 'POST',
        redirect: 'error',
        signal: AbortSignal.any([controller.signal, AbortSignal.timeout(125000)]),
        headers: {
          'Content-Type': 'application/json',
          'X-CSRF-Token': this.csrf,
          'X-Wonderwilds-Profile': this.profile,
        },
        body: JSON.stringify({ description }),
      });
      if (!response.ok)
        throw new Error(
          response.status === 429
            ? 'Please wait a few seconds before generating again.'
            : 'Character generation is unavailable. Try again shortly.',
        );
      const blob = await response.blob();
      if (blob.type !== 'image/png' || blob.size > 8 * 1024 * 1024)
        throw new Error('Invalid character image');
      if (controller.signal.aborted || !root.isConnected || this.disposed) return;
      this.releaseDebug();
      objectUrl = URL.createObjectURL(blob);
      this.debug = { url: objectUrl };
      const prepared = await prepareCharacter(objectUrl);
      if (controller.signal.aborted || !root.isConnected || this.disposed) return;
      this.debug.report = 'The image passed the sheet checks.';
      this.showDebug(root);
      this.pending = prepared;
      root.querySelector<HTMLElement>('.character-walk-preview')!.style.backgroundImage =
        `url('${prepared}')`;
      root.querySelector<HTMLElement>('#character-preview')!.hidden = false;
      status.textContent =
        'Ready! Check the poses, then choose your character or describe another.';
    } catch (error) {
      if (!controller.signal.aborted && root.isConnected) {
        if (error instanceof CharacterSheetError && this.debug) {
          this.debug.keyed = error.keyedPreview;
          this.debug.report = JSON.stringify(error.diagnostic, null, 2);
        }
        this.showDebug(root, true);
        status.textContent =
          error instanceof Error ? error.message : 'Could not create this character. Try again.';
      }
    } finally {
      if (objectUrl && (controller.signal.aborted || !root.isConnected || this.disposed))
        this.releaseDebug();
      if (this.request === controller) this.request = undefined;
      button.disabled = false;
    }
  }
  private async use(root: HTMLElement) {
    const pending = this.pending;
    if (!pending) return;
    try {
      await this.install(pending, () => root.isConnected && !!root.closest('dialog[open]'));
      if (!root.isConnected || this.disposed) return;
      this.saved = pending;
      try {
        localStorage.setItem(this.storageKey, pending);
      } catch {
        this.hooks.toast(
          'Your look is ready for this visit, but browser storage could not save it.',
        );
      }
      this.hooks.close();
    } catch {
      root.querySelector<HTMLElement>('#character-status')!.textContent =
        'Could not use this image. Please generate another character.';
    }
  }
}
