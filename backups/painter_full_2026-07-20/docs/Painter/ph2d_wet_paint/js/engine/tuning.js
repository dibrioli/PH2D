// Live tuning registry (spec section 16): one module owning every calibration
// knob the engine reads at runtime. Each def: key, group, EN/PT label, a
// comfortable slider range (free numeric input may exceed it), default, and
// an optional rebuild kind ('brush' = rebuild the bristle texture, 'paper' =
// re-bake the sheet on release, 'render' = repaint).
//
// The `hidden` group holds the gated extensions of spec section 17: they ship
// implemented but default-neutral with no visible sliders. To re-enable one
// in the UI, flip its `hidden` flag - the panel builds itself from this table.

export const KNOB_DEFS = [
  // group, key, en, pt, min, max, step, def, rebuild
  k('paint', 'pigmentPerDab', 'Pigment per dab', 'Pigmento por toque', 0, 2000, 10, 600),
  k('paint', 'paperGate', 'Paper gate', 'Barreira do grao', 0, 1.5, 0.01, 0.6),
  k('paint', 'felt', 'Felt (pores)', 'Feltro (poros)', 0, 0.2, 0.001, 0.01, 'brush'),
  k('paint', 'bristleCount', 'Bristle count', 'Numero de cerdas', 0, 4000, 10, 950, 'brush'),
  k('paint', 'drag', 'Drag', 'Arrasto', 0, 1, 0.01, 0.16),
  k('paint', 'pickup', 'Pickup', 'Captura', 0, 0.2, 0.001, 0.005),
  k('paint', 'intensity', 'Intensity', 'Intensidade', 0, 4, 0.05, 1),
  k('paint', 'bristleStrength', 'Bristle strength', 'Forca das cerdas', 0, 4, 0.05, 1, 'brush'),
  k('paint', 'bristleSize', 'Bristle size', 'Tamanho das cerdas', 0.2, 4, 0.05, 1, 'brush'),
  k('paint', 'spacing', 'Spacing', 'Espacamento', 0.5, 8, 0.5, 2),
  k('paint', 'tipClean', 'Tip clean', 'Limpeza da ponta', 0, 0.05, 0.0005, 0.001),
  k('paint', 'blendForce', 'Blend force', 'Forca do blend', 0, 1, 0.01, 0.8),
  k('paint', 'gateSaturation', 'Gate saturation', 'Saturacao da barreira', 100, 6000, 50, 2000),
  k('water', 'waterPerDab', 'Water per dab', 'Agua por toque', 0, 4, 0.05, 1),
  k('water', 'waterCap', 'Water cap', 'Teto de agua', 1, 30, 0.5, 8),
  k('water', 'evaporation', 'Evaporation', 'Evaporacao', 0, 8, 0.05, 1),
  k('water', 'rewet', 'Re-wet', 'Reumedecimento', 0, 8, 0.05, 1),
  k('water', 'retention', 'Retention', 'Retencao', 0.9, 1, 0.0005, 0.995),
  k('water', 'edgeDarkening', 'Edge darkening', 'Escurecimento de borda', 0, 200, 1, 50),
  k('water', 'baseEvaporation', 'Base evaporation', 'Evaporacao base', 0, 20, 0.1, 2),
  k('physics', 'leveling', 'Leveling', 'Nivelamento', 0, 2, 0.01, 0.5),
  k('physics', 'capillary', 'Capillary', 'Capilaridade', 0, 1, 0.01, 0.2),
  k('physics', 'brake', 'Brake', 'Freio', 0, 4, 0.05, 1.5),
  k('physics', 'gravity', 'Gravity', 'Gravidade', 0, 0.05, 0.0005, 0.005),
  k('physics', 'levelClamp', 'Level clamp', 'Trava de nivelamento', 0, 1, 0.01, 0.2),
  k('physics', 'viscosity', 'Viscosity', 'Viscosidade', 0, 2, 0.01, 0.2),
  k('physics', 'maxVelocity', 'Max velocity', 'Velocidade maxima', 0.2, 3, 0.05, 1),
  k('physics', 'projection', 'Projection', 'Projecao', 0, 1.5, 0.01, 0.7),
  k('physics', 'brakeReach', 'Brake reach', 'Alcance do freio', 1, 12, 0.5, 4),
  k('physics', 'capillaryGate', 'Capillary gate', 'Barreira capilar', 100, 6000, 50, 2000),
  k('tools', 'eraser', 'Eraser', 'Borracha', 0, 1, 0.01, 0.4),
  k('tools', 'dryer', 'Dryer', 'Secador', 0, 1, 0.01, 0.32),
  k('tools', 'blow', 'Blow', 'Sopro', 0, 1, 0.01, 0.2),
  k('tools', 'smear', 'Smear', 'Esfregar', 0, 1, 0.01, 0.8),
  k('paper', 'paperContrast', 'Contrast', 'Contraste', 0.2, 2.5, 0.05, 1, 'paper'),
  k('paper', 'paperFibres', 'Fibres', 'Fibras', 0, 3, 0.05, 1, 'paper'),
  k('paper', 'paperGrooves', 'Grooves', 'Sulcos', 0, 3, 0.05, 1, 'paper'),
  k('paper', 'visualGrain', 'Visual grain', 'Grao visual', 0, 3, 0.05, 1, 'render'),
  k('paper', 'emboss', 'Emboss', 'Relevo', 0, 4, 0.05, 1, 'render'),
  // Render-only master (paper group header): fades the tooth in the
  // APPEARANCE only - the physics never sees it.
  k('paper', 'paperVisibility', 'Paper visibility', 'Visibilidade do papel', 0, 1.5, 0.01, 1, 'render', { master: true }),
  // ---- gated extensions (spec section 17): neutral defaults, no sliders ----
  k('hidden', 'extDiffusion', 'Diffusion', 'Difusao', 0, 0.25, 0.005, 0),
  k('hidden', 'extBackrun', 'Backrun', 'Refluxo (bloom)', 0, 2, 0.05, 0),
  k('hidden', 'extFingering', 'Fingering', 'Dedos de escorrimento', 0, 1, 0.01, 0),
  k('hidden', 'extRivulets', 'Rivulet count', 'Numero de filetes', 4, 60, 1, 15),
  k('hidden', 'extGranulation', 'Physical granulation', 'Granulacao fisica', 0, 1, 0.01, 0),
  k('hidden', 'extStaining', 'Staining', 'Fixacao (staining)', 0, 1, 0.01, 0.5),
  k('hidden', 'extDryBrush', 'Dry brush', 'Pincel seco', 0, 1, 0.01, 0),
  k('hidden', 'extWetSoften', 'Wet-edge softening', 'Suavizacao em molhado', 0, 1, 0.01, 0),
  k('hidden', 'extRakeLight', 'Rake light', 'Luz rasante', 0, 3, 0.05, 0, 'render'),
  k('hidden', 'extValleyGran', 'Valley granulation', 'Granulacao nos vales', 0, 1, 0.01, 0, 'render'),
  k('hidden', 'extWetSheen', 'Wet sheen', 'Brilho umido', 0, 2, 0.05, 0, 'render'),
  k('hidden', 'extGrainDither', 'Grain dither', 'Ruido de grao', 0, 8, 0.1, 0, 'render'),
  k('hidden', 'extEdgeTint', 'Edge tint', 'Tinta de borda', 0, 1, 0.01, 0.5, 'render'),
];

