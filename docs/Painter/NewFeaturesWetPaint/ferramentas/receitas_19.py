#!/usr/bin/env python3
"""As receitas da varredura dos pincéis por fazer (19/10): para cada pincel do Rebelle ainda sem preset nosso, uma
grelha dos knobs de HOJE a partir do vizinho de fábrica (`base`), à largura MEDIDA da foto dele (`alvo`), e o
controlo `k_<grupo>` (o Wet Round à mesma largura). Fotos: `fixtures/rebelle/<foto>.png` (lote_19.txt).

uso: receitas_19.py <saída.json>   → depois a sonda_midias (PH2D_SONDA_RECEITAS) e o midias_lote.py por grupo.
"""
import itertools
import json
import sys


def grao(kind, lado):
    return {"angle_deg": 0, "flow": False, "kind": kind, "mapping": "Tiled", "offset": [0.0, 0.0],
            "params": [0.5] * 6, "rake": False, "size": [lado, lado], "stencil_angle_deg": 0,
            "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}


R = []


def rec(nome, base, alvo, knobs=None, brush=None, water=None):
    r = {"nome": nome, "base": base, "alvo": alvo, "knobs": knobs or {}, "brush": brush or {}}
    if water is not None:
        r["water"] = water
    R.append(r)


def grupo(g, alvo, linhas):
    rec(f"k_{g}", "Wet Round", alvo)
    for i, (base, knobs, brush, *w) in enumerate(linhas):
        rec(f"{g}_{i:03d}", base, alvo, knobs, brush, w[0] if w else None)


# Lápis 2H · HB · 2B · 4B Color · Scratch (larg 24–25): o Graphite 8B mais ralo, e o Charcoal para o Scratch.
grupo("lapis", 24, [("Graphite 8B", {"pigmentPerDab": p, "paperGate": g}, {"texture": grao("Noise", l)})
                    for p, g, l in itertools.product([5, 10, 20, 40, 80, 160], [0.3, 0.65, 1.0], [16, 32, 64])]
      + [("Charcoal", {"pigmentPerDab": p, "paperGate": g}, {})
         for p, g in itertools.product([8, 16, 32], [0.0, 0.5, 1.0])])
# Pastel Square Hard (31) e Wax Scrape (36): os dois secos de grão, a dureza, a ponta reta.
for g, alvo in (("sq", 31), ("wx", 36)):
    grupo(g, alvo, [(b, {"pigmentPerDab": p, "paperGate": pg}, {"hardness": h, "dab_quina": q, "texture": grao("Noise", l)})
                    for b, h, p, pg, l, q in itertools.product(["Soft Pastel", "Oil Pastel"], [0.2, 0.6, 0.9], [60, 150],
                                                               [0.4, 1.0], [50, 100], [0.0, 1.0])
                    if g == "sq" or q == 0.0])
# Óleo Filbert (40): a acrílica, cerdas e carga.
grupo("filbert", 40, [("Acrylic Round", {"bristleSize": s, "pigmentPerDab": p}, {"hardness": h})
                      for h, s, p in itertools.product([0.3, 0.5, 0.7], [0.5, 1.0, 2.0], [250, 500, 1000])])
# Óleo Rake Sharp (38): o Oil Bristle mais ou menos chato, a força das cerdas, o relevo.
grupo("rake", 38, [("Oil Bristle", {"bristleStrength": s, "emboss": e}, {"dab_flatten": f})
                   for f, s, e in itertools.product([0.0, 0.3, 0.68], [1.0, 2.0, 3.0], [0.0, 0.35, 0.7])])
# Óleo Dabber (36): a esponja — o Express Oil com espalhamento.
grupo("dabber", 36, [("Express Oil", {"bristleSize": s}, {"jitter": j, "hardness": h})
                     for j, s, h in itertools.product([0.0, 0.2, 0.4], [2.0, 4.0, 8.0], [0.3, 0.9])])
# Óleo Flat Oily (10) e Knife Smooth (9): a lâmina de lado no recto (larga na curva).
grupo("lamina", 10, [("Acrylic Round", {}, {"dab_flatten": f, "dab_angle_deg": a, "hardness": h})
                     for f, a, h in itertools.product([0.7, 0.78, 0.85, 0.92], [0, 90], [0.5, 0.9])])
# Tinta Fountain Square (25): a Dip Pen com menos Glaze, menos carga, a ponta chata e reta.
grupo("fountain", 25, [("Dip Pen", {"glaze": gl, "pigmentPerDab": p}, {"dab_flatten": f, "dab_quina": q})
                       for gl, p, f, q in itertools.product([0.4, 0.6, 0.8], [300, 750, 1500], [0.0, 0.4], [0.0, 1.0])])
# Tinta Reed Pen (34): a Ink Bullet.
grupo("reed", 34, [("Ink Bullet", {"glaze": gl, "pigmentPerDab": p}, {}, w)
                   for gl, p, w in itertools.product([0.8, 0.9, 1.0], [1000, 2000], [0.15, 0.3])])
# Marker Chisel (16): o Marker Bullet chato e de ponta reta.
grupo("chisel", 16, [("Marker Bullet", {}, {"dab_flatten": f, "dab_angle_deg": a, "hardness": h, "dab_quina": q})
                     for f, a, h, q in itertools.product([0.0, 0.3, 0.6], [0, 90], [0.55, 0.75], [0.0, 1.0])])

json.dump(R, open(sys.argv[1], "w"), indent=0)
print(len(R), "receitas")
