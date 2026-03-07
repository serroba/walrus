import fs from 'node:fs';
import path from 'node:path';

const inputDir = path.resolve('outputs/latest');
const outDir = path.resolve('docs/assets');
fs.mkdirSync(outDir, { recursive: true });

const files = [
  'timeline_baseline_default.csv',
  'timeline_eco-stress_fragile.csv',
  'timeline_fragmented-low-coupling_default.csv',
];

const palette = {
  bg: '#fffdf7',
  border: '#d8d1c0',
  grid: '#ece5d7',
  text: '#1f1f1f',
  muted: '#666055',
  so: '#1b7f79',
  cx: '#cf5c36',
};

function parseCsv(csv) {
  const lines = csv.trim().split('\n');
  const rows = [];
  for (let i = 1; i < lines.length; i += 1) {
    const [tick, so, cx, h, s, a] = lines[i].split(',');
    rows.push({
      tick: Number(tick),
      so: Number(so),
      cx: Number(cx),
      h: Number(h),
      s: Number(s),
      a: Number(a),
    });
  }
  return rows;
}

function linePath(points, valueKey, x, y) {
  return points
    .map((p, i) => `${i === 0 ? 'M' : 'L'} ${x(p.tick).toFixed(2)} ${y(p[valueKey]).toFixed(2)}`)
    .join(' ');
}

function render(title, rows) {
  const w = 1100;
  const h = 520;
  const m = { t: 70, r: 30, b: 70, l: 70 };
  const iw = w - m.l - m.r;
  const ih = h - m.t - m.b;

  const maxTick = rows[rows.length - 1]?.tick ?? 1;
  const x = (t) => m.l + (t / maxTick) * iw;
  const y = (v) => m.t + (1 - v) * ih;

  const soPath = linePath(rows, 'so', x, y);
  const cxPath = linePath(rows, 'cx', x, y);
  const last = rows[rows.length - 1] ?? { so: 0, cx: 0, h: 0, s: 0, a: 0 };

  const hbars = [
    { k: 'H', v: last.h, c: '#637d5f' },
    { k: 'S', v: last.s, c: '#9e7c46' },
    { k: 'A', v: last.a, c: '#8a5a44' },
  ];
  const maxBar = Math.max(1, ...hbars.map((b) => b.v));

  const bars = hbars
    .map((b, i) => {
      const bw = 60;
      const gap = 22;
      const baseX = w - 260 + i * (bw + gap);
      const bh = (b.v / maxBar) * 100;
      const by = h - 120 - bh;
      return `
        <rect x="${baseX}" y="${by}" width="${bw}" height="${bh}" fill="${b.c}" rx="6" />
        <text x="${baseX + bw / 2}" y="${h - 96}" text-anchor="middle" font-size="14" fill="${palette.muted}">${b.k}</text>
        <text x="${baseX + bw / 2}" y="${by - 6}" text-anchor="middle" font-size="12" fill="${palette.text}">${b.v}</text>
      `;
    })
    .join('\n');

  return `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}">
  <rect x="0" y="0" width="${w}" height="${h}" fill="${palette.bg}" />
  <rect x="18" y="18" width="${w - 36}" height="${h - 36}" fill="none" stroke="${palette.border}" rx="12" />

  <text x="40" y="48" font-size="28" font-family="Georgia, serif" fill="${palette.text}">${title}</text>
  <text x="40" y="74" font-size="14" font-family="Georgia, serif" fill="${palette.muted}">Superorganism Index vs Mean Local Complexity over time</text>

  <line x1="${m.l}" y1="${m.t}" x2="${m.l}" y2="${h - m.b}" stroke="${palette.grid}" />
  <line x1="${m.l}" y1="${h - m.b}" x2="${w - m.r}" y2="${h - m.b}" stroke="${palette.grid}" />

  <path d="${soPath}" fill="none" stroke="${palette.so}" stroke-width="3" />
  <path d="${cxPath}" fill="none" stroke="${palette.cx}" stroke-width="3" />

  <circle cx="${w - 330}" cy="48" r="5" fill="${palette.so}" /><text x="${w - 318}" y="53" font-size="13" fill="${palette.muted}">Superorganism</text>
  <circle cx="${w - 200}" cy="48" r="5" fill="${palette.cx}" /><text x="${w - 188}" y="53" font-size="13" fill="${palette.muted}">Complexity</text>

  <text x="${m.l}" y="${h - 26}" font-size="12" fill="${palette.muted}">Tick 0</text>
  <text x="${w - m.r - 30}" y="${h - 26}" font-size="12" fill="${palette.muted}">Tick ${maxTick}</text>

  <text x="${w - 250}" y="${h - 140}" font-size="12" fill="${palette.muted}">Final modes (H/S/A)</text>
  ${bars}
</svg>`;
}

for (const filename of files) {
  const inPath = path.join(inputDir, filename);
  if (!fs.existsSync(inPath)) {
    console.error(`Missing input: ${inPath}`);
    process.exitCode = 1;
    continue;
  }
  const rows = parseCsv(fs.readFileSync(inPath, 'utf8'));
  const title = filename.replace('timeline_', '').replace('.csv', '').replaceAll('_', ' ');
  const svg = render(title, rows);
  const outName = filename.replace('.csv', '.svg').replace('timeline_', 'snapshot_');
  fs.writeFileSync(path.join(outDir, outName), svg, 'utf8');
  console.log(`Wrote docs/assets/${outName}`);
}
