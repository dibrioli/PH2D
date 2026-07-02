// Left panel: HSV color wheel + brightness bar + palette, brush sliders,
// shape/preset buttons, tool buttons, tilt dial, paper presets, layers list.

import { hsvToRgb, rgbToHsv } from '../engine/colorops.js';
import { NAMED_PIGMENTS, TOOLS, addLayer, removeLayer, selectLayer, setPaper, markDirtyFull } from '../engine/painter.js';
import { t } from './i18n.js';

export function buildLeftPanel(app) {
  buildColorWheel(app);
  buildSliders(app);
  buildShapes(app);
  buildTools(app);
  buildTiltDial(app);
  buildPaper(app);
  buildLayers(app);
}

// ---------------------------------------------------------------------------
// Color: wheel (angle = hue, radius = saturation) + brightness bar
// ---------------------------------------------------------------------------

function buildColorWheel(app) {
  const { engine } = app;
  const wheel = document.getElementById('color-wheel');
  const bar = document.getElementById('bright-bar');
  const swatch = document.getElementById('current-swatch');
  const wctx = wheel.getContext('2d');
  const bctx = bar.getContext('2d');
  const size = wheel.width, rad = size / 2 - 2;
  let [hue, sat, val] = rgbToHsv(...engine.color);

  function drawWheel() {
    const img = wctx.createImageData(size, size);
    for (let y = 0; y < size; y++) {
      for (let x = 0; x < size; x++) {
        const dx = x - size / 2, dy = y - size / 2;
        const d = Math.sqrt(dx * dx + dy * dy);
        const o = (y * size + x) * 4;
        if (d > rad) { img.data[o + 3] = 0; continue; }
        const h = (Math.atan2(dy, dx) * 180) / Math.PI;
        const [r, g, b] = hsvToRgb(h, Math.min(1, d / rad), val);
        img.data[o] = r; img.data[o + 1] = g; img.data[o + 2] = b;
        img.data[o + 3] = d > rad - 1.5 ? (rad - d + 0.5) * 170 : 255;
      }
    }
    wctx.putImageData(img, 0, 0);
    // marker at the current hue/sat
    const a = (hue * Math.PI) / 180;
    const mx = size / 2 + Math.cos(a) * sat * rad;
    const my = size / 2 + Math.sin(a) * sat * rad;
    wctx.beginPath();
    wctx.arc(mx, my, 5, 0, Math.PI * 2);
    wctx.strokeStyle = val > 0.55 ? '#000' : '#fff';
    wctx.lineWidth = 2;
    wctx.stroke();
  }

  function drawBar() {
    const grad = bctx.createLinearGradient(0, 0, bar.width, 0);
    const [r0, g0, b0] = hsvToRgb(hue, sat, 0);
    const [r1, g1, b1] = hsvToRgb(hue, sat, 1);
    grad.addColorStop(0, `rgb(${r0 | 0},${g0 | 0},${b0 | 0})`);
    grad.addColorStop(1, `rgb(${r1 | 0},${g1 | 0},${b1 | 0})`);
    bctx.fillStyle = grad;
    bctx.fillRect(0, 0, bar.width, bar.height);
    const x = val * bar.width;
    bctx.fillStyle = '#fff';
    bctx.fillRect(x - 1.5, 0, 3, bar.height);
  }

  function commit() {
    engine.color = hsvToRgb(hue, sat, val);
    const [r, g, b] = engine.color;
    swatch.style.background = `rgb(${r | 0},${g | 0},${b | 0})`;
    drawWheel(); drawBar();
  }

  function pickWheel(ev) {
    const r = wheel.getBoundingClientRect();
    const dx = ev.clientX - r.left - size / 2;
    const dy = ev.clientY - r.top - size / 2;
    hue = ((Math.atan2(dy, dx) * 180) / Math.PI + 360) % 360;
    sat = Math.min(1, Math.sqrt(dx * dx + dy * dy) / rad);
    commit();
  }
  function pickBar(ev) {
    const r = bar.getBoundingClientRect();
    val = Math.max(0, Math.min(1, (ev.clientX - r.left) / r.width));
    commit();
  }
  dragify(wheel, pickWheel);
  dragify(bar, pickBar);

  // 8-swatch named-pigment palette (color-only presets).
  const pal = document.getElementById('palette');
  for (const p of NAMED_PIGMENTS) {
    const el = document.createElement('div');
    el.className = 'pal-swatch';
    el.title = p.name;
    el.style.background = `rgb(${p.rgb.join(',')})`;
    el.addEventListener('click', () => {
      [hue, sat, val] = rgbToHsv(...p.rgb); // palette picks move the wheel
      commit();
    });
    pal.appendChild(el);
  }
  commit();
}

