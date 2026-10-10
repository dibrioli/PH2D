#!/usr/bin/env python3
"""W9 — mede as saídas de corre_fidelidade.mjs (saidas/w9_*.{0181,next}.{json,svg}) e imprime as tabelas
do estudo 07: a ponta presa (onde fica, o vão ao contorno, a mira), a ponta de seta (comprimento e
abertura × comprimento da seta × espessura), o fuste (recto?) e as setas desenhadas à mão.
Uso: python3 mede_w9.py [pasta_saidas]   (só números; nada é escrito)
"""
import json
import math
import re
import sys
from pathlib import Path

OUT = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent / "saidas"
VERS = ("0181", "next")


def load(caso, v):
    return json.loads((OUT / f"w9_{caso}.{v}.json").read_text())


def step(d, tipo, k=-1):
    ps = [p for p in d["passos"] if p["passo"]["tipo"] == tipo]
    return ps[k]["elementos"] if ps else None


def world(e):
    return [(e["x"] + p[0], e["y"] + p[1]) for p in e["points"]]


# ---- contornos (amostrados) -------------------------------------------------------------------
def outline(s, n=4000):
    x, y, w, h = s["x"], s["y"], s["width"], s["height"]
    t = s["type"]
    rnd = s.get("roundness")
    pts = []
    if t == "ellipse":
        cx, cy = x + w / 2, y + h / 2
        return [(cx + w / 2 * math.cos(2 * math.pi * i / n), cy + h / 2 * math.sin(2 * math.pi * i / n)) for i in range(n)]
    if t == "rectangle":
        r = 0.0
        if rnd:
            S = min(w, h)
            r = min(S / 4, 32) if rnd["type"] == 3 else S / 4
        c = [(x, y), (x + w, y), (x + w, y + h), (x, y + h)]
        for i in range(4):
            a, b, nx = c[i], c[(i + 1) % 4], c[(i + 2) % 4]
            L = math.dist(a, b)
            ua = ((b[0] - a[0]) / L, (b[1] - a[1]) / L)
            p0 = (a[0] + ua[0] * r, a[1] + ua[1] * r)
            p1 = (b[0] - ua[0] * r, b[1] - ua[1] * r)
            for k in range(200):
                f = k / 200
                pts.append((p0[0] + (p1[0] - p0[0]) * f, p0[1] + (p1[1] - p0[1]) * f))
            if r:  # quadrática com o controlo no vértice b
                L2 = math.dist(b, nx)
                ub = ((nx[0] - b[0]) / L2, (nx[1] - b[1]) / L2)
                q2 = (b[0] + ub[0] * r, b[1] + ub[1] * r)
                for k in range(60):
                    f = k / 60
                    pts.append(((1 - f) ** 2 * p1[0] + 2 * (1 - f) * f * b[0] + f * f * q2[0],
                                (1 - f) ** 2 * p1[1] + 2 * (1 - f) * f * b[1] + f * f * q2[1]))
        return pts
    if t == "diamond":
        v = [(x + w / 2, y), (x + w, y + h / 2), (x + w / 2, y + h), (x, y + h / 2)]
        cut = 0.25 if rnd else 0.0
        for i in range(4):
            a, b, nx = v[i], v[(i + 1) % 4], v[(i + 2) % 4]
            p0 = (a[0] + (b[0] - a[0]) * cut, a[1] + (b[1] - a[1]) * cut)
            p1 = (b[0] + (a[0] - b[0]) * cut, b[1] + (a[1] - b[1]) * cut)
            for k in range(300):
                f = k / 300
                pts.append((p0[0] + (p1[0] - p0[0]) * f, p0[1] + (p1[1] - p0[1]) * f))
            if cut:  # cúbica com c1 = c2 = vértice
                q2 = (b[0] + (nx[0] - b[0]) * cut, b[1] + (nx[1] - b[1]) * cut)
                for k in range(60):
                    f = k / 60
                    m = 1 - f
                    pts.append((m ** 3 * p1[0] + 3 * m * m * f * b[0] + 3 * m * f * f * b[0] + f ** 3 * q2[0],
                                m ** 3 * p1[1] + 3 * m * m * f * b[1] + 3 * m * f * f * b[1] + f ** 3 * q2[1]))
        return pts
    return pts


