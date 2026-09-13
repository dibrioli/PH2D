#!/usr/bin/env python3
"""O ORÁCULO de cor: as transformações de vista do Blender 5.2, corridas SEM interface.

Medido em 2026-09-13 (docs/Render3d/04 §3): OpenColorIO 2.5.1 (BSD-3-Clause) sobre o
`config.ocio` que o Blender instala. É o §0.9 do CLAUDE.md — corre-se, não se lê.

    python3 docs/Render3d/ferramentas/oraculo_de_cor.py

⚠️⚠️ **A armadilha que a 1.ª corrida pagou:** no PyOpenColorIO 2.x o `applyRGB` DEVOLVE a cor
nova e NÃO escreve no argumento. Ler o argumento depois da chamada imprime a ENTRADA, e as cinco
vistas saíram «idênticas à rampa» — plausível e falso. ⇒ o controlo abaixo (a vista `Standard`
de `0,5` linear tem de dar `0,7354`, o encode sRGB) corre ANTES de qualquer tabela.
"""

import PyOpenColorIO as ocio

CONFIG = "/usr/share/blender/5.2/datafiles/colormanagement/config.ocio"
SRC = "Linear Rec.709"
RAMP = [0.0, 0.01, 0.18, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0]
RED_HDR = [4.0, 0.2, 0.1]
VIEWS = ["Standard", "AgX", "Khronos PBR Neutral", "ACES 2.0", "Filmic"]

cfg = ocio.Config.CreateFromFile(CONFIG)


def processor(view: str):
    t = ocio.DisplayViewTransform(src=SRC, display="sRGB", view=view)
    return cfg.getProcessor(t).getDefaultCPUProcessor()


# O controlo: a API devolve, e o valor devolvido é o encode sRGB.
control = processor("Standard").applyRGB([0.5, 0.5, 0.5])[0]
assert abs(control - 0.7353581) < 1e-5, f"controlo falhou: Standard(0,5) = {control}"
print("OCIO", ocio.__version__, "· controlo Standard(0,5) =", round(control, 4))
print("entrada linear", RAMP)
for view in VIEWS:
    p = processor(view)
    grey = [round(p.applyRGB([v, v, v])[0], 4) for v in RAMP]
    red = [round(x, 3) for x in p.applyRGB(list(RED_HDR))]
    print(f"{view:20s} cinza -> {grey}  vermelho HDR {RED_HDR} -> {red}")
