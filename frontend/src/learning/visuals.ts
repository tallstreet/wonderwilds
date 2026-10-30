import { spriteMarkup, propFrames, heroFrames } from '../sprites.ts';

/** Only fixed local art is markup; content labels always use textContent. */
export function art(document: Document, name: string, kind: 'prop' | 'hero' = 'prop') {
  const node = document.createElement('span');
  node.className = `ww-art ww-art-${name}`;
  node.setAttribute('aria-hidden', 'true');
  node.innerHTML = spriteMarkup(
    Math.max(0, (kind === 'prop' ? propFrames : heroFrames).indexOf(name)),
    kind,
    84,
  );
  return node;
}
const tree =
  '<path d="M120 116V60" stroke="#9d704a" stroke-width="12" stroke-linecap="round"/><path d="M120 85l-19-15m19 4 18-17" stroke="#9d704a" stroke-width="6"/><circle cx="103" cy="48" r="24" fill="#77b968"/><circle cx="134" cy="43" r="28" fill="#8dca77"/><circle cx="119" cy="23" r="23" fill="#9dd681"/>';
const cup = (x: number, y: number, color = '#e58f96') =>
  `<path d="M${x + 26} ${y + 7}q23-6 17 13q-3 8-18 5" fill="none" stroke="${color}" stroke-width="7"/><path d="M${x} ${y}h30l-3 36q-12 8-24 0z" fill="${color}"/><path d="M${x + 5} ${y + 7}v18" stroke="#ffffff" opacity=".5" stroke-width="4" stroke-linecap="round"/>`;
export function picture(
  document: Document,
  label: string,
  length?: number,
  maximumLength = 100,
): HTMLElement {
  const node = document.createElement('span');
  node.className = 'ww-picture';
  node.setAttribute('aria-hidden', 'true');
  const text = label.toLowerCase();
  if (/^\d+$/.test(label)) {
    const number = document.createElement('strong');
    number.textContent = label;
    number.className = 'ww-number';
    node.append(number);
    const dots = document.createElement('span');
    dots.className = 'ww-count-dots';
    for (let i = 0; i < (Number(label) <= 20 ? Number(label) : 0); i++) {
      const dot = document.createElement('i');
      dots.append(dot);
    }
    node.append(dots);
    return node;
  }
  if (length !== undefined) {
    node.classList.add('ww-rope-picture');
    const rope = document.createElement('span');
    rope.className = 'ww-rope';
    rope.style.width = `${Math.max(2, Math.min(100, (length / maximumLength) * 100))}%`;
    node.append(rope);
    const units = document.createElement('span');
    units.textContent = `${length}`;
    node.append(units);
    return node;
  }
  if (text === 'leaf') {
    node.innerHTML =
      '<svg viewBox="0 0 180 140"><path d="M35 105Q20 20 147 20Q160 115 35 105" fill="#8ac276" stroke="#4b9460" stroke-width="4"/><path d="M30 116L126 40m-55 43-7-28m31 7 29 2" stroke="#d9edaf" stroke-width="5" stroke-linecap="round"/></svg>';
    return node;
  }
  if (text === 'flower') {
    node.append(art(document, 'flower'));
    return node;
  }
  if (text === 'berry') {
    node.append(art(document, 'berry'));
    return node;
  }
  if (text.length === 1) {
    const letter = document.createElement('strong');
    letter.className = 'ww-letter';
    letter.textContent = label;
    node.append(letter);
    return node;
  }
  let drawing = '';
  if (text.includes('seed')) {
    if (text.includes('find'))
      drawing =
        '<ellipse cx="88" cy="87" rx="22" ry="15" transform="rotate(-28 88 87)" fill="#b58a56"/><path d="M80 84q10-9 17-7" fill="none" stroke="#e5c99d" stroke-width="5"/><path d="M45 45v15m-8-8h16m70-20v16m-8-8h16" stroke="#e7bc66" stroke-width="5" stroke-linecap="round"/>';
    else
      drawing =
        '<path d="M43 93h94l-15 38H58z" fill="#d99473"/><path d="M40 87h100v15H40z" fill="#efb28c"/><ellipse cx="90" cy="87" rx="37" ry="9" fill="#845f48"/><path d="M90 87V55" stroke="#5d9e68" stroke-width="6"/><path d="M90 66Q50 65 62 39Q92 36 90 66m0-9q35-35 40-9q-1 21-40 9" fill="#94c975"/>' +
        (text.includes('water')
          ? '<path d="M37 20l-6 13q0 10 10 8q9-3-4-21m24-8-5 12q-2 9 7 8q10-1-2-20m-36 35-6 12q-1 9 8 8q8-2-2-20" fill="#89cce1"/>'
          : '');
  } else if (text.includes('inside') || text === 'in')
    drawing =
      '<path d="M46 70l40-18 48 19-43 24z" fill="#dcae7b"/>' +
      cup(73, 48) +
      '<path d="M46 70l45 17v46l-45-18z" fill="#c99668"/><path d="M91 87l43-16v45l-43 17z" fill="#deb88c"/>';
  else if (text === 'above')
    drawing = '<g transform="translate(0 35) scale(.75)">' + tree + '</g>' + cup(78, 2);
  else if (text === 'below')
    drawing = '<g transform="translate(0 -8) scale(.75)">' + tree + '</g>' + cup(78, 88);
  else if (text === 'cup') drawing = cup(65, 60, '#e2858e');
  else if (text.includes('beside') || text.includes('tree')) drawing = tree + cup(45, 82);
  else if (text.includes('red cup')) drawing = cup(65, 60, '#e2858e');
  else if (text.includes('hat'))
    drawing =
      '<path d="M49 95q7-61 42-62q35 1 41 62" fill="#8ab5dc"/><ellipse cx="91" cy="98" rx="62" ry="15" fill="#7298c2"/><path d="M54 85h75" stroke="#c0dcf1" stroke-width="12"/>';
  else if (
    text.includes('round') ||
    text.includes('circle') ||
    text === 'stone' ||
    text === 'shell'
  )
    drawing =
      '<circle cx="90" cy="71" r="45" fill="#ebbb79" stroke="#d49b60" stroke-width="5"/><ellipse cx="74" cy="53" rx="20" ry="11" fill="#ffdfaa" transform="rotate(-30 74 53)"/>';
  else if (text.includes('square'))
    drawing =
      '<rect x="44" y="24" width="92" height="92" rx="12" fill="#a49ad9" stroke="#8478bb" stroke-width="5"/><path d="M59 44h55" stroke="#d6cef5" stroke-width="8" stroke-linecap="round"/>';
  else if (text.includes('triangle'))
    drawing =
      '<path d="M90 20l57 98H33z" fill="#8fc4ca" stroke="#69a4b1" stroke-width="5" stroke-linejoin="round"/>';
  else if (text.includes('star'))
    drawing =
      '<path d="M90 12l20 37 43 8-30 32 6 43-39-20-39 20 7-43-31-32 43-8z" fill="#f3ce79" stroke="#ddae5a" stroke-width="4" stroke-linejoin="round"/>';
  if (drawing)
    node.innerHTML = `<svg viewBox="0 0 180 140" focusable="false"><ellipse cx="90" cy="126" rx="61" ry="7" fill="#4b6855" opacity=".1"/>${drawing}</svg>`;
  else {
    const caption = document.createElement('span');
    caption.className = 'ww-fallback-picture';
    caption.textContent = label;
    node.append(caption);
  }
  return node;
}
