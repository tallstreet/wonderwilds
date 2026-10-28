/** Pointer gestures work with mouse, pen and touch; native buttons remain keyboard-operable. */
export function pointerMoves(root: HTMLElement, enabled: () => boolean) {
  const document = root.ownerDocument;
  const targets = new Map<HTMLElement, (key: string) => void>();
  let active:
    | {
        pointer: number;
        key: string;
        source: HTMLElement;
        x: number;
        y: number;
        ghost?: HTMLElement;
      }
    | undefined;
  let suppress: HTMLElement | undefined;
  let suppressedAt = 0;
  const cancel = () => {
    active?.ghost?.remove();
    active?.source.classList.remove('ww-dragging');
    for (const target of targets.keys()) target.classList.remove('ww-drop-hover');
    active = undefined;
  };
  const move = (event: PointerEvent) => {
    if (!active || event.pointerId !== active.pointer) return;
    if (!enabled()) {
      cancel();
      return;
    }
    if (!active.ghost && Math.hypot(event.clientX - active.x, event.clientY - active.y) > 8) {
      active.ghost = active.source.cloneNode(true) as HTMLElement;
      active.ghost.removeAttribute('id');
      active.ghost.setAttribute('aria-hidden', 'true');
      active.ghost.classList.add('ww-drag-ghost');
      const rect = active.source.getBoundingClientRect();
      active.ghost.style.width = `${rect.width}px`;
      active.ghost.style.height = `${rect.height}px`;
      root.append(active.ghost);
      active.source.classList.add('ww-dragging');
    }
    if (active.ghost) {
      event.preventDefault();
      active.ghost.style.left = `${event.clientX}px`;
      active.ghost.style.top = `${event.clientY}px`;
      const hit = document.elementFromPoint(event.clientX, event.clientY);
      for (const target of targets.keys())
        target.classList.toggle('ww-drop-hover', Boolean(hit && target.contains(hit)));
    }
  };
  const end = (event: PointerEvent) => {
    if (!active || event.pointerId !== active.pointer) return;
    const item = active;
    if (item.ghost) {
      suppress = item.source;
      suppressedAt = Date.now();
      const hit = document.elementFromPoint(event.clientX, event.clientY);
      const destination = [...targets].find(([target]) => hit && target.contains(hit));
      cancel();
      if (enabled()) destination?.[1](item.key);
    } else cancel();
  };
  const click = (event: MouseEvent) => {
    if (
      suppress &&
      Date.now() - suppressedAt < 500 &&
      event.target instanceof Node &&
      suppress.contains(event.target)
    ) {
      event.preventDefault();
      event.stopImmediatePropagation();
      suppress = undefined;
    }
  };
  root.addEventListener('pointermove', move);
  root.addEventListener('pointerup', end);
  root.addEventListener('pointercancel', cancel);
  root.addEventListener('click', click, true);
  return {
    source(node: HTMLElement, key: string) {
      node.classList.add('ww-movable');
      node.addEventListener('pointerdown', (event) => {
        // A fresh press is an intentional tap, not the synthetic click after a drag.
        suppress = undefined;
        if (
          event.button !== 0 ||
          !enabled() ||
          (node instanceof HTMLButtonElement && node.disabled)
        )
          return;
        cancel();
        active = {
          pointer: event.pointerId,
          key,
          source: node,
          x: event.clientX,
          y: event.clientY,
        };
        node.setPointerCapture(event.pointerId);
      });
    },
    target(node: HTMLElement, drop: (key: string) => void) {
      targets.set(node, drop);
      node.classList.add('ww-dropzone');
    },
    cancel,
    destroy() {
      cancel();
      targets.clear();
      root.removeEventListener('pointermove', move);
      root.removeEventListener('pointerup', end);
      root.removeEventListener('pointercancel', cancel);
      root.removeEventListener('click', click, true);
    },
  };
}
