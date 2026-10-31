import type { QuestionStimulus } from '../../../contracts/generated/contracts.ts';
import { activityIcon } from './icons.ts';
import audioManifest from '../../../content/activities/audio-v1.json';

/** Bounded semantic diagrams, never server HTML or a server-provided SVG string. */
export function renderStimulus(document: Document, stimulus: QuestionStimulus): HTMLElement {
  const root = document.createElement('div');
  root.className = `ww-question-stimulus ww-question-${stimulus.kind}`;
  const text = (tag: string, value: string) => {
    const e = document.createElement(tag);
    e.textContent = value;
    return e;
  };
  const svg = () => {
    const e = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    e.setAttribute('viewBox', '0 0 300 200');
    e.setAttribute('aria-hidden', 'true');
    return e;
  };
  const draw = (parent: SVGElement, tag: string, attributes: Record<string, string | number>) => {
    const e = document.createElementNS('http://www.w3.org/2000/svg', tag);
    Object.entries(attributes).forEach(([k, v]) => e.setAttribute(k, String(v)));
    parent.append(e);
    return e;
  };
  switch (stimulus.kind) {
    case 'audio': {
      const manifest: { assets: { id: string; path: string; reviewStatus: string }[] } =
        audioManifest;
      const asset = manifest.assets.find(
        (a) =>
          a.id === stimulus.assetId &&
          a.reviewStatus === 'reviewed' &&
          /^\/assets\/learning\/audio\/[a-z0-9-]+\.(wav|mp3|ogg)$/.test(a.path),
      );
      if (!asset) {
        root.append(text('p', 'This recording is not ready. Explore the sounds with a grown-up.'));
        break;
      }
      const audio = document.createElement('audio');
      audio.controls = true;
      audio.preload = 'none';
      audio.src = asset.path;
      audio.setAttribute('aria-label', 'Listen to the question recording');
      root.append(audio);
      break;
    }
    case 'text':
    case 'passage':
      root.append(text('p', stimulus.text));
      break;
    case 'picture':
      root.append(activityIcon(stimulus.name, stimulus.description));
      break;
    case 'table': {
      const table = document.createElement('table');
      const head = document.createElement('tr');
      stimulus.headings.forEach((label) => {
        const th = text('th', label);
        th.setAttribute('scope', 'col');
        head.append(th);
      });
      const thead = document.createElement('thead');
      thead.append(head);
      table.append(thead);
      const body = document.createElement('tbody');
      stimulus.rows.forEach((row) => {
        const tr = document.createElement('tr');
        row.forEach((value) => tr.append(text('td', value)));
        body.append(tr);
      });
      table.append(body);
      root.append(table);
      break;
    }
    case 'collection': {
      stimulus.counts.forEach((count, i) => {
        const section = document.createElement('section');
        section.append(text('h3', stimulus.labels[i] ?? 'Objects'));
        const group = document.createElement('div');
        group.className = `ww-question-dots ${stimulus.layout}`;
        group.setAttribute('role', 'group');
        group.setAttribute('aria-label', `${count} objects`);
        const total =
          stimulus.layout === 'ten_frame' ? Math.ceil(Math.max(1, count) / 10) * 10 : count;
        for (let j = 0; j < total; j++) {
          const dot = document.createElement('span');
          dot.className = j < count ? 'dot filled' : 'dot';
          dot.setAttribute('aria-hidden', 'true');
          if (j < count) dot.append(activityIcon('berry'));
          group.append(dot);
        }
        section.append(group);
        root.append(section);
      });
      break;
    }
    case 'clock': {
      const canvas = svg();
      draw(canvas, 'circle', {
        cx: 150,
        cy: 100,
        r: 85,
        fill: '#fffcf4',
        stroke: '#697c5d',
        'stroke-width': 3,
      });
      for (let i = 1; i <= 12; i++) {
        const angle = (i * Math.PI) / 6;
        const n = draw(canvas, 'text', {
          x: 150 + 67 * Math.sin(angle),
          y: 105 - 67 * Math.cos(angle),
          'text-anchor': 'middle',
          fill: '#35403a',
          'font-size': 14,
        });
        n.textContent = String(i);
      }
      const hand = (angle: number, length: number, width: number) =>
        draw(canvas, 'line', {
          x1: 150,
          y1: 100,
          x2: 150 + length * Math.sin(angle),
          y2: 100 - length * Math.cos(angle),
          stroke: '#465d38',
          'stroke-width': width,
          'stroke-linecap': 'round',
        });
      hand((((stimulus.hour % 12) + stimulus.minute / 60) * Math.PI) / 6, 43, 6);
      hand((stimulus.minute * Math.PI) / 30, 61, 3);
      draw(canvas, 'circle', { cx: 150, cy: 100, r: 5, fill: '#77619a' });
      root.append(canvas);
      root.setAttribute('role', 'img');
      root.setAttribute(
        'aria-label',
        `Clock. Minute hand points to ${stimulus.minute === 0 ? 12 : stimulus.minute / 5}. Hour hand is ${stimulus.minute === 0 ? 'at' : 'after'} ${stimulus.hour}.`,
      );
      break;
    }
    case 'fraction': {
      root.setAttribute('role', 'img');
      root.setAttribute(
        'aria-label',
        `${stimulus.numerator} of ${stimulus.denominator} equal parts shaded`,
      );
      const pieces = document.createElement('div');
      pieces.className = 'ww-question-fraction-parts';
      pieces.style.gridTemplateColumns = `repeat(${stimulus.denominator},1fr)`;
      for (let i = 0; i < Math.max(stimulus.denominator, stimulus.numerator); i++) {
        const piece = document.createElement('span');
        piece.className = i < stimulus.numerator ? 'shaded' : '';
        piece.textContent = i < stimulus.numerator ? '●' : '';
        pieces.append(piece);
      }
      root.append(pieces);
      break;
    }
    case 'fraction_line': {
      const canvas = svg();
      const d = stimulus.denominator;
      const max = Math.max(1, Math.ceil(stimulus.numerator / d));
      draw(canvas, 'line', {
        x1: 30,
        x2: 270,
        y1: 100,
        y2: 100,
        stroke: '#65745d',
        'stroke-width': 3,
      });
      for (let i = 0; i <= d * max; i++) {
        const x = 30 + (i / (d * max)) * 240;
        draw(canvas, 'line', {
          x1: x,
          x2: x,
          y1: 92,
          y2: 108,
          stroke: '#65745d',
          'stroke-width': 2,
        });
        if (i % d === 0)
          draw(canvas, 'text', {
            x,
            y: 132,
            'text-anchor': 'middle',
            'font-size': 16,
          }).textContent = String(i / d);
      }
      draw(canvas, 'circle', {
        cx: 30 + (stimulus.numerator / (d * max)) * 240,
        cy: 100,
        r: 7,
        fill: '#77619a',
      });
      root.append(canvas);
      root.setAttribute('role', 'img');
      root.setAttribute(
        'aria-label',
        `Number line divided into ${d} equal intervals per whole. The point is ${stimulus.numerator} intervals from zero.`,
      );
      break;
    }
    case 'composite': {
      const canvas = svg();
      const cx = 250 - (stimulus.cut / stimulus.width) * 200;
      const cy = 30 + (stimulus.cut / stimulus.height) * 140;
      draw(canvas, 'path', {
        d: `M50 30H${cx}V${cy}H250V170H50Z`,
        fill: '#e5edd9',
        stroke: '#6b8956',
        'stroke-width': 3,
      });
      root.append(
        canvas,
        text(
          'p',
          `Outer width ${stimulus.width} cm; outer height ${stimulus.height} cm. Top-right corner removed: ${stimulus.cut} cm × ${stimulus.cut} cm. Diagram not to scale.`,
        ),
      );
      break;
    }
    case 'prism': {
      const canvas = svg();
      draw(canvas, 'path', {
        d: 'M60 65L115 25H250V130L195 175H60Z',
        fill: '#e7e1f0',
        stroke: '#77619a',
        'stroke-width': 3,
      });
      draw(canvas, 'path', {
        d: 'M60 65H195V175M195 65L250 25',
        fill: 'none',
        stroke: '#77619a',
        'stroke-width': 3,
      });
      root.append(
        canvas,
        text(
          'p',
          `Length ${stimulus.length} cm, width ${stimulus.width} cm, height ${stimulus.height} cm. Diagram not to scale.`,
        ),
      );
      break;
    }
    case 'place_value': {
      if (stimulus.blocks) {
        const value = Number(stimulus.value);
        for (const [label, count, style] of [
          ['Hundreds', Math.floor(value / 100), 'hundreds'],
          ['Tens', Math.floor(value / 10) % 10, 'tens'],
          ['Ones', value % 10, 'ones'],
        ] as const) {
          const section = text('section', '');
          section.append(text('h3', label));
          const blocks = text('div', '');
          blocks.className = 'ww-place-blocks';
          blocks.setAttribute('role', 'img');
          blocks.setAttribute('aria-label', `${count} ${label.toLowerCase()}`);
          for (let i = 0; i < count; i++) {
            const block = text('span', '');
            block.className = style;
            blocks.append(block);
          }
          section.append(blocks);
          root.append(section);
        }
        break;
      }
      const [whole, decimal] = stimulus.value.split('.');
      const labels = decimal
        ? ['Ones', ...['Tenths', 'Hundredths', 'Thousandths'].slice(0, decimal.length)]
        : [
            'Millions',
            'Hundred thousands',
            'Ten thousands',
            'Thousands',
            'Hundreds',
            'Tens',
            'Ones',
          ].slice(-whole!.length);
      const digits = decimal ? [whole!, ...decimal] : [...whole!];
      labels.forEach((label, i) => {
        const cell = text('div', '');
        cell.append(text('small', label), text('strong', digits[i] ?? '0'));
        root.append(cell);
      });
      break;
    }
    case 'measure': {
      const max = Math.max(...stimulus.lengths);
      stimulus.lengths.forEach((length, i) => {
        const row = text('div', '');
        row.className = 'ww-question-measure-row';
        row.append(text('strong', `${stimulus.labels[i]} · ${length} ${stimulus.unit}`));
        const bar = text('span', '');
        bar.style.width = `${Math.max(8, (length / max) * 100)}%`;
        row.append(bar);
        root.append(row);
      });
      break;
    }
    case 'graph': {
      const canvas = svg();
      const max = Math.max(stimulus.scale, ...stimulus.values);
      const top = Math.ceil(max / stimulus.scale) * stimulus.scale;
      const baseline = stimulus.baseline ?? 0;
      const range = Math.max(1, top - baseline);
      draw(canvas, 'path', {
        d: 'M35 10V165H290',
        fill: 'none',
        stroke: '#65745d',
        'stroke-width': 2,
      });
      for (let i = 0; i <= Math.min(10, top / stimulus.scale); i++) {
        const value =
          baseline +
          i * Math.max(stimulus.scale, Math.ceil(top / 10 / stimulus.scale) * stimulus.scale);
        if (value > top) break;
        const y = 165 - ((value - baseline) / range) * 150;
        draw(canvas, 'line', { x1: 35, x2: 290, y1: y, y2: y, stroke: '#e3e7dc' });
        draw(canvas, 'text', {
          x: 30,
          y: y + 4,
          'text-anchor': 'end',
          'font-size': 10,
          fill: '#35403a',
        }).textContent = String(value);
      }
      const points = stimulus.values.map((v, i) => [
        50 + (i * 230) / Math.max(1, stimulus.values.length - 1),
        165 - ((v - baseline) / range) * 150,
      ]);
      points.forEach(([x, y], i) => {
        if (stimulus.style === 'bar')
          draw(canvas, 'rect', {
            x: x! - 8,
            y: y!,
            width: 16,
            height: 165 - y!,
            fill: '#91b477',
            rx: 3,
          });
        else draw(canvas, 'circle', { cx: x!, cy: y!, r: 4, fill: '#77619a' });
        draw(canvas, 'text', {
          x: x!,
          y: 183,
          'text-anchor': 'middle',
          'font-size': 9,
          fill: '#35403a',
        }).textContent = stimulus.labels[i]!;
      });
      if (stimulus.style === 'line')
        draw(canvas, 'polyline', {
          points: points.map((p) => p.join(',')).join(' '),
          stroke: '#77619a',
          'stroke-width': 2,
          fill: 'none',
        });
      root.append(
        canvas,
        renderStimulus(document, {
          kind: 'table',
          headings: ['Category', 'Value'],
          rows: stimulus.labels.map((label, i) => [label, String(stimulus.values[i])]),
        }),
      );
      break;
    }
    case 'grid': {
      const canvas = svg();
      const unit = 150 / stimulus.size;
      for (let i = 0; i <= stimulus.size; i++) {
        draw(canvas, 'line', {
          x1: 60 + i * unit,
          x2: 60 + i * unit,
          y1: 15,
          y2: 165,
          stroke: '#c8d8bd',
        });
        draw(canvas, 'line', {
          x1: 60,
          x2: 210,
          y1: 15 + i * unit,
          y2: 15 + i * unit,
          stroke: '#c8d8bd',
        });
        draw(canvas, 'text', {
          x: 60 + i * unit,
          y: 182,
          'text-anchor': 'middle',
          'font-size': 10,
        }).textContent = String(i);
        draw(canvas, 'text', {
          x: 48,
          y: 169 - i * unit,
          'text-anchor': 'end',
          'font-size': 10,
        }).textContent = String(i);
      }
      draw(canvas, 'circle', {
        cx: 60 + stimulus.x * unit,
        cy: 165 - stimulus.y * unit,
        r: 6,
        fill: '#77619a',
      });
      root.append(canvas);
      root.setAttribute('role', 'img');
      root.setAttribute(
        'aria-label',
        `Grid with ${stimulus.size} intervals per axis. Point is ${stimulus.x} across and ${stimulus.y} up.`,
      );
      break;
    }
    case 'angle': {
      const canvas = svg();
      const a = stimulus.values[0]!;
      if (stimulus.triangle)
        draw(canvas, 'path', {
          d: 'M60 160L230 160L130 30Z',
          fill: '#f0edf7',
          stroke: '#77619a',
          'stroke-width': 3,
        });
      else
        draw(canvas, 'path', {
          d: `M40 150H260M150 150L${150 + 100 * Math.cos((a * Math.PI) / 180)} ${150 - 100 * Math.sin((a * Math.PI) / 180)}`,
          fill: 'none',
          stroke: '#77619a',
          'stroke-width': 3,
        });
      root.append(
        canvas,
        text(
          'p',
          `${stimulus.values.map((v) => `${v}°`).join(', ')} and one unknown angle. Diagram not to scale.`,
        ),
      );
      break;
    }
    case 'number_line': {
      root.append(text('p', stimulus.labels.join(' — ')));
      root.setAttribute('aria-label', `Number line from ${stimulus.min} to ${stimulus.max}`);
      break;
    }
  }
  return root;
}
