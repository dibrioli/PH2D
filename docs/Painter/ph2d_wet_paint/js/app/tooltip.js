// Shared floating rich-tooltip system (tuning knobs + all chrome controls).
// One fixed-position singleton on <body>, so panels can never clip it; the
// placement flips sides so it never leaves the viewport next to either the
// left or the right panel.

import { CORR_LABELS } from './knobdocs.js';

const MARGIN = 8;   // viewport padding
const OFFSET = 12;  // gap between anchor and tooltip

let tipEl = null;

function ensureEl() {
  if (!tipEl) tipEl = document.getElementById('knob-tooltip');
  return tipEl;
}

/**
 * Render + place the tooltip near an anchor rect.
 * content = { emoji?, title, doc, tip?, corr? [[letter, strength]] }.
 */
function show(anchor, content) {
  const tip = ensureEl();
  tip.textContent = '';
  const title = document.createElement('div');
  title.className = 'tt-title';
  title.textContent = content.emoji ? `${content.emoji} ${content.title}` : content.title;
  tip.appendChild(title);
  if (content.doc) {
    const body = document.createElement('div');
    body.textContent = content.doc;
    tip.appendChild(body);
  }
  if (content.tip) {
    const rec = document.createElement('div');
    rec.className = 'tt-recipe';
    rec.textContent = content.tip;
    tip.appendChild(rec);
  }
  if (content.corr && content.corr.length) {
    const corr = document.createElement('div');
    corr.className = 'tt-corr';
    for (const [letter, strength] of content.corr) {
      const el = document.createElement('span');
      el.className = `corr-letter corr-${letter}`;
      el.textContent = letter;
      el.style.opacity = String(0.4 + strength * 0.2);
      el.title = `${CORR_LABELS[letter]} (força ${strength}/3)`;
      corr.appendChild(el);
    }
    tip.appendChild(corr);
  }
  // Measure hidden, then place with side-flipping.
  tip.style.visibility = 'hidden';
  tip.hidden = false;
  const w = tip.offsetWidth, h = tip.offsetHeight;
  const r = anchor.getBoundingClientRect();
  let x = r.right + OFFSET; // prefer the right side (left-panel anchors)
  if (x + w > window.innerWidth - MARGIN) x = r.left - w - OFFSET; // flip left (right-panel anchors)
  if (x < MARGIN) x = Math.min(Math.max(MARGIN, r.left), window.innerWidth - w - MARGIN);
  let y = r.top;
  if (y + h > window.innerHeight - MARGIN) y = window.innerHeight - h - MARGIN;
  if (y < MARGIN) y = MARGIN;
  tip.style.left = `${x}px`;
  tip.style.top = `${y}px`;
  tip.style.visibility = 'visible';
}

export function hideTooltip() {
  const tip = ensureEl();
  tip.hidden = true;
}

/**
 * Attach a rich tooltip to an element. `provider` is called on every hover
 * and returns the content object (or null to skip) - so dynamic titles
 * (e.g. per-language knob labels) stay fresh.
 */
export function attachRichTooltip(anchor, provider) {
  anchor.dataset.richTip = '1'; // suppress the native title (see i18n.applyStatic)
  anchor.removeAttribute('title');
  anchor.addEventListener('mouseenter', () => {
    const content = typeof provider === 'function' ? provider() : provider;
    if (content) show(anchor, content);
  });
  anchor.addEventListener('mouseleave', hideTooltip);
  anchor.addEventListener('pointerdown', hideTooltip);
}