function dragify(el, handler) {
  el.addEventListener('pointerdown', (ev) => {
    if (ev.button !== 0) return;
    el.setPointerCapture(ev.pointerId);
    handler(ev);
    const move = (e2) => handler(e2);
    const up = () => {
      el.removeEventListener('pointermove', move);
      el.removeEventListener('pointerup', up);
    };
    el.addEventListener('pointermove', move);
    el.addEventListener('pointerup', up);
  });
}

// ---------------------------------------------------------------------------
// Brush sliders
// ---------------------------------------------------------------------------

const SLIDERS = [
  ['size', 'brushSize', 0, 1],
  ['pressure', 'pressure', 0, 1],
  ['water', 'water', 0, 1],
  ['erase', 'eraseStrength', 0, 1],
];

function buildSliders(app) {
  const box = document.getElementById('slider-block');
  app.sliderEls = {};
  for (const [key, label] of SLIDERS) {
    const row = document.createElement('div');
    row.className = 'ctl-slider';
    const lab = document.createElement('label');
    const name = document.createElement('span');
    name.dataset.i18n = label;
    name.textContent = t(app.lang, label);
    const valEl = document.createElement('span');
    lab.append(name, valEl);
    const input = document.createElement('input');
    input.type = 'range';
    input.min = '0'; input.max = '1'; input.step = '0.01';
    input.value = String(app.engine.sliders[key]);
    valEl.textContent = Number(input.value).toFixed(2);
    input.addEventListener('input', () => {
      app.engine.sliders[key] = Number(input.value);
      valEl.textContent = Number(input.value).toFixed(2);
    });
    row.append(lab, input);
    box.appendChild(row);
    app.sliderEls[key] = { input, valEl };
  }
}

export function setSlider(app, key, v) {
  app.engine.sliders[key] = v;
  const el = app.sliderEls[key];
  if (el) { el.input.value = String(v); el.valEl.textContent = v.toFixed(2); }
}

// ---------------------------------------------------------------------------
// Shapes + wet/dry brush presets
// ---------------------------------------------------------------------------

function buildShapes(app) {
  const row = document.getElementById('shape-row');
  const shapes = [['round', 'round'], ['flat', 'flat'], ['fan', 'fan']];
  for (const [shape, label] of shapes) {
    const b = document.createElement('button');
    b.dataset.i18n = label;
    b.textContent = t(app.lang, label);
    if (shape === app.engine.shape) b.classList.add('on');
    b.addEventListener('click', () => {
      app.engine.shape = shape;
      for (const c of row.children) c.classList.remove('on');
      b.classList.add('on');
    });
    row.appendChild(b);
  }
  const presets = document.getElementById('preset-row');
  const wet = document.createElement('button');
  wet.dataset.i18n = 'wetBrush';
  wet.textContent = t(app.lang, 'wetBrush');
  wet.addEventListener('click', () => { setSlider(app, 'pressure', 0.35); setSlider(app, 'water', 0.85); });
  const dry = document.createElement('button');
  dry.dataset.i18n = 'dryBrush';
  dry.textContent = t(app.lang, 'dryBrush');
  dry.addEventListener('click', () => { setSlider(app, 'pressure', 0.85); setSlider(app, 'water', 0.2); });
  presets.append(wet, dry);
}

// ---------------------------------------------------------------------------
// Tools
// ---------------------------------------------------------------------------

function buildTools(app) {
  const row = document.getElementById('tool-row');
  app.toolButtons = {};
  for (const tool of TOOLS) {
    const b = document.createElement('button');
    b.dataset.i18n = tool;
    b.textContent = t(app.lang, tool);
    if (tool === app.engine.tool) b.classList.add('on');
    b.addEventListener('click', () => setTool(app, tool));
    row.appendChild(b);
    app.toolButtons[tool] = b;
  }
}

export function setTool(app, tool) {
  app.engine.tool = tool;
  for (const [name, b] of Object.entries(app.toolButtons)) {
    b.classList.toggle('on', name === tool);
  }
}

// ---------------------------------------------------------------------------
// Tilt dial: a round pad with a polar grid (8 rings x 12 spokes). Dragging
// the knob sets the gravity direction + magnitude snapped to the grid, and
// implicitly turns tilt on. Boot: ON, straight down, half radius (scale 1).
// ---------------------------------------------------------------------------

