#!/usr/bin/env python3
"""As fotografias da sonda do azul-claro do Wet Drip (prompt 16, Fase B): `sonda_mistura_wet_drip` com
`PH2D_SONDA_OUT=<dir_ppm>` grava `<variante>_<momento>.ppm` (momento = passadas · t120 · escorre · seco, e as
cenas de uma tinta só `<variante>_soazul_*` / `_sovermelho_*`).

Saída em <dir_png>:
  · `<variante>_1x.png`  — a tela inteira 1:1 nos quatro momentos, lado a lado;
  · `<variante>_8x.png`  — as quatro zonas dos sintomas a 8× (orla de baixo a t120 · fios · pontas · fronteira);
  · `prancha_1x.png` / `prancha_8x.png` — as variantes pedidas em linhas (as quatro caixas, molhado e seco).

uso: prancha_mistura_wet_drip.py <dir_ppm> <dir_png> [variante …]   (sem variantes = todas as do dir)
"""
import os
import sys

from PIL import Image, ImageDraw

MOMENTOS = ["passadas", "t120", "escorre", "seco"]
# As zonas dos quatro sintomas (coordenadas da sonda, razão 1); a do `_umpasse` encosta no vermelho (y 195–240).
ZONAS = [
    ("1 pontas", "escorre", (200, 320, 250, 344)),
    ("2 orla de baixo", "t120", (300, 228, 350, 252)),
    ("3 fios", "escorre", (250, 290, 300, 320)),
    ("4 fronteira", "escorre", (400, 226, 450, 270)),
]
ZONAS_UM_PASSE = [
    ("1 pontas no vermelho", "escorre", (360, 195, 420, 240)),
    ("2 orla de baixo", "t120", (300, 195, 350, 215)),
    ("3 fios", "escorre", (300, 195, 350, 235)),
    ("4 fronteira", "escorre", (220, 205, 270, 240)),
]
PRANCHA = [
    "nenhuma_molhado_borda1", "km_molhado_borda1", "glaze_molhado_borda1", "duas_molhado_borda1",
    "nenhuma_seco_borda1", "duas_seco_borda1", "duas_molhado_borda1_viva", "duas_molhado_borda1_soazul",
    "nenhuma_molhado_borda1_umpasse", "duas_molhado_borda1_umpasse",
]


def abre(dirp, nome, mom):
    return Image.open(os.path.join(dirp, f"{nome}_{mom}.ppm")).convert("RGB")


def um_por_um(dirp, nome):
    tl = [abre(dirp, nome, m) for m in MOMENTOS]
    w, h = tl[0].size
    out = Image.new("RGB", (w * len(tl), h + 16), "white")
    d = ImageDraw.Draw(out)
    for j, (m, t) in enumerate(zip(MOMENTOS, tl)):
        out.paste(t, (j * w, 16))
        d.text((j * w + 4, 2), f"{nome} · {m}", fill="black")
    return out


def oito(dirp, nome, escala=8):
    zonas = ZONAS_UM_PASSE if nome.endswith("_umpasse") else ZONAS
    tiles = []
    for lab, mom, box in zonas:
        t = abre(dirp, nome, mom).crop(box)
        tiles.append((lab, t.resize(((box[2] - box[0]) * escala, (box[3] - box[1]) * escala), Image.NEAREST)))
    w = sum(t.size[0] for _, t in tiles) + 10 * (len(tiles) - 1)
    h = max(t.size[1] for _, t in tiles) + 16
    out = Image.new("RGB", (w, h), "white")
    d = ImageDraw.Draw(out)
    x = 0
    for lab, t in tiles:
        out.paste(t, (x, 16))
        d.text((x + 4, 2), f"{nome} · {lab}", fill="black")
        x += t.size[0] + 10
    return out


def empilha(imgs, saida):
    w = max(i.size[0] for i in imgs)
    h = sum(i.size[1] for i in imgs) + 6 * len(imgs)
    out = Image.new("RGB", (w, h), "white")
    y = 0
    for i in imgs:
        out.paste(i, (0, y))
        y += i.size[1] + 6
    out.save(saida)


def main():
    dirp, dirs, *nomes = sys.argv[1:]
    os.makedirs(dirs, exist_ok=True)
    if not nomes:
        nomes = sorted({f[: -len("_escorre.ppm")] for f in os.listdir(dirp) if f.endswith("_escorre.ppm")
                        and "_soazul" not in f and "_sovermelho" not in f})
        nomes += [n + "_soazul" for n in nomes if n == "duas_molhado_borda1"]
    for n in nomes:
        um_por_um(dirp, n).save(os.path.join(dirs, f"{n}_1x.png"))
        if "razao" not in n:
            oito(dirp, n).save(os.path.join(dirs, f"{n}_8x.png"))
    linhas = [n for n in PRANCHA if os.path.exists(os.path.join(dirp, f"{n}_escorre.ppm"))]
    empilha([um_por_um(dirp, n).crop((0, 0, 600 * 4, 416)) for n in linhas], os.path.join(dirs, "prancha_1x.png"))
    empilha([oito(dirp, n) for n in linhas], os.path.join(dirs, "prancha_8x.png"))


if __name__ == "__main__":
    main()
