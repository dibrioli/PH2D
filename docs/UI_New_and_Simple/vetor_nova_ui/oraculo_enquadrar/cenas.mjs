// As cenas NOSSAS do oráculo do «enquadrar tudo» (F8) — em px de página/cena, viewport 1280×800.
// Um objecto = [x, y, w, h, tipo]; `tranca` = índices trancados; `sel` = índices selecionados.
// O MESMO ficheiro alimenta os dois corredores e o gate (`fixtures_*/<cena>.json` traz a cena junto).
export const VIEWPORT = { width: 1280, height: 800 };

export const CENAS = {
  pequeno: { objs: [[600, 385, 40, 30, "rect"]] },
  ponto: { objs: [[640, 400, 2, 2, "rect"]] },
  medio_ao_centro: { objs: [[440, 250, 400, 300, "rect"]] },
  espalhados: {
    objs: [
      [-900, -500, 120, 80, "rect"],
      [300, 200, 200, 150, "rect"],
      [2400, 1500, 160, 160, "rect"],
      [1000, -300, 100, 60, "ellipse"],
    ],
  },
  comprido: { objs: [[0, 400, 6000, 20, "rect"]] },
  alto: { objs: [[600, 0, 20, 4000, "rect"]] },
  enorme: { objs: [[0, 0, 400000, 250000, "rect"]] },
  vazia: { objs: [] },
  trancado_longe: {
    objs: [
      [300, 200, 200, 150, "rect"],
      [2400, 1500, 160, 160, "rect"],
    ],
    tranca: [1],
  },
  selecao_um: {
    objs: [
      [-900, -500, 120, 80, "rect"],
      [300, 200, 200, 150, "rect"],
      [2400, 1500, 160, 160, "rect"],
    ],
    sel: [1],
  },
  selecao_pequena: {
    objs: [
      [-900, -500, 120, 80, "rect"],
      [600, 385, 40, 30, "rect"],
    ],
    sel: [1],
  },
};

// As câmaras de partida: o resultado não pode depender de onde a vista estava (só a animação).
export const PARTIDAS = {
  z1_origem: { x: 0, y: 0, z: 1 },
  z025_deslocada: { x: 500, y: 300, z: 0.25 },
  z4_deslocada: { x: -200, y: -100, z: 4 },
};
