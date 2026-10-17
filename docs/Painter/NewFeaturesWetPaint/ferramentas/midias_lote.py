#!/usr/bin/env python3
"""O medidor de um LOTE da sonda das mídias (prompt 43): cada <receita>.ppm de PH2D_SONDA_OUT contra a foto do
Rebelle e o controlo ctl.png (o Wet Round à mesma largura, SECO — ponha-o no lote como
{"nome":"ctl","base":"Wet Round","raio":…}), nas métricas que contam (MS) + a largura ± 2 px, ordenado; no fim, as
que passam tudo pela distância ao Rebelle (soma |n−r| ÷ |c−r|).

uso: midias_lote.py <dir do lote> <docs/Painter/NewFeaturesWetPaint> [foto, omissão airbrush] [métricas,…] [prefixo]
  (o controlo é `<dir>/ctl.png`, ou `PH2D_LOTE_CTL=<nome>`; o prefixo filtra as receitas do lote)
⚠️ Meça SECO (PH2D_SONDA_SECA=1320) e lembre que o preset de fábrica traz a SUA lei de largura (43 §4).
"""
import sys, os, glob
sys.path.insert(0, sys.argv[2] + "/ferramentas")
from midias_regua import metricas
from PIL import Image
d = sys.argv[1]; F = sys.argv[2]
for p in glob.glob(d + "/*.ppm"):
    q = p[:-4] + ".png"
    if not os.path.exists(q) or os.path.getmtime(q) < os.path.getmtime(p):
        Image.open(p).save(q)
FOTO = sys.argv[3] if len(sys.argv) > 3 else "airbrush"
R = metricas(F + f"/fixtures/rebelle/{FOTO}.png")
C = metricas(d + "/" + os.environ.get("PH2D_LOTE_CTL", "ctl") + ".png")
MS = sys.argv[4].split(",") if len(sys.argv) > 4 else ["cob", "tex", "borda", "orla", "cv", "cruz"]
PREFIXO = sys.argv[5] if len(sys.argv) > 5 else ""
print("R   " + " ".join(f"{m} {R[m]:.3f}" for m in MS) + f" larg {R['larg']:.0f}")
print("C   " + " ".join(f"{m} {C[m]:.3f}" for m in MS) + f" larg {C['larg']:.0f}")
rows = []
for q in sorted(glob.glob(d + "/*.png")):
    n = os.path.basename(q)[:-4]
    if n == "ctl" or not n.startswith(PREFIXO): continue
    N = metricas(q)
    if N is None: print(n, "sem traço"); continue
    ok = []
    for m in MS:
        meio = 0.5 * (R[m] + C[m]); ok.append(abs(N[m] - R[m]) < abs(meio - R[m]))
    lw = abs(N["larg"] - R["larg"]) <= 2
    rows.append((sum(ok) + lw, n, N, ok, lw))
for s, n, N, ok, lw in sorted(rows, key=lambda r: (-r[0], abs(r[2]["tex"] - R["tex"]))):
    print(f"{n:<14} {s}/{len(MS) + 1} " + " ".join(f"{m} {N[m]:.3f}{'✓' if o else '✗'}" for m, o in zip(MS, ok)) + f" larg {N['larg']:.0f}{'✓' if lw else '✗'}")
# distância normalizada ao Rebelle (soma |n−r| ÷ |c−r| nas métricas que contam), só os que passam tudo
print("--- mais perto do Rebelle (passam 7/7) ---")
best = []
for s, n, N, ok, lw in rows:
    if all(ok) and lw:
        best.append((sum(abs(N[m] - R[m]) / abs(C[m] - R[m]) for m in MS), n))
for dist, n in sorted(best)[:8]:
    print(f"{n:<16} dist {dist:.2f}")
