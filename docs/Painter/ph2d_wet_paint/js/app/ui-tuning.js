// Right panel: the live tuning panel (spec section 16) and the experimental
// panel (the two K-M checkboxes). Built entirely from the tuning registry -
// per-knob slider + free numeric input + reset (red-tinted when off-default),
// per-group reset, rich PT-BR hover tooltips with correlation letters, and
// the render-only paper-appearance master in the paper group header.

import { KNOB_GROUPS } from '../engine/tuning.js';
import { rebakePaper, markDirtyFull } from '../engine/painter.js';
import { KNOB_DOCS, CORR_LABELS } from './knobdocs.js';
import { GROUP_LABELS, t } from './i18n.js';

export function buildTuningPanel(app) {
  const { engine } = app;
  const panel = document.getElementById('tuning-panel');
  panel.textContent = '';
  app.knobEls = {};

  for (const group of KNOB_GROUPS) {
    const box = document.createElement('div');
    box.className = 'tune-group';
    const head = document.createElement('div');
    head.className = 'tune-head';
    const title = document.createElement('span');
    title.textContent = GROUP_LABELS[app.lang][group];
    const actions = document.createElement('span');
    actions.className = 'grp-actions';

    // Paper-appearance master lives in the paper group header (eye + slider).
    if (group === 'paper') {
      const eye = document.createElement('button');
      eye.className = 'mini-btn';
      eye.title = t(app.lang, 'paperVisibility');
      const slider = document.createElement('input');
      slider.type = 'range';
      slider.min = '0'; slider.max = '1.5'; slider.step = '0.01';
      slider.style.width = '70px';
      slider.value = String(engine.tuning.get('paperVisibility'));
      let lastVisible = engine.tuning.get('paperVisibility') || 1;
      const syncEye = () => {
        const v = engine.tuning.get('paperVisibility');
        eye.textContent = v > 0 ? '👁' : '–';
        eye.classList.toggle('on', v > 0);
      };
      slider.addEventListener('input', () => {
        engine.tuning.set('paperVisibility', Number(slider.value));
        if (Number(slider.value) > 0) lastVisible = Number(slider.value);
        syncEye();
      });
      slider.addEventListener('pointerdown', (ev) => ev.stopPropagation());
      eye.addEventListener('click', (ev) => {
        ev.stopPropagation();
        const v = engine.tuning.get('paperVisibility');
        engine.tuning.set('paperVisibility', v > 0 ? 0 : lastVisible);
        slider.value = String(engine.tuning.get('paperVisibility'));
        syncEye();
      });
      syncEye();
      actions.append(eye, slider);
    }

    const reset = document.createElement('button');
    reset.className = 'mini-btn';
    reset.textContent = t(app.lang, 'resetGroup');
    reset.addEventListener('click', (ev) => {
      ev.stopPropagation();
      engine.tuning.resetGroup(group);
      refreshKnobs(app);
      finishPaperRebuild(app, group);
    });
    actions.appendChild(reset);
    head.append(title, actions);
    head.addEventListener('click', () => box.classList.toggle('closed'));

    const body = document.createElement('div');
    body.className = 'tune-body';
    for (const def of engine.tuning.defs) {
      if (def.group !== group || def.master) continue;
      body.appendChild(buildKnobRow(app, def));
    }
    box.append(head, body);
    panel.appendChild(box);
  }
  buildExperimental(app);
  if (!app._resizerBuilt) { buildResizer(); app._resizerBuilt = true; }
}

function buildKnobRow(app, def) {
  const { engine } = app;
  const row = document.createElement('div');
  row.className = 'knob-row';

  const label = document.createElement('span');
  label.className = 'kl';
  const name = document.createElement('span');
  const valueEcho = document.createElement('span');
  label.append(name, valueEcho);
  attachTooltip(app, label, def);

  const slider = document.createElement('input');
  slider.type = 'range';
  slider.min = String(def.min);
  slider.max = String(def.max);
  slider.step = String(def.step);

  const num = document.createElement('input');
  num.type = 'number';
  num.step = String(def.step);

  const reset = document.createElement('button');
  reset.className = 'knob-reset';
  reset.textContent = '↺';

  const apply = (v, { fromSlider = false } = {}) => {
    engine.tuning.set(def.key, v);
    syncRow(app, def);
    if (def.rebuild === 'paper' && !fromSlider) finishPaperRebuild(app, 'paper');
  };
  slider.addEventListener('input', () => apply(Number(slider.value), { fromSlider: true }));
  // Paper re-bakes are expensive: they run on slider RELEASE, not per tick.
  slider.addEventListener('change', () => { if (def.rebuild === 'paper') finishPaperRebuild(app, 'paper'); });
  num.addEventListener('change', () => apply(Number(num.value))); // NaN -> default (registry)
  reset.addEventListener('click', () => apply(def.def));

  row.append(label, slider, num, reset);
  app.knobEls[def.key] = { def, name, valueEcho, slider, num, reset };
  syncRow(app, def);
  return row;
}

