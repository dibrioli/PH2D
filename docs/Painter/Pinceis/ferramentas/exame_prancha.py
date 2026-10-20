#!/usr/bin/env python3
"""A PRANCHA do EXAME (00_MISSAO §3.1/§3.5): por cena e por foto (A molhada, B seca), uma linha com o ORÁCULO · o
NOSSO · o VIZINHO mais perto da biblioteca (a matriz), a meia escala; e a versão CEGA — as mesmas linhas sem rótulos,
cada uma numa ordem SORTEADA, com a chave num JSON à parte (para a revisão cega, `05_REVISAO_CEGA.md`).

uso: exame_prancha.py --nosso <dir> --vizinho <dir> [--oraculo <dir>] --saida <png> [--cega <png> --chave <json>]
     [--titulo-nosso T] [--titulo-vizinho T] [--titulo-oraculo T] [--bateria b.json] [--semente N]
"""
import argparse
import json
import os
import random
import sys

from PIL import Image, ImageDraw, ImageFont

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from exame_regua import BATERIA, acha_foto, cenas_e_fotos  # noqa: E402

W, H, TOPO, MARGEM = 500, 320, 26, 6


def fonte(tam):
    for f in ("/usr/share/fonts/TTF/DejaVuSans.ttf", "/usr/share/fonts/dejavu/DejaVuSans.ttf",
              "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", "/usr/share/fonts/noto/NotoSans-Regular.ttf"):
        if os.path.exists(f):
            return ImageFont.truetype(f, tam)
    return ImageFont.load_default()


def abre(d, cena, foto):
    p = acha_foto(d, cena, foto) if d else None
    if not p:
        im = Image.new("RGB", (W, H), (235, 235, 235))
        ImageDraw.Draw(im).text((W // 2 - 40, H // 2 - 8), "sem foto", fill=(120, 120, 120), font=fonte(16))
        return im
    return Image.open(p).convert("RGB").resize((W, H), Image.LANCZOS)


def folha(linhas, rotulos_col, rotulo_linha):
    n_col = len(linhas[0][1])
    out = Image.new("RGB", (n_col * (W + MARGEM) + MARGEM, len(linhas) * (H + TOPO + MARGEM) + MARGEM), "white")
    d = ImageDraw.Draw(out)
    f = fonte(15)
    for i, (rot, ims) in enumerate(linhas):
        y = MARGEM + i * (H + TOPO + MARGEM)
        for j, im in enumerate(ims):
            x = MARGEM + j * (W + MARGEM)
            out.paste(im, (x, y + TOPO))
            d.rectangle([x - 1, y + TOPO - 1, x + W, y + TOPO + H], outline=(160, 160, 160))
            txt = rotulos_col(i, j)
            d.text((x + 4, y + 4), (rotulo_linha(rot) + " · " if rotulo_linha(rot) else "") + txt, fill="black",
                   font=f)
    return out


def grava(im, caminho):
    """256 cores (a bateria é azul, vermelho e papel): a prancha cabe em ~1 MB sem perder o grão que se vê."""
    im.quantize(colors=256, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE).save(caminho, optimize=True)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--nosso", required=True); ap.add_argument("--vizinho", required=True)
    ap.add_argument("--oraculo"); ap.add_argument("--saida", required=True)
    ap.add_argument("--cega"); ap.add_argument("--chave")
    ap.add_argument("--titulo-nosso", default="PH2D"); ap.add_argument("--titulo-vizinho", default="vizinho")
    ap.add_argument("--titulo-oraculo", default="oráculo"); ap.add_argument("--bateria", default=BATERIA)
    ap.add_argument("--semente", type=int, default=None)
    x = ap.parse_args()
    bat = json.load(open(x.bateria))
    fontes = [("oraculo", x.oraculo, f"ORÁCULO · {x.titulo_oraculo}"), ("nosso", x.nosso, f"NOSSO · {x.titulo_nosso}"),
              ("vizinho", x.vizinho, f"VIZINHO · {x.titulo_vizinho}")]
    cf = list(cenas_e_fotos(bat))
    linhas = [((c, f), [abre(d, c, f) for _, d, _ in fontes]) for c, f in cf]
    nome_foto = {"A": "A molhada", "B": "B seca"}
    grava(folha(linhas, lambda i, j: fontes[j][2], lambda r: f"{r[0]} {nome_foto.get(r[1], r[1])}"), x.saida)
    print(x.saida)
    if not x.cega:
        return
    rng = random.Random(x.semente if x.semente is not None else int.from_bytes(os.urandom(4), "little"))
    chave, cegas = {}, []
    # Só as linhas com as TRÊS fotos: um «sem foto» denunciaria o lado que falta.
    completas = [((c, f), ims) for (c, f), ims in linhas if all(d and acha_foto(d, c, f) for _, d, _ in fontes)]
    if not completas:
        print("sem linha completa: a prancha cega não se faz")
        return
    cf = [r for r, _ in completas]
    for (c, f), ims in completas:
        ordem = [0, 1, 2]
        rng.shuffle(ordem)
        cegas.append(((c, f), [ims[k] for k in ordem]))
        chave[f"{c}_{f}"] = {str(p + 1): fontes[k][0] for p, k in enumerate(ordem)}
    grava(folha(cegas, lambda i, j: str(j + 1),
                lambda r: f"linha {cf.index(r) + 1} ({r[0]} {nome_foto.get(r[1], r[1])})"), x.cega)
    json.dump({"prancha": os.path.basename(x.cega), "linhas": [f"{c}_{f}" for c, f in cf], "chave": chave,
               "quem": {"oraculo": x.titulo_oraculo, "nosso": x.titulo_nosso, "vizinho": x.titulo_vizinho}},
              open(x.chave, "w"), indent=1, ensure_ascii=False)
    print(x.cega, x.chave)


if __name__ == "__main__":
    main()
