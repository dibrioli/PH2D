#!/usr/bin/env python3
"""Oráculo do bico cortado (F12 A): o `resvg` (Apache-2.0/MIT, `pacman -Qi resvg`) desenha uma
quina de ângulo interno `a` com `stroke-linejoin` miter e miter-clip, e mede o TOPO do traço.
Caixa preta — só a saída. Uso: python3 corre_resvg.py > fixtures/miter_clip_resvg.json
"""
import json, math, subprocess, sys, tempfile, os
from PIL import Image

W, LIMIT, VX, VY, LEG = 24.0, 4.0, 150.0, 60.0, 200.0
ANGLES = [60, 40, 30, 29, 28, 25, 20, 10, 4, 0]


def top(svg):
    with tempfile.TemporaryDirectory() as d:
        s, p = os.path.join(d, "q.svg"), os.path.join(d, "q.png")
        open(s, "w").write(svg)
        subprocess.run(["resvg", s, p], check=True)
        im = Image.open(p).convert("L")
        w, h = im.size
        return next(y for y in range(h) for x in range(w) if im.getpixel((x, y)) < 128)


out = []
for a in ANGLES:
    h = math.radians(a / 2)
    ax, ay = VX - LEG * math.sin(h), VY + LEG * math.cos(h)
    bx, by = VX + LEG * math.sin(h), VY + LEG * math.cos(h)
    row = {"interior_deg": a}
    for j in ["miter", "miter-clip"]:
        svg = (f'<svg xmlns="http://www.w3.org/2000/svg" width="300" height="300">'
               f'<rect width="300" height="300" fill="white"/>'
               f'<path d="M {ax:.5f} {ay:.5f} L {VX} {VY} L {bx:.5f} {by:.5f}" fill="none" '
               f'stroke="black" stroke-width="{W}" stroke-linejoin="{j}" '
               f'stroke-miterlimit="{LIMIT}"/></svg>')
        row[j.replace("-", "_") + "_top_y"] = top(svg)
    out.append(row)

ver = subprocess.run(["resvg", "--version"], capture_output=True, text=True).stdout.strip()
json.dump({
    "_cabecalho": {
        "oraculo": f"resvg {ver}", "licenca": "Apache-2.0 / MIT (pacman -Qi resvg) — caixa preta",
        "como": "corre_resvg.py: uma quina, pernas de 200 px, vértice em (150, 60), y para BAIXO",
        "largura": W, "limite": LIMIT, "medida": "1.ª linha de píxel com tinta < 128 (±1 px)",
        "data": "2026-10-11",
    },
    "experiencias": out,
}, sys.stdout, ensure_ascii=False, indent=1)
