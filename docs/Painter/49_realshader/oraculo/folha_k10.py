"""A folha K10: antes | depois | Cycles (mesma malha, ÷ o papel longe, sRGB) | Rebelle, por caso.

    python3 folha_k10.py <entrada_k10> <saida_k10> <rebelle dabber_3_tamanhos_A.png> <saida.jpg>

`<entrada_k10>/K10_{antes,depois}/<cena>/<amb>/oleo.ppm` (a sonda com `rebote_papel` 0 e o de fábrica, a MESMA tinta).
"""
import os
import subprocess
import sys
import tempfile

import numpy as np
from PIL import Image

import halo_regua as H
import rebote_regua as R

ent, sai, reb, out = sys.argv[1:5]
CASOS = [
    ("(a) Dabber 3 a 200, vãos · Estúdio", "dabber3", "estudio", "studio_sol10", (200, 215, 500, 365)),
    ("(b) faixa sob o planalto da faca · Estúdio", "travessia", "estudio", "studio_sol10", (150, 280, 450, 380)),
    ("(c) halo dos traços lisos · Clássico", "lisos", "classico", "sol10", (40, 30, 340, 190)),
    ("cena do dono, sombra do empaste · Clássico", "dono", "classico", "sol10", (250, 150, 550, 300)),
]
tmp = tempfile.mkdtemp()
linhas = []
for k, (rot, cena, amb, cy, (x0, y0, x1, y1)) in enumerate(CASOS):
    paineis = []
    for v in ("antes", "depois"):
        im = Image.open(f"{ent}/K10_{v}/{cena}/{amb}/oleo.ppm").convert("RGB").crop((x0, y0, x1, y1))
        paineis.append((f"{v} · {rot}" if v == "antes" else v, im))
    lin, _ = R.cycles(f"{sai}/{cena}/{cy}_cor.exr")
    tinta = np.asarray(Image.open(f"{ent}/K10_antes/{cena}/{amb}/cobertura.pgm")) > 25
    ref = np.median((lin @ R.W)[H.distancia(tinta) >= 41])
    cyc = R.srgb(lin / ref).round().astype(np.uint8)
    paineis.append(("Cycles (GI, mesma malha)", Image.fromarray(cyc).crop((x0, y0, x1, y1))))
    if cena == "dabber3":
        paineis.append(("Rebelle", Image.open(reb).convert("RGB").crop((350, 420, 650, 570))))
    else:
        paineis.append(("Rebelle: sem sombra no papel (K4)", Image.new("RGB", (x1 - x0, y1 - y0), (235, 235, 235))))
    fs = []
    for j, (t, im) in enumerate(paineis):
        f = f"{tmp}/p{k}_{j}.png"
        im.resize((im.width * 2, im.height * 2), Image.NEAREST).save(f)
        fs.append(f)
        subprocess.run(["magick", f, "-gravity", "North", "-background", "white", "-splice", "0x22",
                        "-pointsize", "16", "-annotate", "+0+2", t, f], check=True)
    lf = f"{tmp}/l{k}.png"
    subprocess.run(["magick", *fs, "+append", lf], check=True)
    linhas.append(lf)
subprocess.run(["magick", *linhas, "-append", "-quality", "88", out], check=True)
print(out, os.path.getsize(out))
