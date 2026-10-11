#!/usr/bin/env python3
"""Juiz das DISCORDÂNCIAS do Offset entre o VectorCraft e o PH2D: cada lado contra a verdade
`fill ∪ stroke(2d)` (d > 0) / `fill ∖ stroke(2d)` (d < 0), mesma junção, miterlimit 4, no resvg.

Entradas: o corpus (`target/vectorcraft/corpus/offset/*.svg`), a fixture do oráculo e o NOSSO
resultado gravado por `OURS_DIR=target/vectorcraft/ours cargo test … prints_both_sides -- --ignored`.
Uso: python3 -I juiz_resvg_offset.py <fixture.tsv> [limiar % = 0.5]
"""
import os, re, subprocess, sys, tempfile
import numpy as np
from PIL import Image

CORPUS = os.environ.get("VC_CORPUS", "target/vectorcraft/corpus")
OURS = os.environ.get("OURS_DIR", "target/vectorcraft/ours")
TMP = tempfile.mkdtemp(prefix="juiz_")
HDR = '<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="1600" viewBox="-400 -400 1600 1600">'
PATH_RE = re.compile(r"<path\b([^>]*)>")
ATTR_RE = re.compile(r'([a-z-]+)="([^"]*)"')


def paths(svg):
    return [dict(ATTR_RE.findall(m.group(1))) for m in PATH_RE.finditer(svg)]


def mask(body, name):
    p = os.path.join(TMP, name + ".svg")
    open(p, "w").write(HDR + '<rect x="-400" y="-400" width="1600" height="1600" fill="#fff"/>' + body + "</svg>")
    subprocess.run(["resvg", p, p + ".png"], check=True)
    img = np.asarray(Image.open(p + ".png").convert("RGB"))
    return img[:, :, 0] < 128


def fill(ps):
    return "".join(f'<path d="{a["d"]}" fill="#000" fill-rule="{a.get("fill-rule", "nonzero")}"/>' for a in ps)


rows = [l.rstrip("\n").split("\t") for l in open(sys.argv[1]) if not l.startswith("#")]
lim = float(sys.argv[2]) if len(sys.argv) > 2 else 0.5
print("caso\tjunção\td\tVC×verdade %\tPH2D×verdade %")
for kind, case, join, d, rc, ds in rows:
    if kind != "offset":
        continue
    src = paths(open(os.path.join(CORPUS, "offset", case + ".svg")).read())
    dd = float(d) * 100.0
    s = src[0]
    stroke = (f'<path d="{s["d"]}" fill="none" stroke="{{c}}" stroke-width="{2 * abs(dd)}" '
              f'stroke-linejoin="{join}" stroke-miterlimit="4"/>')
    truth = mask(fill(src) + (stroke.format(c="#000") if dd > 0 else stroke.format(c="#fff")), f"t_{case}_{join}_{d}")
    theirs = [dict(zip(("fill-rule", "d"), x.split(":", 1))) for x in ds.split("|") if x] if ds else []
    vc = mask(fill(theirs), f"v_{case}_{join}_{d}")
    ours = mask(fill(paths(open(os.path.join(OURS, f"{case}_{join}_{d}.svg")).read())), f"o_{case}_{join}_{d}")
    ref = max(int((truth if dd > 0 else mask(fill(src), "s")).sum()), 1)
    dv, do = 100 * (vc ^ truth).sum() / ref, 100 * (ours ^ truth).sum() / ref
    if dv > lim or do > lim:
        print(f"{case}\t{join}\t{d}\t{dv:.3f}\t{do:.3f}")