function buildTiltDial(app) {
  const { engine } = app;
  const dial = document.getElementById('tilt-dial');
  const toggle = document.getElementById('tilt-toggle');
  const ctx = dial.getContext('2d');
  const size = dial.width, c = size / 2, R = c - 6;
  let ring = 4; // 0..8 -> scale = ring/4 (boot = 1)
  let spoke = 3; // 12 spokes of 30deg; 3 = straight down

  function syncSim() {
    const a = (spoke * 30 * Math.PI) / 180;
    engine.sim.tiltDirX = Math.cos(a);
    engine.sim.tiltDirY = Math.sin(a);
    engine.sim.tiltScale = ring / 4; // ring 4 = the gravity knob's magnitude
  }

  function draw() {
    ctx.clearRect(0, 0, size, size);
    ctx.strokeStyle = '#3a3f48';
    ctx.lineWidth = 1;
    for (let k = 1; k <= 8; k++) {
      ctx.beginPath();
      ctx.arc(c, c, (R * k) / 8, 0, Math.PI * 2);
      ctx.stroke();
    }
    for (let k = 0; k < 12; k++) {
      const a = (k * 30 * Math.PI) / 180;
      ctx.beginPath();
      ctx.moveTo(c, c);
      ctx.lineTo(c + Math.cos(a) * R, c + Math.sin(a) * R);
      ctx.stroke();
    }
    const on = engine.sim.tiltOn;
    const a = (spoke * 30 * Math.PI) / 180;
    const kx = c + Math.cos(a) * (R * ring) / 8;
    const ky = c + Math.sin(a) * (R * ring) / 8;
    if (on && ring > 0) {
      ctx.strokeStyle = '#4c8ed9';
      ctx.lineWidth = 2;
      ctx.beginPath(); ctx.moveTo(c, c); ctx.lineTo(kx, ky); ctx.stroke();
    }
    ctx.beginPath();
    ctx.arc(kx, ky, 6, 0, Math.PI * 2);
    ctx.fillStyle = on ? '#6aa5e8' : '#555b64';
    ctx.fill();
    toggle.textContent = on ? t(app.lang, 'tiltOn') : t(app.lang, 'tiltOff');
    toggle.classList.toggle('on', on);
  }
  app.redrawTilt = draw;

  function pick(ev) {
    const r = dial.getBoundingClientRect();
    const dx = ev.clientX - r.left - c, dy = ev.clientY - r.top - c;
    const d = Math.sqrt(dx * dx + dy * dy);
    ring = Math.max(0, Math.min(8, Math.round((d / R) * 8)));         // snap ring
    spoke = ((Math.round(((Math.atan2(dy, dx) * 180) / Math.PI) / 30) % 12) + 12) % 12; // snap spoke
    engine.sim.tiltOn = true; // dragging the knob implies tilt on
    syncSim();
    draw();
  }
  dragify(dial, pick);
  toggle.addEventListener('click', () => {
    engine.sim.tiltOn = !engine.sim.tiltOn;
    draw();
  });
  syncSim();
  draw();
}

// ---------------------------------------------------------------------------
// Paper presets (double-click on the canvas cycles the 4 sheet indices)
// ---------------------------------------------------------------------------

const PAPERS = [['cold', 'coldPress'], ['rough', 'rough'], ['hot', 'hotPress']];

function buildPaper(app) {
  const row = document.getElementById('paper-row');
  for (const [preset, label] of PAPERS) {
    const b = document.createElement('button');
    b.dataset.i18n = label;
    b.textContent = t(app.lang, label);
    if (preset === app.engine.paperPreset) b.classList.add('on');
    b.addEventListener('click', () => {
      setPaper(app.engine, preset);
      for (const c of row.children) c.classList.remove('on');
      b.classList.add('on');
    });
    row.appendChild(b);
  }
}

// ---------------------------------------------------------------------------
// Layers
// ---------------------------------------------------------------------------

function buildLayers(app) {
  const { engine } = app;
  document.getElementById('layer-add').addEventListener('click', () => {
    if (addLayer(engine)) refreshLayers(app);
  });
  document.getElementById('layer-remove').addEventListener('click', () => {
    if (removeLayer(engine, engine.activeLayer)) refreshLayers(app);
  });
  refreshLayers(app);
}

export function refreshLayers(app) {
  const { engine } = app;
  const list = document.getElementById('layer-list');
  list.textContent = '';
  // Topmost layer first in the list.
  for (let idx = engine.layers.length - 1; idx >= 0; idx--) {
    const layer = engine.layers[idx];
    const item = document.createElement('div');
    item.className = 'layer-item' + (idx === engine.activeLayer ? ' active' : '');
    const eye = document.createElement('span');
    eye.className = 'eye' + (layer.visible ? '' : ' off');
    eye.textContent = '◉';
    eye.addEventListener('click', (ev) => {
      ev.stopPropagation();
      layer.visible = !layer.visible;
      markDirtyFull(engine);
      refreshLayers(app);
    });
    const name = document.createElement('span');
    name.className = 'lname';
    name.textContent = `${t(app.lang, 'layer')} ${idx + 1}`;
    const op = document.createElement('input');
    op.type = 'range';
    op.min = '0'; op.max = '1'; op.step = '0.01';
    op.value = String(layer.opacity);
    op.addEventListener('input', () => {
      layer.opacity = Number(op.value);
      markDirtyFull(engine);
    });
    op.addEventListener('pointerdown', (ev) => ev.stopPropagation());
    item.append(eye, name, op);
    item.addEventListener('click', () => {
      selectLayer(engine, idx);
      refreshLayers(app);
    });
    list.appendChild(item);
  }
}
