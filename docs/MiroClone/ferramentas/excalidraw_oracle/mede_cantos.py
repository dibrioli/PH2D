#!/usr/bin/env python3
"""Mede os cantos das formas nas fixtures do oráculo Excalidraw (roughness 0) — só lê saidas/.

Uso: python3 mede_cantos.py [pasta_saidas]
Lê formas_canto_retangulo / formas_canto_losango / formas_contorno_elipse (.svg + .restored.json)
e imprime: raio do canto do rectângulo por lado menor; construção do canto do losango (vértices,
tipo de curva, corte ao longo de cada aresta); erro do contorno da elipse contra a elipse analítica.
Cada <g> do SVG é um elemento, na ordem do restored.json (conferido pelo centro do `rotate`).
"""
import json
import math
import re
import sys
from pathlib import Path

DIR = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent / "saidas"
NUM = r"-?\d+(?:\.\d+)?(?:e-?\d+)?"


def groups(name):
    """[(elemento, tx, ty, [segmentos])] — segmento = ('L'|'C', p0, c1, c2, p3) em coords locais."""
    svg = (DIR / f"{name}.svg").read_text()
    els = json.loads((DIR / f"{name}.restored.json").read_text())
    gs = re.findall(
        rf'<g stroke-linecap="round" transform="translate\(({NUM}) ({NUM})\) rotate\(0 ({NUM}) ({NUM})\)">'
        r'<path d="([^"]*)" stroke="[^"]*" stroke-width="([^"]*)"',
        svg,
    )
    assert len(gs) == len(els), f"{name}: {len(gs)} grupos SVG vs {len(els)} elementos"
    out = []
    for el, (tx, ty, cx, cy, d, sw) in zip(els, gs):
        assert abs(float(cx) - el["width"] / 2) < 1e-6 and abs(float(cy) - el["height"] / 2) < 1e-6, el["id"]
        segs, cur = [], None
        for cmd, args in re.findall(r"([MC])([^MC]*)", d):
            v = [float(x) for x in re.findall(NUM, args)]
            if cmd == "M":
                cur = (v[0], v[1])
            else:
                for k in range(0, len(v), 6):
                    c1, c2, p3 = (v[k], v[k + 1]), (v[k + 2], v[k + 3]), (v[k + 4], v[k + 5])
                    segs.append((cur, c1, c2, p3))
                    cur = p3
        out.append((el, float(tx), float(ty), float(sw), segs))
    return out


def collinear(s, eps=0.02):
    """Os controlos a menos de `eps` da corda (o SVG traz 2 casas: arredondamento ≤ 0,005·√2)."""
    p0, c1, c2, p3 = s
    L = math.hypot(p3[0] - p0[0], p3[1] - p0[1])
    def dist(c):
        return abs((p3[0] - p0[0]) * (c[1] - p0[1]) - (p3[1] - p0[1]) * (c[0] - p0[0])) / L
    return dist(c1) < eps and dist(c2) < eps


def bez(s, t):
    p0, c1, c2, p3 = s
    u = 1 - t
    return tuple(u**3 * p0[i] + 3 * u * u * t * c1[i] + 3 * u * t * t * c2[i] + t**3 * p3[i] for i in (0, 1))


def dedupe(segs):
    """O roughjs desenha cada aresta duas vezes (multiStroke); fica uma de cada (p0, p3)."""
    seen, out = set(), []
    for s in segs:
        k = (s[0], s[3])
        if k not in seen:
            seen.add(k)
            out.append(s)
    return out


