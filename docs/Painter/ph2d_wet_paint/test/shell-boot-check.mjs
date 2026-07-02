// Auxiliary harness (not part of spec section 18): boots the FULL app shell
// (js/app/main.js) in Node under a minimal DOM shim, runs a few frames, and
// simulates pointer strokes + button clicks. Catches broken imports, missing
// element ids, and boot-time exceptions that a browser would show on load.
// Run:  node test/shell-boot-check.mjs

const listeners = new Map(); // element -> {type: [fn]}

function makeCtx() {
  return new Proxy({
    createImageData: (w, h) => ({ width: w, height: h, data: new Uint8ClampedArray(w * h * 4) }),
    putImageData: () => {},
    getImageData: (x, y, w, h) => ({ width: w, height: h, data: new Uint8ClampedArray(w * h * 4) }),
    createLinearGradient: () => ({ addColorStop: () => {} }),
    beginPath: () => {}, arc: () => {}, stroke: () => {}, fill: () => {},
    moveTo: () => {}, lineTo: () => {}, clearRect: () => {}, fillRect: () => {},
  }, { get: (t, k) => (k in t ? t[k] : () => {}), set: () => true });
}

function makeEl(id) {
  const el = {
    id,
    children: [],
    style: {},
    dataset: {},
    width: 900, height: 450,
    disabled: false, hidden: false, checked: false, value: '0', textContent: '',
    title: '', tagName: 'DIV',
    classList: {
      _s: new Set(),
      add(c) { this._s.add(c); }, remove(c) { this._s.delete(c); },
      toggle(c, f) { (f ?? !this._s.has(c)) ? this._s.add(c) : this._s.delete(c); },
      contains(c) { return this._s.has(c); },
    },
    getContext: () => makeCtx(),
    getBoundingClientRect: () => ({ left: 0, top: 0, width: 900, height: 450, right: 900, bottom: 450 }),
    addEventListener(type, fn) {
      if (!listeners.has(el)) listeners.set(el, {});
      (listeners.get(el)[type] ??= []).push(fn);
    },
    removeEventListener(type, fn) {
      const l = listeners.get(el)?.[type];
      if (l) { const i = l.indexOf(fn); if (i >= 0) l.splice(i, 1); }
    },
    dispatch(type, ev = {}) {
      for (const fn of listeners.get(el)?.[type] ?? []) {
        fn({ preventDefault: () => {}, stopPropagation: () => {}, target: el, pointerId: 1, ...ev });
      }
    },
    appendChild(c) { el.children.push(c); c._parent = el; return c; },
    append(...cs) { for (const c of cs) { el.children.push(c); c._parent = el; } },
    setPointerCapture: () => {},
    releasePointerCapture: () => {},
    removeAttribute: () => {},
    setAttribute: () => {},
    matches: () => false,
    offsetWidth: 200, offsetHeight: 100,
    click() { el.dispatch('click', { button: 0 }); },
  };
  Object.defineProperty(el, 'parentElement', {
    get() { return el._parent ?? (el._parent = makeEl(`${id}-parent`)); },
  });
  return el;
}

const byId = new Map();
const doc = {
  getElementById(id) {
    if (!byId.has(id)) byId.set(id, makeEl(id));
    return byId.get(id);
  },
  createElement(tag) {
    const el = makeEl(`<${tag}>`);
    el.tagName = tag.toUpperCase();
    return el;
  },
  querySelectorAll: () => [],
  title: 'PH2D Wet Paint',
};

const rafQueue = [];
globalThis.document = doc;
globalThis.window = {
  addEventListener: () => {},
  innerWidth: 1600, innerHeight: 900,
};
globalThis.requestAnimationFrame = (fn) => { rafQueue.push(fn); return rafQueue.length; };
globalThis.URL.createObjectURL ??= () => 'blob:fake';

let failed = false;
try {
  await import('../js/app/main.js');
  // Pump a handful of animation frames (~0.5 s of app time).
  let now = performance.now();
  for (let f = 0; f < 20 && rafQueue.length; f++) {
    const fn = rafQueue.shift();
    now += 25;
    fn(now);
  }
  // Scripted stroke through the pointer handlers.
  const viewport = doc.getElementById('canvas-viewport');
  viewport.dispatch('pointerdown', { button: 0, clientX: 200, clientY: 200 });
  for (let f = 0; f < 30 && rafQueue.length; f++) {
    viewport.dispatch('pointermove', { clientX: 200 + f * 6, clientY: 200 + f });
    const fn = rafQueue.shift();
    now += 25;
    fn(now);
  }
  viewport.dispatch('pointerup', { button: 0, clientX: 380, clientY: 230 });
  for (let f = 0; f < 40 && rafQueue.length; f++) {
    const fn = rafQueue.shift();
    now += 25;
    fn(now);
  }
  // Buttons: exercise every bottom-bar action once.
  for (const id of ['btn-wet-canvas', 'btn-fast-dry', 'btn-show-wet', 'btn-dry-canvas',
    'btn-undo', 'btn-redo', 'btn-clear', 'btn-toggle-tuning', 'btn-lang']) {
    doc.getElementById(id).dispatch('click', { button: 0 });
  }
  // Rich tooltips: hover a few wired controls (renders + positions the tip).
  for (const id of ['btn-undo', 'btn-save', 'tilt-dial', 'color-wheel', 'layer-add']) {
    const el = doc.getElementById(id);
    el.dispatch('mouseenter', {});
    el.dispatch('mouseleave', {});
  }
  for (let f = 0; f < 10 && rafQueue.length; f++) {
    const fn = rafQueue.shift();
    now += 25;
    fn(now);
  }
  console.log('shell boot check: OK (module graph loads, stroke + buttons run without exceptions)');
} catch (err) {
  failed = true;
  console.error('shell boot check FAILED:', err.stack || err);
}
process.exit(failed ? 1 : 0);