function syncRow(app, def) {
  const el = app.knobEls[def.key];
  if (!el) return;
  const v = app.engine.tuning.get(def.key);
  el.name.textContent = app.lang === 'pt' ? def.pt : def.en;
  el.valueEcho.textContent = formatKnob(v);
  el.slider.value = String(v); // browser clamps the visual thumb if out of range
  el.num.value = formatKnob(v);
  el.reset.classList.toggle('off-default', !app.engine.tuning.isDefault(def.key));
}

export function refreshKnobs(app) {
  for (const key of Object.keys(app.knobEls)) syncRow(app, app.knobEls[key].def);
}

function formatKnob(v) {
  if (!Number.isFinite(v)) return '0';
  return Math.abs(v) >= 100 ? String(Math.round(v)) : String(Number(v.toPrecision(4)));
}

function finishPaperRebuild(app, group) {
  if (group === 'paper' && app.engine.paperDirty) rebakePaper(app.engine);
}

// ---------------------------------------------------------------------------
// Rich PT-BR tooltip with correlation letters
// ---------------------------------------------------------------------------

function attachTooltip(app, anchor, def) {
  const tip = document.getElementById('knob-tooltip');
  anchor.addEventListener('mouseenter', (ev) => {
    const doc = KNOB_DOCS[def.key];
    if (!doc) return;
    tip.textContent = '';
    const title = document.createElement('div');
    title.className = 'tt-title';
    title.textContent = `${def.pt} · ${def.en}`;
    const body = document.createElement('div');
    body.textContent = doc.doc;
    const recipe = document.createElement('div');
    recipe.className = 'tt-recipe';
    recipe.textContent = doc.recipe;
    const corr = document.createElement('div');
    corr.className = 'tt-corr';
    for (const [letter, strength] of doc.corr) {
      const el = document.createElement('span');
      el.className = `corr-letter corr-${letter}`;
      el.textContent = letter;
      el.style.opacity = String(0.4 + strength * 0.2);
      el.title = `${CORR_LABELS[letter]} (força ${strength}/3)`;
      corr.appendChild(el);
    }
    tip.append(title, body, recipe, corr);
    tip.hidden = false;
    const r = anchor.getBoundingClientRect();
    tip.style.left = `${Math.max(8, r.left - 330)}px`;
    tip.style.top = `${Math.min(window.innerHeight - 160, r.top)}px`;
  });
  anchor.addEventListener('mouseleave', () => { tip.hidden = true; });
}

// ---------------------------------------------------------------------------
// Experimental panel: the two visible K-M checkboxes. The section-17
// extensions ship implemented but neutral, with no visible sliders - their
// knobs live in the registry's `hidden` group; flip `hidden` group entries
// into a visible group in tuning.js to re-enable UI for them.
// ---------------------------------------------------------------------------

function buildExperimental(app) {
  const { engine } = app;
  const panel = document.getElementById('exp-panel');
  panel.textContent = '';
  const box = document.createElement('div');
  box.className = 'tune-group';
  const head = document.createElement('div');
  head.className = 'tune-head';
  const title = document.createElement('span');
  title.dataset.i18n = 'experimental';
  title.textContent = t(app.lang, 'experimental');
  head.appendChild(title);
  head.addEventListener('click', () => box.classList.toggle('closed'));
  const body = document.createElement('div');
  body.className = 'tune-body';

  const mk = (key, get, set) => {
    const lab = document.createElement('label');
    lab.className = 'chk';
    const input = document.createElement('input');
    input.type = 'checkbox';
    input.checked = get();
    input.addEventListener('change', () => { set(input.checked); markDirtyFull(engine); });
    const span = document.createElement('span');
    span.dataset.i18n = key;
    span.textContent = t(app.lang, key);
    lab.append(input, span);
    return lab;
  };
  body.appendChild(mk('kmMixing', () => engine.sim.kmMixing, (v) => { engine.sim.kmMixing = v; }));
  body.appendChild(mk('kmGlaze', () => engine.kmGlaze, (v) => { engine.kmGlaze = v; }));
  const note = document.createElement('div');
  note.className = 'exp-note';
  note.dataset.i18n = 'expNote';
  note.textContent = t(app.lang, 'expNote');
  body.appendChild(note);
  box.append(head, body);
  panel.appendChild(box);
}

// ---------------------------------------------------------------------------
// Panel resize (drag the left edge) - the panel is also collapsible via the
// bottom-bar toggle (handled in main.js).
// ---------------------------------------------------------------------------

function buildResizer() {
  const grip = document.getElementById('right-resize');
  const panel = document.getElementById('right-panel');
  grip.addEventListener('pointerdown', (ev) => {
    ev.preventDefault();
    grip.setPointerCapture(ev.pointerId);
    const startX = ev.clientX;
    const startW = panel.getBoundingClientRect().width;
    const move = (e2) => {
      const w = Math.max(180, Math.min(560, startW + (startX - e2.clientX)));
      panel.style.width = `${w}px`;
    };
    const up = () => {
      grip.removeEventListener('pointermove', move);
      grip.removeEventListener('pointerup', up);
    };
    grip.addEventListener('pointermove', move);
    grip.addEventListener('pointerup', up);
  });
}