def rect_table():
    print("## Rectângulo — raio do canto (lado menor S)")
    print(f"{'id':<22}{'w':>6}{'h':>6}{'S':>6}{'raio':>8}{'r/S':>8}  canto")
    for el, tx, ty, sw, segs in groups("formas_canto_retangulo"):
        curves = [s for s in dedupe(segs) if not collinear(s)]
        S = min(el["width"], el["height"])
        if not curves:
            print(f"{el['id']:<22}{el['width']:>6g}{el['height']:>6g}{S:>6g}{0:>8g}{0:>8.3f}  agudo (sw={sw})")
            continue
        radii, quad = set(), True
        for p0, c1, c2, p3 in curves:
            corner = (p3[0], p0[1]) if abs(c1[1] - p0[1]) < 1e-6 else (p0[0], p3[1])
            r = max(abs(p3[0] - p0[0]), abs(p3[1] - p0[1]))
            radii.add(round(r, 3))
            # quadrática elevada: c1 = p0 + 2/3 (canto − p0), c2 = p3 + 2/3 (canto − p3); 2 casas no SVG.
            for p, c in ((p0, c1), (p3, c2)):
                for i in (0, 1):
                    if abs(p[i] + 2 / 3 * (corner[i] - p[i]) - c[i]) > 0.006:
                        quad = False
        r = max(radii)
        kind = "quadrática, controlo no vértice" if quad else "NÃO quadrática"
        print(f"{el['id']:<22}{el['width']:>6g}{el['height']:>6g}{S:>6g}{r:>8g}{r / S:>8.3f}  {kind}; raios {sorted(radii)}")


def diamond_table():
    print("\n## Losango — vértices e corte do canto (topo = (tx, 0), direita = (w, ry))")
    print(f"{'id':<20}{'w':>5}{'h':>5}{'tx':>7}{'ry':>7}{'corte dx':>10}{'corte dy':>10}{'dx/tx':>8}{'dy/ry':>8}"
          f"{'fora da aresta':>16}  curva")
    for el, tx_, ty_, sw, segs in groups("formas_canto_losango"):
        w, h = el["width"], el["height"]
        ds = dedupe(segs)
        corners = [s for s in ds if not collinear(s)]
        if not corners:
            pts = {s[0] for s in ds} | {s[3] for s in ds}
            top = min(pts, key=lambda p: p[1])
            right = max(pts, key=lambda p: p[0])
            print(f"{el['id']:<20}{w:>5g}{h:>5g}{top[0]:>7g}{right[1]:>7g}{'—':>10}{'—':>10}{'—':>8}{'—':>8}{'—':>16}  agudo")
            continue
        assert len(corners) == 4, el["id"]
        both = all(s[1] == s[2] for s in corners)
        top_c = min(corners, key=lambda s: s[1][1])
        right_c = max(corners, key=lambda s: s[1][0])
        top, right = top_c[1], right_c[1]
        # o canto do topo vai de (tx−dx, dy) a (tx+dx, dy)
        dx = (top_c[3][0] - top_c[0][0]) / 2
        dy = top_c[0][1] - top[1]
        # distância do ponto de corte (tx+dx, dy) à recta topo→direita
        q = (top[0] + dx, top[1] + dy)
        ex, ey = right[0] - top[0], right[1] - top[1]
        off = abs(ex * (q[1] - top[1]) - ey * (q[0] - top[0])) / math.hypot(ex, ey)
        kind = "cúbica, c1 = c2 = vértice" if both else "OUTRA"
        print(f"{el['id']:<20}{w:>5g}{h:>5g}{top[0]:>7g}{right[1]:>7g}{dx:>10g}{dy:>10g}{dx / top[0]:>8.4f}"
              f"{dy / right[1]:>8.4f}{off:>16.3f}  {kind}")


def ellipse_table():
    print("\n## Elipse — contorno contra a elipse analítica (centro w/2,h/2; semi-eixos w/2,h/2)")
    for el, tx, ty, sw, segs in groups("formas_contorno_elipse"):
        a, b = el["width"] / 2, el["height"] / 2
        worst, n = 0.0, 0
        for s in segs:
            for k in range(33):
                x, y = bez(s, k / 32)
                # distância (aprox. de 1.ª ordem) ao contorno: F / |∇F|
                F = ((x - a) / a) ** 2 + ((y - b) / b) ** 2 - 1
                g = math.hypot(2 * (x - a) / a**2, 2 * (y - b) / b**2)
                worst = max(worst, abs(F) / g)
                n += 1
        print(f"{el['id']:<14} {el['width']:g}x{el['height']:g}: {len(segs)} cúbicas, {n} amostras, "
              f"desvio máx {worst:.4f} (stroke-width {sw}; translate {tx:g},{ty:g})")


if __name__ == "__main__":
    rect_table()
    diamond_table()
    ellipse_table()