def inside(s, p):
    x, y, w, h = s["x"], s["y"], s["width"], s["height"]
    cx, cy = x + w / 2, y + h / 2
    if s["type"] == "ellipse":
        return ((p[0] - cx) / (w / 2)) ** 2 + ((p[1] - cy) / (h / 2)) ** 2 < 1
    if s["type"] == "diamond":
        return abs(p[0] - cx) / (w / 2) + abs(p[1] - cy) / (h / 2) < 1
    return x < p[0] < x + w and y < p[1] < y + h


def gap(s, p):
    d = min(math.dist(p, q) for q in outline(s))
    return -d if inside(s, p) else d


def seg_dist(p, a, b):
    ab = (b[0] - a[0], b[1] - a[1])
    L2 = ab[0] ** 2 + ab[1] ** 2
    t = max(0, min(1, ((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1]) / L2)) if L2 else 0
    return math.dist(p, (a[0] + ab[0] * t, a[1] + ab[1] * t))


def line_dist(p, a, b):
    ab = (b[0] - a[0], b[1] - a[1])
    L = math.hypot(*ab)
    return abs((p[0] - a[0]) * ab[1] - (p[1] - a[1]) * ab[0]) / L if L else math.dist(p, a)


# ---- SVG ---------------------------------------------------------------------------------------
def svg_groups(caso, v):
    s = (OUT / f"w9_{caso}.{v}.svg").read_text()
    gs = []
    for m in re.finditer(r'<g transform="translate\(([-\d.e]+) ([-\d.e]+)\) rotate\([^)]*\)"[^>]*>(.*?)</g>', s, re.S):
        tx, ty = float(m.group(1)), float(m.group(2))
        paths = re.findall(r'<path d="([^"]*)"', m.group(3))
        circles = re.findall(r'<circle[^>]*>', m.group(3)) + re.findall(r'<ellipse[^>]*>', m.group(3))
        gs.append((tx, ty, paths, m.group(3)))
    return gs


def nums(d):
    return [float(x) for x in re.findall(r"-?\d+(?:\.\d+)?(?:e-?\d+)?", d)]


def subpaths(d):
    return [sp for sp in re.split(r"(?=M)", d) if sp.strip()]


def orbit_end(s, P, Q):
    """⭐ A lei XB1: a ponta presa a `s` em modo orbit com o foco P (o fixedPoint no mundo), vinda da outra
    ponta Q: o 1.º ponto da recta Q→P que entra na faixa de vão g = 5 + strokeWidth(forma)/2 à volta do
    contorno (ou entra na forma); se a recta chega a P sem entrar, a ponta é o próprio P."""
    g = 5 + s["strokeWidth"] / 2
    ol = outline(s, 3000)[::2]

    def before(t):
        p = (Q[0] + (P[0] - Q[0]) * t, Q[1] + (P[1] - Q[1]) * t)
        return (not inside(s, p)) and min(math.dist(p, q) for q in ol) > g

    if before(1.0):
        return P
    lo, hi = 0.0, 1.0
    for _ in range(40):
        mid = (lo + hi) / 2
        lo, hi = (mid, hi) if before(mid) else (lo, mid)
    return (Q[0] + (P[0] - Q[0]) * lo, Q[1] + (P[1] - Q[1]) * lo)


