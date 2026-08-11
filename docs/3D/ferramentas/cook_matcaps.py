#!/usr/bin/env python3
"""Coze os matcaps de `crates/ph2d-mesh-render/assets/matcaps/`.

Este script NAO roda em nenhum build — ele existe para a §1 de
`assets/matcaps/LICENSES.md` ser conferivel em vez de copiada, e para a proxima
pessoa que quiser acrescentar um matcap nao ter de re-descobrir a lei de
espaco de cor.

    python3 docs/3D/ferramentas/cook_matcaps.py <dir-de-trabalho>

BAIXAR AS FONTES
================

Os `.exr` do Blender vivem em git-lfs, e o `git clone` normal traz apenas
ponteiros. Sem o `git-lfs` instalado, o conteudo sai do servidor LFS do proprio
Blender em duas etapas (o mirror do GitHub NAO hospeda os objetos):

    # 1. o ponteiro (traz o oid e o tamanho)
    curl -sL https://raw.githubusercontent.com/blender/blender/v5.2.0/\\
release/datafiles/studiolights/matcap/basic_bright.exr

    # 2. o objeto, pelo oid do passo 1
    curl -sL -o basic_bright.exr \\
      https://projects.blender.org/blender/blender.git/info/lfs/objects/<oid>

O do SculptGL e' um arquivo comum:

    https://github.com/stephaneginier/sculptgl -> app/resources/matcaps/matcapFV.jpg

A LEI
=====

Duas fontes, uma saida: **sRGB de 8 bits**, porque a GPU devolve linear de graca
num formato `...UnormSrgb` e o shader quer linear.

- Blender: cena-referida LINEAR, com `diffuse` e `specular` em camadas
  separadas. O matcap e' a SOMA (o nosso caminho de matcap nao multiplica por
  cor de vertice, que e' para o que a separacao serve no Blender). Depois:
  transferencia sRGB.
- SculptGL: ja' e' sRGB autorado ⇒ so' re-embalado. Aplicar a transferencia o
  clarearia DUAS vezes.

⚠️ O `sum.max` medido dos oito e' **0,941** — nada e' cortado pelo clamp, e e'
por isso que 8 bits bastam. O script imprime a coluna para isso continuar sendo
uma medicao e nao uma lembranca.
"""

import hashlib
import os
import sys

import numpy as np
import OpenEXR
from PIL import Image

# (stem de origem, stem de saida) — o do Blender e' `grey`, o rotulo do app e'
# `Basic Gray`; o stem preserva a procedencia.
BLENDER = [
    "basic_bright", "basic_dark", "basic_grey", "basic_side",
    "clay_brown", "clay_green", "clay_warm", "red_wax",
]


def linear_to_srgb(x):
    """A transferencia sRGB, com o joelho — nao um `x ** (1/2.2)`."""
    x = np.clip(x, 0.0, 1.0)
    return np.where(x <= 0.0031308, x * 12.92, 1.055 * np.power(x, 1 / 2.4) - 0.055)


def main(work):
    out = os.path.join(work, "cooked")
    os.makedirs(out, exist_ok=True)
    print("%-14s %8s %8s  %s" % ("stem", "sum.max", "png KiB", "sha256 da fonte"))

    jpg = os.path.join(work, "matcapFV.jpg")
    if os.path.exists(jpg):
        arr = np.asarray(Image.open(jpg).convert("RGB"))
        Image.fromarray(arr, "RGB").save(os.path.join(out, "sculptgl_fv.png"), optimize=True)
        digest = hashlib.sha256(open(jpg, "rb").read()).hexdigest()
        size = os.path.getsize(os.path.join(out, "sculptgl_fv.png")) // 1024
        print("%-14s %8s %8d  %s" % ("sculptgl_fv", "sRGB", size, digest))

    for stem in BLENDER:
        src = os.path.join(work, stem + ".exr")
        if not os.path.exists(src):
            continue
        with OpenEXR.File(src) as x:
            ch = x.parts[0].channels
            d = ch["diffuse"].pixels.astype(np.float32)[..., :3]
            s = ch["specular"].pixels.astype(np.float32)[..., :3]
        total = d + s
        srgb = np.rint(linear_to_srgb(total) * 255.0).astype(np.uint8)
        Image.fromarray(srgb, "RGB").save(os.path.join(out, stem + ".png"), optimize=True)
        digest = hashlib.sha256(open(src, "rb").read()).hexdigest()
        size = os.path.getsize(os.path.join(out, stem + ".png")) // 1024
        print("%-14s %8.3f %8d  %s" % (stem, total.max(), size, digest))


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else ".")
