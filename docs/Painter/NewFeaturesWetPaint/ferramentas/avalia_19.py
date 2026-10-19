#!/usr/bin/env python3
"""O veredito da varredura dos pincéis por fazer (19/10): cada foto do Rebelle contra as receitas do seu grupo
(`receitas_19.py`) e o controlo `k_<grupo>`, pela barra do 18 §2 — uma métrica CONTA só se o controlo estiver
a mais de 3× o ruído do Rebelle da foto; passa quem fica do lado do Rebelle do ponto médio; a largura ± 2 px.
Imprime, por foto: as métricas que contam, quantas receitas passam TUDO, e as 3 melhores (nº de métricas, distância).

uso: avalia_19.py <dir do lote da sonda (.ppm)> <docs/Painter/NewFeaturesWetPaint> [sufixo dos grupos, «b»]
"""
import glob
import math
import os
import sys

from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from midias_regua import metricas  # noqa: E402

RUIDO = {"cob": 0.012, "tex": 0.37, "borda": 0.031, "orla": 1.0, "cv": 0.012, "cruz": 0.03}
FOTOS = {"lapis": ["pencil2h", "pencilhb", "pencil2b", "pencil4bcolor", "scratch"], "sq": ["squarehard"],
         "wx": ["waxscrape"], "filbert": ["filbert"], "rake": ["rake"], "dabber": ["dabber"],
         "lamina": ["flatoily", "knife"], "fountain": ["fountainsquare"], "reed": ["reedpen"], "chisel": ["markerchisel"]}

d, F = sys.argv[1], sys.argv[2]
for p in glob.glob(d + "/*.ppm"):
    q = p[:-4] + ".png"
    if not os.path.exists(q) or os.path.getmtime(q) < os.path.getmtime(p):
        Image.open(p).save(q)
cache = {}


def m(png):
    if png not in cache:
        try:
            cache[png] = metricas(png)
        except IndexError:  # a ponta de um traço que não se acha (a lâmina de lado): sem medida
            cache[png] = None
    return cache[png]


SUFIXO = sys.argv[3] if len(sys.argv) > 3 else ""  # «b»: os grupos da 2.ª passada (receitas_19b.py)
for g, fotos in FOTOS.items():
    g += SUFIXO
    if not os.path.exists(f"{d}/k_{g}.ppm"):
        continue
    C = m(f"{d}/k_{g}.png")
    nossos = [(os.path.basename(q)[:-4], m(q)) for q in sorted(glob.glob(f"{d}/{g}_*.png"))]
    for foto in fotos:
        R = m(f"{F}/fixtures/rebelle/{foto}.png")
        conta = [k for k in RUIDO if not math.isnan(R[k]) and not math.isnan(C[k]) and abs(C[k] - R[k]) > 3 * RUIDO[k]]
        rows = []
        for n, N in nossos:
            if N is None:
                continue
            ok = {k: abs(N[k] - R[k]) < abs(0.5 * (R[k] + C[k]) - R[k]) for k in conta}
            lw = abs(N["larg"] - R["larg"]) <= 2
            dist = sum(abs(N[k] - R[k]) / max(abs(C[k] - R[k]), 1e-9) for k in conta)
            rows.append((sum(ok.values()) + lw, -dist, n, N, ok, lw))
        rows.sort(reverse=True)
        tudo = sum(1 for r in rows if r[0] == len(conta) + 1)
        print(f"\n## {foto} ({g}) — contam {','.join(conta) or 'nenhuma'} + larg · passam tudo: {tudo}/{len(rows)}")
        print("   R " + " ".join(f"{k} {R[k]:.3f}" for k in RUIDO) + f" larg {R['larg']:.0f}")
        print("   C " + " ".join(f"{k} {C[k]:.3f}" for k in RUIDO) + f" larg {C['larg']:.0f}")
        for s, nd, n, N, ok, lw in rows[:3]:
            print(f"   {n:<13} {s}/{len(conta) + 1} dist {-nd:.2f} " + " ".join(
                f"{k} {N[k]:.3f}{'✓' if ok[k] else '✗'}" for k in conta) + f" larg {N['larg']:.0f}{'✓' if lw else '✗'}")
