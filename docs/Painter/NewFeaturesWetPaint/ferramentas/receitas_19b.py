#!/usr/bin/env python3
"""A 2.ª passada da varredura dos pincéis por fazer (19/10), feita a partir do veredito da 1.ª (`avalia_19.py`):
(1) os lápis passavam tudo menos a LARGURA (a lei raio → largura da sonda lê o traço molhado; a régua, seco e com
grão: 3–5 px a menos) ⇒ as 5 melhores de cada foto com o alvo corrigido por `alvo × R_larg / N_larg`;
(2) grelhas novas no eixo que reprovou: a lâmina (Flat Oily, Knife) lisa — bases sem cerdas; o Fountain Square com
grão; o Dabber com a borda dura.

uso: receitas_19b.py <dir do lote 1> <docs/Painter/NewFeaturesWetPaint> <saída.json>
"""
import glob
import itertools
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from midias_regua import metricas  # noqa: E402


def grao(kind, lado):  # o de receitas_19.py (que grava ao ser importado)
    return {"angle_deg": 0, "flow": False, "kind": kind, "mapping": "Tiled", "offset": [0.0, 0.0],
            "params": [0.5] * 6, "rake": False, "size": [lado, lado], "stencil_angle_deg": 0,
            "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}


d, F, saida = sys.argv[1], sys.argv[2], sys.argv[3]
lote1 = {r["nome"]: r for r in json.load(open(os.path.join(os.path.dirname(saida), "receitas_19.json")))}
R = []


def rec(nome, base, alvo, knobs=None, brush=None, water=None):
    r = {"nome": nome, "base": base, "alvo": alvo, "knobs": knobs or {}, "brush": brush or {}}
    if water is not None:
        r["water"] = water
    R.append(r)


# (1) Os lápis: as 5 melhores por foto (pela mesma distância do avalia_19), à largura da foto.
RUIDO = {"cob": 0.012, "tex": 0.37, "borda": 0.031, "orla": 1.0, "cv": 0.012, "cruz": 0.03}
C = metricas(f"{d}/k_lapis.png")
nossos = {os.path.basename(q)[:-4]: metricas(q) for q in glob.glob(f"{d}/lapis_*.png")}
escolhidos = {}
for foto in ["pencil2h", "pencilhb", "pencil2b", "pencil4bcolor", "scratch"]:
    Rf = metricas(f"{F}/fixtures/rebelle/{foto}.png")
    conta = [k for k in RUIDO if abs(C[k] - Rf[k]) > 3 * RUIDO[k]]
    dist = {n: sum(abs(N[k] - Rf[k]) / abs(C[k] - Rf[k]) for k in conta) for n, N in nossos.items() if N}
    for n in sorted(dist, key=dist.get)[:5]:
        escolhidos[n] = max(escolhidos.get(n, 0), Rf["larg"])
rec("k_lapisb", "Wet Round", 24)
for n, larg in sorted(escolhidos.items()):
    r = lote1[n]
    rec(f"lapisb_{n[6:]}", r["base"], round(r["alvo"] * larg / nossos[n]["larg"], 1), r["knobs"], r["brush"])
# (2a) A lâmina lisa (Flat Oily 10, Knife 9): a Dip Pen e a Marker Bullet chatas; a acrílica sem cerdas.
rec("k_laminab", "Wet Round", 10)
for b, f, a, gl in itertools.product(["Dip Pen", "Marker Bullet"], [0.7, 0.78, 0.85], [0, 90], [0.3, 0.6]):
    rec(f"laminab_{len(R):03d}", b, 10, {"glaze": gl}, {"dab_flatten": f, "dab_angle_deg": a, "hardness": 0.9})
for f, a in itertools.product([0.7, 0.78, 0.85], [0, 90]):
    rec(f"laminab_{len(R):03d}", "Acrylic Round", 10, {"bristleStrength": 0.0, "felt": 1.0},
        {"dab_flatten": f, "dab_angle_deg": a, "hardness": 0.9})
# (2b) O Fountain Square (25): a Dip Pen com o papel e o grão.
rec("k_fountainb", "Wet Round", 25)
for pg, l, gl in itertools.product([0.2, 0.4, 0.7], [None, 16, 48], [0.4, 0.6]):
    rec(f"fountainb_{len(R):03d}", "Dip Pen", 25, {"glaze": gl, "pigmentPerDab": 750, "paperGate": pg},
        {"texture": grao("Noise", l)} if l else {})
# (2c) O Dabber (36): a borda dura e a orla curta — a acrílica e o Oil Bristle com espalhamento.
rec("k_dabberb", "Wet Round", 36)
for b, j, h, s in itertools.product(["Acrylic Round", "Oil Bristle"], [0.1, 0.3], [0.6, 0.9], [2.0, 8.0]):
    rec(f"dabberb_{len(R):03d}", b, 36, {"bristleSize": s}, {"jitter": j, "hardness": h})

json.dump(R, open(saida, "w"), indent=0)
print(len(R), "receitas")

# 3.ª passada (`receitas_19b.py … --c`): os três a um passo — o Flat Oily (o cruzamento escuro de mais: mais carga
# cobre), o Dabber (textura 6,8 contra 10,7: o relevo e o grão) e o Scratch (o Charcoal entre a largura 24 e 36).
if "--c" in sys.argv:
    R.clear()
    rec("k_laminac", "Wet Round", 10)
    for p, f, fe in itertools.product([250, 500, 1000], [0.74, 0.78, 0.82], [0.5, 1.0]):
        rec(f"laminac_{len(R):03d}", "Acrylic Round", 10, {"bristleStrength": 0.0, "felt": fe, "pigmentPerDab": p},
            {"dab_flatten": f, "dab_angle_deg": 0, "hardness": 0.9})
    rec("k_dabberc", "Wet Round", 36)
    for j, e, s, l in itertools.product([0.3, 0.5], [0.7, 1.2], [2.0, 3.0], [None, 32]):
        rec(f"dabberc_{len(R):03d}", "Oil Bristle", 36, {"bristleSize": 2.0, "emboss": e, "bristleStrength": s},
            {"jitter": j, "hardness": 0.6, **({"texture": grao("Noise", l)} if l else {})})
    rec("k_lapisc", "Wet Round", 24)
    for a, p, g in itertools.product([28, 30, 32], [12, 16, 20], [0.5, 0.8]):
        rec(f"lapisc_{len(R):03d}", "Charcoal", a, {"pigmentPerDab": p, "paperGate": g})
    json.dump(R, open(saida.replace(".json", "c.json"), "w"), indent=0)
    print(len(R), "receitas (3.ª passada)")
