#!/usr/bin/env python3
"""Grava a fixture que ATA a verdade analítica do gate `the_offset_sweep_matches_the_table` a um
rasterizador externo: a área (unidades²) de `S ∪ stroke(2d)` (d > 0) e `S ∖ stroke(2d)` (d < 0) da
estrela do gate, renderizada pelo `resvg`, para as 600 combinações (3 junções × 100 d × 2 lados).

Uso (da raiz da worktree):
    python3 "docs/Vector Module/ferramentas/vectorcraft_2026-10-11/verdade_resvg_estrela.py" \
        > crates/ph2d-vec-boolean/tests/it/fixtures/estrela_offset_resvg.tsv
"""
import datetime, math, os, subprocess, sys, tempfile
from concurrent.futures import ThreadPoolExecutor
import numpy as np
from PIL import Image

SCALE = 50.0  # px por unidade do documento
CX, R, RATIO = -2.2, 1.4, 0.45
pts = []
for i in range(10):
    a = -math.pi / 2 + i * math.pi / 5
    rr = R if i % 2 == 0 else R * RATIO
    pts.append((CX + rr * math.cos(a), rr * math.sin(a)))
D = "M" + " L".join(f"{x:.9f},{y:.9f}" for x, y in pts) + " Z"
TMP = tempfile.mkdtemp(prefix="verdade_resvg_")
VER = subprocess.run(["resvg", "--version"], capture_output=True, text=True).stdout.strip()


def area(join, sign, i):
    d = 0.04 * i
    half = 1.5 + d * (4.0 if join == "miter" else 1.05)
    w = int(math.ceil(2 * half * SCALE))
    vb = f"{CX - half} {-half} {2 * half} {2 * half}"
    if sign > 0:
        body = (f'<path d="{D}" fill="#000" stroke="#000" stroke-width="{2 * d}" '
                f'stroke-linejoin="{join}" stroke-miterlimit="4"/>')
    else:
        body = (f'<rect x="{CX - half}" y="{-half}" width="{2 * half}" height="{2 * half}" fill="#fff"/>'
                f'<path d="{D}" fill="#000"/>'
                f'<path d="{D}" fill="none" stroke="#fff" stroke-width="{2 * d}" '
                f'stroke-linejoin="{join}" stroke-miterlimit="4"/>')
    svg = (f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{w}" viewBox="{vb}">'
           f"{body}</svg>")
    p = os.path.join(TMP, f"{join}_{sign}_{i}.svg")
    open(p, "w").write(svg)
    subprocess.run(["resvg", p, p + ".png"], check=True)
    img = np.asarray(Image.open(p + ".png").convert("RGBA")).astype(np.int32)
    if sign > 0:
        n = int((img[:, :, 3] > 127).sum())
    else:
        n = int(((img[:, :, 3] > 127) & (img[:, :, 0] < 128)).sum())
    os.remove(p); os.remove(p + ".png")
    px = (2 * half / w) ** 2
    return join, sign, i, n * px


jobs = [(j, s, i) for j in ("miter", "round", "bevel") for s in (1, -1) for i in range(1, 101)]
with ThreadPoolExecutor(int(os.environ.get("JOBS", "6"))) as ex:
    rows = list(ex.map(lambda a: area(*a), jobs))
print(f"# origem: resvg {VER} (alpha > 127), {SCALE:g} px/unidade, estrela cook(Star,[-3.6,-1.4],[-0.8,1.4],[5,0.45,0])")
print(f"# comando: python3 \"docs/Vector Module/ferramentas/vectorcraft_2026-10-11/verdade_resvg_estrela.py\"")
print(f"# data: {datetime.date.today().isoformat()}")
print("# junção\tsinal\ti (d = 0,04·i)\tárea (unidades²)")
for j, s, i, a in rows:
    print(f"{j}\t{s}\t{i}\t{a:.6f}")
