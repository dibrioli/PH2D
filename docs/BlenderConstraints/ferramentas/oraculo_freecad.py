"""Oráculo FreeCAD do sketch da W4 — CORRE-SE, nunca se lê o fonte (FreeCAD 1.1.4, LGPL-2.0, `pacman -Qi freecad`).

Uso (sem placa, sem interface):
    W4_CENAS="$PWD/docs/BlenderConstraints/fixtures/w4_cenas" \
    W4_SAIDA="$PWD/docs/BlenderConstraints/fixtures/freecad_1_1_4_sketch_arrasto.json" \
    bash scripts/ph2d-run.sh freecadcmd docs/BlenderConstraints/ferramentas/oraculo_freecad.py

As cenas são NOSSAS (geradas por `bancada_solvers -- cenas`). Cada ponto vira um `Part.Point`; cada segmento referido
por uma regra vira uma recta com as pontas coincidentes aos pontos (o idioma do Sketcher); o arrasto é
`SketchObject.moveGeometry` (o `movePoint` de antes do 1.x) passo a passo, como o rato. Grava: o tempo de cada passo, a pior violação (a MESMA régua
da bancada) e as posições nos passos 15/30/45/60.
"""

import hashlib
import json
import math
import os
import time

import FreeCAD as App
import Part
import Sketcher

CENAS = ["escada_50", "escada_202", "escada_802", "misto_50", "misto_200", "misto_800"]
AMOSTRAS = (15, 30, 45, 60)
V = lambda p: App.Vector(p[0], p[1], 0)


def dist(a, b):
    return math.hypot(a[0] - b[0], a[1] - b[1])


def dpr(p, a, b):
    return abs((b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0])) / dist(a, b)