function k(group, key, en, pt, min, max, step, def, rebuild = null, extra = {}) {
  return { group, key, en, pt, min, max, step, def, rebuild, ...extra };
}

export const KNOB_GROUPS = ['paint', 'water', 'physics', 'tools', 'paper'];

/**
 * Create a live tuning instance. `onChange(def, value)` fires on every write
 * (the shell uses it to trigger brush/paper rebuilds and repaints).
 */
export function createTuning(onChange = null) {
  const byKey = new Map(KNOB_DEFS.map((d) => [d.key, d]));
  const values = {};
  for (const d of KNOB_DEFS) values[d.key] = d.def;
  return {
    defs: KNOB_DEFS,
    byKey,
    values,
    get(key) { return values[key]; },
    /** NaN (e.g. from a garbled numeric input) falls back to the default. */
    set(key, v) {
      const d = byKey.get(key);
      if (!d) return;
      const nv = Number.isNaN(v) || typeof v !== 'number' ? d.def : v;
      if (values[key] === nv) return;
      values[key] = nv;
      if (onChange) onChange(d, nv);
    },
    reset(key) { this.set(key, byKey.get(key)?.def); },
    resetGroup(group) {
      for (const d of KNOB_DEFS) if (d.group === group) this.set(d.key, d.def);
    },
    isDefault(key) { return values[key] === byKey.get(key)?.def; },
  };
}
