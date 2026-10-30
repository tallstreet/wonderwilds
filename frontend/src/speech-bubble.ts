import type { Point, WorldEncounter } from './types.ts';

/** Non-modal dialogue: never captures movement or acknowledges a story action. */
export class SpeechBubble {
  private element = document.createElement('aside');
  private anchor?: WorldEncounter;
  private origin?: Point;
  private escape = (event: KeyboardEvent) => {
    if (event.key === 'Escape') this.clear();
  };
  constructor(parent: HTMLElement) {
    this.element.className = 'world-speech';
    this.element.hidden = true;
    // Keep touch scrolling outside the canvas surface, which disables touch gestures.
    (parent.parentElement ?? parent).append(this.element);
    document.addEventListener('keydown', this.escape);
  }
  show(marker: WorldEncounter, texts: string[], origin: Point) {
    this.anchor = marker;
    this.origin = { ...origin };
    const heading = document.createElement('strong');
    heading.textContent = marker.character;
    const content = document.createElement('div');
    content.setAttribute('data-story-dialogue', '');
    content.setAttribute('role', 'status');
    content.setAttribute('aria-live', 'polite');
    content.append(heading);
    for (const text of new Set(texts)) {
      const paragraph = document.createElement('p');
      paragraph.textContent = text;
      content.append(paragraph);
    }
    const close = document.createElement('button');
    close.className = 'speech-dismiss';
    close.setAttribute('aria-label', 'Dismiss speech');
    close.textContent = '×';
    close.onclick = () => this.clear();
    this.element.replaceChildren(content, close);
    this.element.hidden = false;
  }
  update(player: Point, region: string, project: (point: Point) => Point) {
    if (!this.anchor || !this.origin) return;
    if (
      document.querySelector('dialog[open]') ||
      region !== this.anchor.region ||
      Math.hypot(player.x - this.origin.x, player.y - this.origin.y) > 100
    ) {
      this.clear();
      return;
    }
    const point = project({ x: this.anchor.x, y: this.anchor.y - 100 });
    const width = this.element.offsetWidth;
    const left = Math.max(12, Math.min(innerWidth - width - 12, point.x - width / 2));
    this.element.style.left = `${left}px`;
    this.element.style.top = `${Math.max(90, Math.min(innerHeight - this.element.offsetHeight - 110, point.y - this.element.offsetHeight - 12))}px`;
    this.element.style.setProperty(
      '--speech-tail',
      `${Math.max(24, Math.min(width - 24, point.x - left))}px`,
    );
  }
  clear() {
    this.element.hidden = true;
    this.anchor = undefined;
  }
  destroy() {
    document.removeEventListener('keydown', this.escape);
    this.element.remove();
  }
}