def violacao(c, x, r):
    t = r["t"]
    g = lambda k: x[r[k]]
    if t == "h":
        return abs(g("a")[1] - g("b")[1])
    if t == "v":
        return abs(g("a")[0] - g("b")[0])
    if t in ("paralelo", "perpendicular"):
        u = (g("b")[0] - g("a")[0], g("b")[1] - g("a")[1])
        v = (g("d")[0] - g("c")[0], g("d")[1] - g("c")[1])
        k = u[0] * v[1] - u[1] * v[0] if t == "paralelo" else u[0] * v[0] + u[1] * v[1]
        return abs(k) / dist(g("c"), g("d"))
    if t == "distancia":
        return abs(dist(g("a"), g("b")) - r["d"])
    if t == "igual":
        return abs(dist(g("a"), g("b")) - dist(g("c"), g("d")))
    if t == "sobre_linha":
        return dpr(g("p"), g("a"), g("b"))
    if t == "simetrico":
        a, b, p, q = g("a"), g("b"), g("p"), g("q")
        d = (b[0] - a[0], b[1] - a[1])
        s = ((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / (d[0] ** 2 + d[1] ** 2)
        pe = (a[0] + s * d[0], a[1] + s * d[1])
        return dist((2 * pe[0] - p[0], 2 * pe[1] - p[1]), q)
    if t == "tangente":
        ci = c["circulos"][r["circulo"]]
        return abs(dpr(x[ci["centro"]], g("a"), g("b")) - ci["raio"])
    if t == "coincidente":
        return dist(g("a"), g("b"))
    raise ValueError(t)


def monta(doc, c):
    sk = doc.addObject("Sketcher::SketchObject", c["nome"])
    pts = sk.addGeometry([Part.Point(V(p)) for p in c["pontos"]], False)
    linhas, geo, cons = {}, [], []

    def linha(a, b):
        if (a, b) not in linhas:
            linhas[(a, b)] = len(pts) + len(geo)
            geo.append(("l", a, b))
        return linhas[(a, b)]

    circs = []
    for ci in c["circulos"]:
        circs.append(len(pts) + len(geo))
        geo.append(("c", ci["centro"], ci["raio"]))
    for r in c["regras"]:
        t = r["t"]
        if t in ("h", "v"):
            cons.append(Sketcher.Constraint("Horizontal" if t == "h" else "Vertical", pts[r["a"]], 1, pts[r["b"]], 1))
        elif t in ("paralelo", "perpendicular"):
            cons.append(Sketcher.Constraint("Parallel" if t == "paralelo" else "Perpendicular",
                                            linha(r["a"], r["b"]), linha(r["c"], r["d"])))
        elif t == "distancia":
            cons.append(Sketcher.Constraint("Distance", pts[r["a"]], 1, pts[r["b"]], 1, r["d"]))
        elif t == "igual":
            cons.append(Sketcher.Constraint("Equal", linha(r["a"], r["b"]), linha(r["c"], r["d"])))
        elif t == "sobre_linha":
            cons.append(Sketcher.Constraint("PointOnObject", pts[r["p"]], 1, linha(r["a"], r["b"])))
        elif t == "simetrico":
            cons.append(Sketcher.Constraint("Symmetric", pts[r["p"]], 1, pts[r["q"]], 1, linha(r["a"], r["b"])))
        elif t == "tangente":
            cons.append(Sketcher.Constraint("Tangent", linha(r["a"], r["b"]), circs[r["circulo"]]))
        elif t == "coincidente":
            cons.append(Sketcher.Constraint("Coincident", pts[r["a"]], 1, pts[r["b"]], 1))
    partes = []
    for gk in geo:
        if gk[0] == "l":
            partes.append(Part.LineSegment(V(c["pontos"][gk[1]]), V(c["pontos"][gk[2]])))
        else:
            partes.append(Part.Circle(V(c["pontos"][gk[1]]), App.Vector(0, 0, 1), gk[2]))
    ids = sk.addGeometry(partes, False)
    ligar = []
    for gid, gk in zip(ids, geo):
        if gk[0] == "l":
            ligar.append(Sketcher.Constraint("Coincident", gid, 1, pts[gk[1]], 1))
            ligar.append(Sketcher.Constraint("Coincident", gid, 2, pts[gk[2]], 1))
        else:
            ligar.append(Sketcher.Constraint("Coincident", gid, 3, pts[gk[1]], 1))
            ligar.append(Sketcher.Constraint("Radius", gid, gk[2]))
    for i in c["fixos"]:
        ligar.append(Sketcher.Constraint("DistanceX", pts[i], 1, c["pontos"][i][0]))
        ligar.append(Sketcher.Constraint("DistanceY", pts[i], 1, c["pontos"][i][1]))
    sk.addConstraint(ligar + cons)
    return sk, pts


def corre(doc, c):
    sk, pts = monta(doc, c)
    rc = sk.solve()
    k = c["arrasto"]["ponto"]
    tempos, pior, amostras = [], 0.0, []
    for s, alvo in enumerate(c["arrasto"]["caminho"], start=1):
        t0 = time.perf_counter()
        sk.moveGeometry(pts[k], 1, V(alvo))  # devolve None: o sucesso prova-se pela violação abaixo
        tempos.append((time.perf_counter() - t0) * 1e3)
        x = [(sk.getPoint(g, 1).x, sk.getPoint(g, 1).y) for g in pts]
        v = max([violacao(c, x, rr) for rr in c["regras"]] + [dist(x[k], alvo)]
                + [dist(x[i], c["pontos"][i]) for i in c["fixos"]])
        pior = max(pior, v)
        if s in AMOSTRAS:
            amostras.append([list(p) for p in x])
    return {
        "solve_inicial": rc,
        "graus_de_liberdade_livres": sk.DoF, "redundantes": list(sk.RedundantConstraints), "conflitos": list(sk.ConflictingConstraints),
        "pior_passo_ms": max(tempos),
        "media_passo_ms": sum(tempos) / len(tempos),
        "passos_ms": tempos,
        "pior_violacao_px": pior,
        "amostras": amostras,
    }


def main():
    d, saida = os.environ["W4_CENAS"], os.environ["W4_SAIDA"]
    doc = App.newDocument("w4")
    out = {"cabecalho": {
        "oraculo": "FreeCAD " + ".".join(App.Version()[:3]) + " (freecadcmd, Sketcher/planegcs), LGPL-2.0 — corrido, nunca lido",
        "gerador": "docs/BlenderConstraints/ferramentas/oraculo_freecad.py",
        "entradas": "docs/BlenderConstraints/fixtures/w4_cenas (bancada_solvers -- cenas)",
        "arrasto": "SketchObject.moveGeometry por passo (cada chamada reinicia o arrasto: limite SUPERIOR do tempo do Sketcher)",
        "data": time.strftime("%Y-%m-%d"),
        "sha256": {},
    }, "cenas": {}}
    for nome in CENAS:
        txt = open(os.path.join(d, nome + ".json"), "rb").read()
        out["cabecalho"]["sha256"][nome] = hashlib.sha256(txt).hexdigest()
        out["cenas"][nome] = corre(doc, json.loads(txt))
        r = out["cenas"][nome]
        print(nome, "solve", r["solve_inicial"], "pior passo ms %.3f" % r["pior_passo_ms"],
              "viol %.2e" % r["pior_violacao_px"], "DoF", r["graus_de_liberdade_livres"], flush=True)
    json.dump(out, open(saida, "w"), indent=1)


main()
