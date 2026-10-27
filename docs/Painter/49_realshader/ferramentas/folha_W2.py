#!/usr/bin/env python3
"""Folha final da W2 (27/10): a cena do dono (vermelho empilhado Dabber + Oil Bristle) em três colunas —
hoje (o `oleo.ppm` da W1c, gravado em `oraculo/entrada/` a 26/10 22:41) | W2 (Estúdio, força do ambiente de
fábrica 0,6) | Cycles (`oraculo/saida/studio_sol10_cor_agx.png`, HDRI já sem a rotação de 180°) — e por baixo a
faixa do Oil Bristle (y 270..460) com o material do cartão: Metallic 1 · Clearcoat 1 (rug 0,03) · os dois.

As entradas W2 saem de UMA corrida da sonda (a cena pinta-se uma vez, cada variante grava a sua pasta):

    PH2D_OLEO_PNG=<w2> PH2D_OLEO_VARIANTES="estudio:;metal:mat_metal=1;verniz:mat_verniz=1,mat_verniz_rugosidade=0.03;metalverniz:mat_metal=1,mat_verniz=1,mat_verniz_rugosidade=0.03" \\
      bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib medida_oleo_real_oraculo -- --ignored --nocapture
    python3 -I docs/Painter/49_realshader/ferramentas/folha_W2.py <raiz do repo> <w2> <pasta de trabalho>
"""
import pathlib
import subprocess
import sys

RAIZ, W2, TMP = (pathlib.Path(a) for a in sys.argv[1:4])
O = RAIZ / "docs/Painter/49_realshader/oraculo"
SAIDA = RAIZ / "docs/Painter/49_realshader/out/W2_folha.jpg"
FAIXA = "700x190+0+270"


def magick(*a):
    subprocess.run(["magick", *map(str, a)], check=True)


def rotulada(img, texto, out, crop=None):
    extra = ["-gravity", "northwest", "-crop", crop, "+repage"] if crop else []
    magick("(", "-size", "700x30", "-background", "#1c1c1c", "-fill", "white", "-font", "DejaVu-Sans",
           "-pointsize", "17", "-gravity", "west", f"label: {texto}", ")", "(", img, *extra, ")",
           "-append", "-bordercolor", "#1c1c1c", "-border", "2", out)
    return out


TMP.mkdir(parents=True, exist_ok=True)
topo = [
    rotulada(O / "entrada/oleo.ppm", "hoje — W1c Estúdio (tag backup/oleo-real-W1c)", TMP / "a.png"),
    rotulada(W2 / "estudio/oleo.ppm", "W2 — Estúdio, força do ambiente 0,6 (fábrica)", TMP / "b.png"),
    rotulada(O / "saida/studio_sol10_cor_agx.png", "Cycles — studio_sol10 (mesma receita)", TMP / "c.png"),
]
faixa = [
    rotulada(W2 / "metal/oleo.ppm", "W2 faixa — Metallic 1", TMP / "d.png", FAIXA),
    rotulada(W2 / "verniz/oleo.ppm", "W2 faixa — Clearcoat 1 (rugosidade 0,03)", TMP / "e.png", FAIXA),
    rotulada(W2 / "metalverniz/oleo.ppm", "W2 faixa — Metallic 1 + Clearcoat 1", TMP / "f.png", FAIXA),
]
magick("(", *topo, "+append", ")", "(", *faixa, "+append", ")", "-append", "-quality", "90", SAIDA)
print(SAIDA)
