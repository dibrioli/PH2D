#!/usr/bin/env python3
"""W23 bloco 6 — resume as saídas do `corre_bloco6.mjs` em tabelas (Markdown) e compara o «Draw to shape»
tocado pelo RATO com o elemento que a API (`recognizeShape` + `convertToShape`) deu ao MESMO traço
(`oraculo_quickshape/fixtures_excalidraw/<nome>.txt`, `elemento_ponta_corrente_arrow`), desfeita a translação.

Uso: python3 bloco6_resume.py [saidas_dir]
"""
import json
import os
import sys

AQUI = os.path.dirname(os.path.abspath(__file__))
SAI = sys.argv[1] if len(sys.argv) > 1 else os.path.join(AQUI, "saidas")
API = os.path.abspath(os.path.join(AQUI, "../../../UI_New_and_Simple/vetor_nova_ui/oraculo_quickshape/fixtures_excalidraw"))

laco = json.load(open(os.path.join(SAI, "bloco6_laco.next.json")))
print("## Laço —", laco["cabecalho"]["versao"], "· entrada sha256", laco["cabecalho"]["entrada_sha256"][:12])
print("| caso | contain | overlap |")
print("|---|---|---|")
por = {}
for r in laco["resultados"]:
    por.setdefault(r["caso"], {})[r["boxSelectionMode"]] = r
for caso, m in por.items():
    cel = lambda r: ", ".join(r["selectedElementIds"]) + (f" (grupo {', '.join(r['selectedGroupIds'])})" if r["selectedGroupIds"] else "") or "—"
    print(f"| {caso} | {cel(m['contain']) or '—'} | {cel(m['overlap']) or '—'} |")

traco = json.load(open(os.path.join(SAI, "bloco6_traco.next.json")))
rs = [r for r in traco["resultados"] if "traco" in r]
auto = {r["traco"]: r for r in rs if r["ferramenta"] == "autoshape"}
livre = {r["traco"]: r for r in rs if r["ferramenta"] == "freedraw"}


def api_elem(nome):
    for l in open(os.path.join(API, f"{nome}.txt"), encoding="utf-8"):
        if l.startswith("# elemento_ponta_corrente_arrow:"):
            return json.loads(l.split(":", 1)[1])
    return None


print()
print("## Draw to shape —", traco["cabecalho"]["versao"], "· entrada sha256", traco["cabecalho"]["entrada_sha256"][:12])
print("| traço | rato: Draw to shape | x,y,w,h,ângulo (ecrã) | API convertToShape | Δ geometria (px, desfeita a translação) | controlo Draw |")
print("|---|---|---|---|---|---|")
maxd = 0.0
iguais = difer = 0
for nome in sorted(auto):
    r = auto[nome]
    a = api_elem(nome)
    tx = r["transformacao"]
    if r["elementos"]:
        e = r["elementos"][0]
        geo = f"{e['x']:.1f},{e['y']:.1f},{e['width']:.1f},{e['height']:.1f},{e['angle']:.3f}"
        tipo = r["tipo"] + (f" {e.get('startArrowhead')}→{e.get('endArrowhead')}" if e["type"] == "arrow" else "")
    else:
        e, geo, tipo = None, "—", "nenhum (o traço some)"
    d = "—"
    if e and a and a["type"] == e["type"]:
        dd = max(abs(e["x"] - tx["dx"] - a["x"]), abs(e["y"] - tx["dy"] - a["y"]), abs(e["width"] - a["w"]), abs(e["height"] - a["h"]), abs(e["angle"] - a["angle"]))
        if "points" in e and "points" in a:
            pe = [[e["x"] + p[0] - tx["dx"], e["y"] + p[1] - tx["dy"]] for p in e["points"]]
            if len(pe) == len(a["points"]):
                dd = max(dd, max(max(abs(p[0] - q[0]), abs(p[1] - q[1])) for p, q in zip(pe, a["points"])))
            else:
                dd = float("nan")
        d = f"{dd:.3f}"
        maxd = max(maxd, dd) if dd == dd else maxd
    at = a["type"] if a else "?"
    if (e["type"] if e else "nenhum") == at:
        iguais += 1
    else:
        difer += 1
    c = livre.get(nome)
    ctrl = c["tipo"] if c else "?"
    print(f"| {nome} | {tipo} | {geo} | {at} | {d} | {ctrl} |")
print()
print(f"tipo rato = tipo API: {iguais}/{iguais + difer} (convertToShape «nenhum» = recognizeShape «freedraw»); Δ geometria máx {maxd:.3f} px;"
      f" controlo Draw → freedraw: {sum(1 for r in livre.values() if r['tipo'] == 'freedraw')}/{len(livre)}")
