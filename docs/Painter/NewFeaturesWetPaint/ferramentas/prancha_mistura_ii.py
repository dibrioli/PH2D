#!/usr/bin/env python3
"""A prancha final da «Mistura II» (prompt 16): as quatro capturas do dono · o defeito reproduzido (antes) · depois ·
o Rebelle, 1:1 e a 8×. As PPM vêm da sonda `sonda_mistura_wet_drip` (`PH2D_SONDA_OUT=<dir_ppm>`, variantes
`faseE_antes_duas_*` e `faseE_duas_*`, colunas molhado/seco); as capturas e o Rebelle vêm de `fixtures/`.

uso (a partir de docs/Painter/NewFeaturesWetPaint/):
  python3 -I ferramentas/prancha_mistura_ii.py <dir_ppm> [antes_duas duas]
saída: prancha_mistura_wet_drip.png (1:1) e prancha_mistura_wet_drip_8x.png.
"""
import importlib.util
import os
import sys

from PIL import Image, ImageDraw

AQUI = os.path.dirname(os.path.abspath(__file__))
RAIZ = os.path.dirname(AQUI)
spec = importlib.util.spec_from_file_location("prancha", os.path.join(AQUI, "prancha_mistura_wet_drip.py"))
P = importlib.util.module_from_spec(spec)
spec.loader.exec_module(P)

DONO = [
    "1_pontas_dos_fios_ciano.png",
    "2_faixa_ciano_na_orla_de_baixo.png",
    "3_fios_ciano_pixelados.png",
    "4_halo_claro_na_fronteira.png",
]
# Rebelle, na ordem dos sintomas: ponta · orla de baixo · fios (junção) · fronteira.
REB8 = ["ponta", "orla_D", "juncao", "fronteira"]
TITULOS = ["1 pontas dos fios", "2 orla de baixo", "3 fios", "4 fronteira"]


def abre(p):
    return Image.open(os.path.join(RAIZ, "fixtures", p)).convert("RGB")


def faixa(titulo, imgs, gap=10, fundo="white"):
    w = sum(i.size[0] for i in imgs) + gap * (len(imgs) - 1)
    h = max(i.size[1] for i in imgs) + 18
    out = Image.new("RGB", (w, h), fundo)
    d = ImageDraw.Draw(out)
    d.text((4, 3), titulo, fill="black")
    x = 0
    for i in imgs:
        out.paste(i, (x, 18))
        x += i.size[0] + gap
    return out


def empilha(fs, gap=8):
    w = max(f.size[0] for f in fs)
    h = sum(f.size[1] for f in fs) + gap * (len(fs) - 1)
    out = Image.new("RGB", (w, h), "white")
    y = 0
    for f in fs:
        out.paste(f, (0, y))
        y += f.size[1] + gap
    return out


def a_altura(im, h):
    return im.resize((round(im.size[0] * h / im.size[1]), h), Image.LANCZOS)


def a_largura(im, w):
    return im.resize((w, round(im.size[1] * w / im.size[0])), Image.LANCZOS)


def nossa(dirp, var, mom="escorre"):
    return P.abre(dirp, var, mom)


def prancha_1x(dirp, antes, depois):
    dono = faixa("o dono (capturas do ecra, reduzidas a altura 400; nao sao 1:1)",
                 [a_altura(abre(f"smoke_dono_mistura_wet_drip/{n}"), 400) for n in DONO])
    blocos = [dono]
    for tipo, reb in (("molhado", "mol100_b.png"), ("seco", "seco100_b.png")):
        blocos.append(faixa(
            f"{tipo} - ANTES (as duas caixas, o defeito) | DEPOIS | Rebelle (oraculo)",
            [nossa(dirp, f"{antes}_{tipo}"), nossa(dirp, f"{depois}_{tipo}"),
             abre(f"rebelle_mistura_wet_drip/{reb}")]))
    return empilha(blocos)


def celula(im, w, h):
    c = Image.new("RGB", (w, h), "white")
    c.paste(im, (0, 0))
    return c


def prancha_8x(dirp, antes, depois):
    colw, colh = 480, 448
    blocos = []
    d = [a_largura(abre(f"smoke_dono_mistura_wet_drip/{n}"), colw) for n in DONO]
    blocos.append(faixa("o dono (capturas, largura 480)  |  " + "  |  ".join(TITULOS),
                        [celula(i, colw, max(x.size[1] for x in d)) for i in d]))
    for tipo, reb in (("molhado", "mol100"), ("seco", "seco100")):
        for rot, var in (("ANTES", antes), ("DEPOIS", depois)):
            cel = []
            for lab, mom, box in P.ZONAS:
                t = P.abre(dirp, f"{var}_{tipo}", mom).crop(box)
                cel.append(celula(t.resize(((box[2] - box[0]) * 8, (box[3] - box[1]) * 8), Image.NEAREST),
                                  colw, colh))
            blocos.append(faixa(f"{tipo} - {rot} (8x, vizinho mais proximo)", cel))
        cel = [celula(abre(f"rebelle_mistura_wet_drip/{reb}_8x_{z}.png"), colw, colh) for z in REB8]
        blocos.append(faixa(f"{tipo} - Rebelle (8x)", cel))
    return empilha(blocos)


def main():
    dirp = sys.argv[1]
    antes, depois = (sys.argv[2:4] if len(sys.argv) >= 4 else ("faseE_antes_duas", "faseE_duas"))
    p1 = prancha_1x(dirp, antes, depois)
    p8 = prancha_8x(dirp, antes, depois)
    p1.save(os.path.join(RAIZ, "prancha_mistura_wet_drip.png"), optimize=True)
    p8.save(os.path.join(RAIZ, "prancha_mistura_wet_drip_8x.png"), optimize=True)
    print(p1.size, p8.size)


if __name__ == "__main__":
    main()