def main():
    print("## B — a ponta presa depois de o editor a re-ligar (empurrar → ←)\n")
    print("| caso | versão | forma | ponta final | vão ao contorno | ligação gravada | dist. da recta (outra ponta → fixedPoint) |")
    print("|---|---|---|---|---|---|---|")
    B = sorted(p.name.split(".")[0][3:] for p in OUT.glob("w9_B_*.next.json"))
    for caso in B:
        for v in VERS:
            d = load(caso, v)
            els = step(d, "empurrar")
            shp = {e["id"]: e for e in els if e["type"] in ("rectangle", "ellipse", "diamond")}
            for e in els:
                if e["type"] != "arrow":
                    continue
                pts = world(e)
                for k, key in ((0, "startBinding"), (-1, "endBinding")):
                    b = e.get(key)
                    if not b or b["elementId"] not in shp:
                        continue
                    s = shp[b["elementId"]]
                    p = pts[k]
                    other = pts[-1 if k == 0 else 0]
                    fp = b.get("fixedPoint")
                    aim = "—"
                    if fp:
                        fw = (s["x"] + fp[0] * s["width"], s["y"] + fp[1] * s["height"])
                        aim = f"{line_dist(p, other, fw):.2f}"
                    bb = {kk: (round(vv, 4) if isinstance(vv, float) else vv) for kk, vv in b.items() if kk != "elementId"}
                    print(f"| {caso} | {v} | {s['type'][:4]} sw{e['strokeWidth']} | ({p[0]:.2f}, {p[1]:.2f}) | {gap(s, p):.2f} | `{json.dumps(bb)}` | {aim} |")
    print("\n## dono_bv — as três setas do dono\n")
    for v in VERS:
        d = load("dono_bv", v)
        for tipo in ("restore", "empurrar"):
            els = step(d, tipo)
            shp = {e["id"]: e for e in els if e["type"] in ("rectangle", "ellipse", "diamond")}
            for e in els:
                if e["type"] != "arrow":
                    continue
                pts = world(e)
                g = []
                for k, key in ((0, "startBinding"), (-1, "endBinding")):
                    b = e.get(key)
                    if b and b["elementId"] in shp:
                        g.append(f"{shp[b['elementId']]['type'][:4]} {gap(shp[b['elementId']], pts[k]):.2f}")
                print(f"- {v} {tipo:8} {e['id'][:6]} ({pts[0][0]:.1f},{pts[0][1]:.1f})→({pts[-1][0]:.1f},{pts[-1][1]:.1f}) vãos {g} start {e['startBinding']} end {e['endBinding']}")
    print("\n## XB1 contra os pontos GRAVADOS no ficheiro do dono (a recta entre os dois focos, recortada no vão)\n")
    els = json.loads((OUT.parent / "entradas" / "w9_dono_bv.excalidraw").read_text())["elements"]
    S = {e["id"]: e for e in els}
    fw = lambda s, fp: (s["x"] + fp[0] * s["width"], s["y"] + fp[1] * s["height"])  # noqa: E731
    for a in (e for e in els if e["type"] == "arrow"):
        pts = world(a)
        sa, sb = S[a["startBinding"]["elementId"]], S[a["endBinding"]["elementId"]]
        A, B = fw(sa, a["startBinding"]["fixedPoint"]), fw(sb, a["endBinding"]["fixedPoint"])
        e0, e1 = orbit_end(sa, A, B), orbit_end(sb, B, A)
        print(f"- {a['id'][:6]}: lei ({e0[0]:.2f},{e0[1]:.2f})→({e1[0]:.2f},{e1[1]:.2f}) · ficheiro ({pts[0][0]:.2f},{pts[0][1]:.2f})→({pts[-1][0]:.2f},{pts[-1][1]:.2f}) · erro {math.dist(e0, pts[0]):.2f} / {math.dist(e1, pts[-1]):.2f}")
    print("\n## P — a ponta `arrow`: comprimento das riscas e meia-abertura\n")
    print("| versão | espessura | L da seta | risca | meia-abertura ° | risca/L |")
    print("|---|---|---|---|---|---|")
    for v in VERS:
        d = load("P_comprimento", v)
        els = step(d, "restore")
        gs = svg_groups("P_comprimento", v)
        # cada seta: 1 grupo do fuste + 2 das riscas (pela ordem)
        gi = 0
        for e in els:
            shaft = gs[gi]
            gi += 1
            heads = []
            while gi < len(gs) and len(heads) < 2:
                tx, ty, paths, raw = gs[gi]
                if abs(ty - shaft[1]) > 1e-6 or abs(tx - shaft[0]) > 1e-6:
                    break
                heads.append(paths)
                gi += 1
            L = e["points"][-1][0]
            if not heads:
                print(f"| {v} | {e['strokeWidth']} | {L} | — | — | — |")
                continue
            n = nums(subpaths(heads[0][0])[0])
            p0 = (n[0], n[1])
            tip = (n[-2], n[-1])
            r = math.dist(p0, tip)
            ang = math.degrees(math.atan2(abs(p0[1] - tip[1]), abs(tip[0] - p0[0])))
            print(f"| {v} | {e['strokeWidth']} | {L} | {r:.2f} | {ang:.2f} | {r / L:.3f} |")
    print("\n## P_tipos — o que cada ponta vira (serializar) e quantos grupos/primitivas no SVG\n")
    for v in VERS:
        d = load("P_tipos", v)
        ser = step(d, "serializar")
        s = (OUT / f"w9_P_tipos.{v}.svg").read_text()
        print(f"- {v}: " + ", ".join(f"{e['id'][2:]}→{e['endArrowhead']}" for e in ser if e["id"].endswith("L200")))
    print("\n## F — o fuste: desvio máximo dos pontos do caminho à recta/polilinha dos pontos\n")
    for v in VERS:
        d = load("F_fuste", v)
        els = step(d, "restore")
        gs = svg_groups("F_fuste", v)
        order = []
        for g in gs:  # o 1.º grupo de cada translate é o fuste
            if not order or (g[0], g[1]) != (order[-1][0], order[-1][1]):
                order.append(g)
        for e, g in zip(els, order):
            n = nums(subpaths(g[2][0])[0]) if g[2] else []
            cp = [(n[i], n[i + 1]) for i in range(0, len(n) - 1, 2)]
            poly = [tuple(p) for p in e["points"]]
            dev = max((min(seg_dist(c, poly[i], poly[i + 1]) for i in range(len(poly) - 1)) for c in cp), default=float("nan"))
            print(f"- {v} {e['id']}: {len(poly)} pontos, roundness {e.get('roundness')}, rugosidade {e['roughness']}: afastamento máx. dos pontos do 1.º traço à polilinha = {dev:.2f} · `{subpaths(g[2][0])[0][:90].strip()}`")
    print("\n## M — à mão (o que o editor grava ao desenhar, e depois de empurrar)\n")
    for p in sorted(OUT.glob("w9_M*.next.json")):
        caso = p.name.split(".")[0][3:]
        for v in VERS:
            if not (OUT / f"w9_{caso}.{v}.json").exists():
                continue
            d = load(caso, v)
            for tipo in ("desenhar", "empurrar"):
                els = step(d, tipo)
                if els is None:
                    continue
                shp = {e["id"]: e for e in els if e["type"] in ("rectangle", "ellipse", "diamond")}
                for e in els:
                    if e["id"] in ("R", "E", "D"):
                        continue
                    if e["type"] != "arrow":
                        if tipo == "desenhar":
                            keep = {k: e[k] for k in e if k not in ("points", "pressures", "id", "seed", "index", "created")}
                            print(f"- {v} {caso}: {json.dumps(keep, ensure_ascii=False)}")
                        continue
                    pts = world(e)
                    g = []
                    for k, key in ((0, "startBinding"), (-1, "endBinding")):
                        b = e.get(key)
                        if b and b["elementId"] in shp:
                            g.append(f"{key[:-7]} {shp[b['elementId']]['type'][:4]} vão {gap(shp[b['elementId']], pts[k]):.2f} {json.dumps({kk: (round(vv, 4) if isinstance(vv, float) else vv) for kk, vv in b.items() if kk != 'elementId'})}")
                    print(f"- {v} {caso} {tipo}: ({pts[0][0]:.2f},{pts[0][1]:.2f})→({pts[-1][0]:.2f},{pts[-1][1]:.2f}) rnd {e.get('roundness')} · " + " · ".join(g))


main()
