#!/usr/bin/env python3
# Agrega a matriz A×B: por cena × variante, mínimo e mediana (n) de cada fase; + «água» = p50/p95 do quadro (quadro_ms)
# nos 3 s depois do mouse-up, a partir dos quadros crus e das marcas.
import json, os, re, statistics as st, subprocess, sys
SP = os.path.dirname(os.path.abspath(__file__))
LW = "/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor"
M, OUT = f"{SP}/mede/matriz", f"{SP}/agr"
subprocess.run(["python3", f"{LW}/docs/Painter/ferramentas/mede_pen_up_4k.py", "--matriz", M, OUT], check=True,
               stdout=subprocess.DEVNULL)
def pct(v, q):
    v = sorted(v); return v[min(len(v) - 1, int(q * (len(v) - 1) + 0.5))] if v else float("nan")
cel = {}
for l in open(f"{OUT}/matriz.jsonl"):
    r = json.loads(l)
    m = re.match(r"([AB])(\d+)", r["rot"]); v = m.group(1)
    mk = {}
    for x in open(f"{M}/{r['rotulo']}.marcas"):
        p = x.split()
        if len(p) >= 2: mk[p[0]] = int(p[1])
    if "controlo_fim" not in mk: continue
    up = (mk["up"] - mk["down"]) / 1e6
    fr = [tuple(map(float, x.split(","))) for x in open(f"{OUT}/quadros/{r['rotulo']}.csv").readlines()[1:]]
    ag = [q for t, i, q in fr if up < t <= up + 3000]
    agi = [i for t, i, q in fr if up < t <= up + 3000]
    r["agua_q50"], r["agua_q95"], r["agua_int50"], r["agua_n"] = pct(ag, .5), pct(ag, .95), pct(agi, .5), len(ag)
    k = (r["tela"], r["pincel"], r["painel"])
    cel.setdefault(k, {}).setdefault(v, []).append(r)
F = ["toque", "traco_p50", "traco_p95", "soltar", "soltar_1s", "ctrlz", "agua_q50", "agua_q95", "agua_int50", "agua_hz"]
print("| cena | fase | A mín | A med | B mín | B med | n A/B | B−A (mín) |")
print("|---|---|---|---|---|---|---|---|")
for k in sorted(cel):
    for f in F:
        a = [r[f] for r in cel[k].get("A", []) if r.get(f) is not None]
        b = [r[f] for r in cel[k].get("B", []) if r.get(f) is not None]
        if not a or not b: continue
        print(f"| {k[1]}:{k[2]} {k[0]} | {f} | {min(a):.1f} | {st.median(a):.1f} | {min(b):.1f} | {st.median(b):.1f} |"
              f" {len(a)}/{len(b)} | {min(b)-min(a):+.1f} |")
print()
for k in sorted(cel):
    for v in "AB":
        print(k, v, "carga:", [r["carga"].split()[0] for r in cel[k].get(v, [])], "placa:",
              sorted({r["placa"][:40] for r in cel[k].get(v, [])}))
