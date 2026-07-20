// Canvas viewport: blitting the engine's RGBA composite (dirty-rect aware),
// wheel zoom 0.25x-8x toward the cursor, middle-drag pan, and the reset pill.

import { renderRegion } from '../engine/render.js';
import { gatherParams } from '../engine/sim.js';
import { activeGrid, takeDirty } from '../engine/painter.js';

export function createView(engine) {
  const canvas = document.getElementById('paint-canvas');
  const wrap = document.getElementById('canvas-wrap');
  const viewport = document.getElementById('canvas-viewport');
  const pill = document.getElementById('zoom-pill');
  const ctx = canvas.getContext('2d');
  const W = engine.width, H = engine.height;
  const image = ctx.createImageData(W, H);

  const view = {
    canvas, viewport,
    zoom: 1, panX: 0, panY: 0,
    panning: false, panStartX: 0, panStartY: 0, panBaseX: 0, panBaseY: 0,
  };

  function applyTransform() {
    wrap.style.transform = `translate(${view.panX}px, ${view.panY}px) scale(${view.zoom})`;
    canvas.style.imageRendering = view.zoom >= 2 ? 'pixelated' : 'auto';
    pill.textContent = `${Math.round(view.zoom * 100)}%`;
  }

  function resetTransform() {
    const vw = viewport.clientWidth, vh = viewport.clientHeight;
    view.zoom = 1;
    view.panX = Math.max(0, (vw - W) / 2);
    view.panY = Math.max(0, (vh - H) / 2);
    applyTransform();
  }

  /** Pointer event -> engine cell coordinates (floats). */
  view.toCell = (ev) => {
    const r = viewport.getBoundingClientRect();
    const px = (ev.clientX - r.left - view.panX) / view.zoom;
    const py = (ev.clientY - r.top - view.panY) / view.zoom;
    return { x: px + 1, y: py + 1 }; // pixel (0..W-1) maps to cell (1..W)
  };

  /** Repaint whatever the engine marked dirty. */
  view.flush = () => {
    const d = takeDirty(engine);
    if (!d) return;
    const g = activeGrid(engine);
    let x0 = 1, y0 = 1, x1 = W, y1 = H;
    if (d !== 'full') { x0 = d.x0; y0 = d.y0; x1 = d.x1; y1 = d.y1; }
    const cfg = {
      P: gatherParams(engine.sim),
      layers: engine.layers,
      activeGrid: g,
      showWet: engine.showWet,
      kmGlaze: engine.kmGlaze,
      extBypass: engine.sim.extBypass,
    };
    renderRegion(cfg, image.data, x0, y0, x1, y1);
    ctx.putImageData(image, 0, 0, x0 - 1, y0 - 1, x1 - x0 + 1, y1 - y0 + 1);
  };

  // ---- zoom (wheel, toward the cursor) ----
  viewport.addEventListener('wheel', (ev) => {
    ev.preventDefault();
    const r = viewport.getBoundingClientRect();
    const mx = ev.clientX - r.left, my = ev.clientY - r.top;
    const oldZoom = view.zoom;
    const factor = Math.exp(-ev.deltaY * 0.0015);
    let z = oldZoom * factor;
    z = Math.max(0.25, Math.min(8, z));
    // Keep the canvas point under the cursor fixed.
    view.panX = mx - ((mx - view.panX) / oldZoom) * z;
    view.panY = my - ((my - view.panY) / oldZoom) * z;
    view.zoom = z;
    applyTransform();
  }, { passive: false });

  // ---- middle-drag pan ----
  viewport.addEventListener('pointerdown', (ev) => {
    if (ev.button !== 1) return;
    ev.preventDefault();
    view.panning = true;
    view.panStartX = ev.clientX; view.panStartY = ev.clientY;
    view.panBaseX = view.panX; view.panBaseY = view.panY;
    viewport.setPointerCapture(ev.pointerId);
  });
  viewport.addEventListener('pointermove', (ev) => {
    if (!view.panning) return;
    view.panX = view.panBaseX + (ev.clientX - view.panStartX);
    view.panY = view.panBaseY + (ev.clientY - view.panStartY);
    applyTransform();
  });
  viewport.addEventListener('pointerup', (ev) => {
    if (ev.button === 1) view.panning = false;
  });

  pill.addEventListener('click', resetTransform);
  window.addEventListener('resize', () => { if (view.zoom === 1) resetTransform(); });
  resetTransform();
  return view;
}
