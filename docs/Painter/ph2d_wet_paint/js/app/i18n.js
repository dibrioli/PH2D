// App chrome string table, EN / PT (spec section 16). Knob tooltips are
// intentionally PT-BR only and live in knobdocs.js.

export const STRINGS = {
  en: {
    undo: 'Undo', redo: 'Redo',
    wetCanvas: 'Wet canvas', dryCanvas: 'Dry canvas', fastDry: 'Fast dry', showWet: 'Show wet',
    clear: 'Clear', savePng: 'Save PNG', withPaper: 'paper',
    tuning: 'Tuning', experimental: 'Experimental',
    tilt: 'Tilt', paper: 'Paper', layers: 'Layers',
    brushSize: 'Brush', pressure: 'Pressure', water: 'Water', eraseStrength: 'Erase',
    round: 'Round', flat: 'Flat', fan: 'Fan',
    wetBrush: 'Wet brush', dryBrush: 'Dry brush',
    paint: 'Paint', erase: 'Erase', smear: 'Smear', blend: 'Blend',
    wet: 'Wet', dry: 'Dry', blow: 'Blow',
    coldPress: 'Cold press', rough: 'Rough', hotPress: 'Hot press',
    layer: 'Layer',
    kmMixing: 'Pigment mixing (K–M)', kmGlaze: 'Glaze layering (K–M)',
    expNote: 'Kubelka–Munk subtractive color. Further gated extensions (diffusion, backrun, fingering, dry-brush, render extras) ship neutral; see the tuning registry’s hidden group.',
    tiltOn: 'on', tiltOff: 'off',
    resetGroup: 'reset',
    paperVisibility: 'Paper visibility',
  },
  pt: {
    undo: 'Desfazer', redo: 'Refazer',
    wetCanvas: 'Molhar tela', dryCanvas: 'Secar tela', fastDry: 'Secagem rápida', showWet: 'Ver umidade',
    clear: 'Limpar', savePng: 'Salvar PNG', withPaper: 'papel',
    tuning: 'Ajustes', experimental: 'Experimental',
    tilt: 'Inclinação', paper: 'Papel', layers: 'Camadas',
    brushSize: 'Pincel', pressure: 'Pressão', water: 'Água', eraseStrength: 'Borracha',
    round: 'Redondo', flat: 'Chato', fan: 'Leque',
    wetBrush: 'Pincel molhado', dryBrush: 'Pincel seco',
    paint: 'Pintar', erase: 'Apagar', smear: 'Esfregar', blend: 'Misturar',
    wet: 'Molhar', dry: 'Secar', blow: 'Soprar',
    coldPress: 'Prensado frio', rough: 'Rugoso', hotPress: 'Prensado quente',
    layer: 'Camada',
    kmMixing: 'Mistura de pigmento (K–M)', kmGlaze: 'Camadas em velatura (K–M)',
    expNote: 'Cor subtrativa Kubelka–Munk. As demais extensões (difusão, backrun, fingering, pincel seco, extras de render) estão implementadas com padrão neutro; veja o grupo oculto do registro de ajustes.',
    tiltOn: 'ligado', tiltOff: 'desligado',
    resetGroup: 'restaurar',
    paperVisibility: 'Visibilidade do papel',
  },
};

export const GROUP_LABELS = {
  en: { paint: 'Paint', water: 'Water', physics: 'Physics', tools: 'Tools', paper: 'Paper' },
  pt: { paint: 'Tinta', water: 'Água', physics: 'Física', tools: 'Ferramentas', paper: 'Papel' },
};

export function t(lang, key) {
  return STRINGS[lang][key] ?? STRINGS.en[key] ?? key;
}

/** Apply data-i18n text swaps across the static DOM. */
export function applyStatic(lang) {
  for (const el of document.querySelectorAll('[data-i18n]')) {
    el.textContent = t(lang, el.dataset.i18n);
  }
}
