// World coordinates measured from the bridge-frame's trimmed 470 x 270 atlas
// rectangle, displayed at 174 x 88. Only the opening between rails is walkable.
// Keep scenery and navigation on the same geometry rather than sprite centres.
export const BRIDGE = Object.freeze({
  x: 690,
  y: 357,
  width: 174,
  height: 88,
  deckTop: 346,
  deckBottom: 372,
});
// A small footprint around the midpoint between planted feet. Head, cape,
// tail, and animation lift never participate in collisions.
export const FOOTPRINT = Object.freeze({ halfWidth: 12, halfHeight: 4 });
export function bridgeContainsFeet(y: number) {
  return (
    y - FOOTPRINT.halfHeight >= BRIDGE.deckTop && y + FOOTPRINT.halfHeight <= BRIDGE.deckBottom
  );
}
