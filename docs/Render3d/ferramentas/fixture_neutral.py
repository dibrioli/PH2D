#!/usr/bin/env python3
"""Grava a fixture do ORÁCULO da vista `Khronos PBR Neutral` — nos NÓS do LUT.

    python3 docs/Render3d/ferramentas/fixture_neutral.py \
        > crates/ph2d-view-transform/fixtures/ocio_neutral_nodes.txt

# Por que nos NÓS, e não numa rede qualquer (medido 2026-09-13, docs/Render3d/05 §2)

O Blender 5.2 define a vista como `lg2 [-9, 10]` → um LUT `57³` com interpolação tetraédrica →
sRGB. Num NÓ do LUT a interpolação é exacta, e o que sobra entre a fórmula publicada e o oráculo é a
impressão de 7 casas do `.cube` mais a cadeia `f32` do OCIO: **máximo `8,9e-6`** (57 cinzas e 2 000
cores). FORA dos nós o oráculo erra até **`0,030`** (mediana `1,4e-4`) — é a interpolação DELE, não a
lei. ⇒ a fixture guarda só nós, e a barra do gate (`2e-5`) mora no vão entre as duas populações.

Formato (uma linha por amostra, depois do cabeçalho `#`): `r g b  R G B` — a entrada em luz LINEAR
da cena (Rec.709) e a saída do oráculo em sRGB CODIFICADO, `{:.9e}`.
"""

import hashlib
import random

import PyOpenColorIO as ocio

CONFIG = "/usr/share/blender/5.2/datafiles/colormanagement/config.ocio"
LUT = "/usr/share/blender/5.2/datafiles/colormanagement/luts/pbrNeutral.cube"
NODES = 57
LG2_MIN, LG2_MAX = -9.0, 10.0
COLOURED = 400
SEED = 20260913


def sha256(path: str) -> str:
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def node(k: int) -> float:
    return 2.0 ** (k / (NODES - 1) * (LG2_MAX - LG2_MIN) + LG2_MIN)


cfg = ocio.Config.CreateFromFile(CONFIG)
t = ocio.DisplayViewTransform(src="Linear Rec.709", display="sRGB", view="Khronos PBR Neutral")
p = cfg.getProcessor(t).getDefaultCPUProcessor()

# O controlo que a 1.ª sonda não tinha: o `applyRGB` DEVOLVE (docs/Render3d/04 §3).
std = cfg.getProcessor(
    ocio.DisplayViewTransform(src="Linear Rec.709", display="sRGB", view="Standard")
).getDefaultCPUProcessor()
assert abs(std.applyRGB([0.5, 0.5, 0.5])[0] - 0.7353581) < 1e-5

rows = [[node(k)] * 3 for k in range(NODES)]
rng = random.Random(SEED)
rows += [[node(rng.randrange(NODES)) for _ in range(3)] for _ in range(COLOURED)]

print("# ORÁCULO: a vista `Khronos PBR Neutral` do Blender 5.2, corrida pelo OpenColorIO sem interface")
print(f"# OpenColorIO {ocio.__version__} (BSD-3-Clause) · config.ocio sha256 {sha256(CONFIG)}")
print(f"# pbrNeutral.cube sha256 {sha256(LUT)}")
print(f"# entrada: os NÓS do LUT, lg2 [{LG2_MIN:g}, {LG2_MAX:g}] em {NODES} passos — {NODES} cinzas + "
      f"{COLOURED} cores (semente {SEED})")
print("# gerado por docs/Render3d/ferramentas/fixture_neutral.py · formato: r g b  R G B "
      "(linear da cena · sRGB codificado)")
for r in rows:
    out = p.applyRGB(list(r))
    print(" ".join(f"{v:.9e}" for v in r) + "  " + " ".join(f"{v:.9e}" for v in out))
