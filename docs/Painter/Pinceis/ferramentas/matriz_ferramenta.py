#!/usr/bin/env python3
"""O grupo «ferramenta» da MATRIZ (J4 `mistura`) — o que uma FERRAMENTA que não pinta (borracha, borrador,
misturador, água, secador) faz à tinta que já está na tela, na cena «sobre» da bateria.

O vector da matriz (`exame_matriz.vector`) é de tinta POSTA: o traço de uma borracha é «vazio» e a distância dela a
qualquer pincel saía NaN. Este grupo mede, nos cruzamentos 2 e 3 (x = 500, 750; o 1.º fica de fora — nas fotos antigas
do Rebelle ele saía ao tamanho da base, `fixtures/rebelle_mistura/README.md`) sobre as duas faixas (a molhada, y 150;
a seca, y 470), nas fotos A e B, as contas do `exames/mistura_foto.py`:

  resto  — o que fica da faixa no eixo (1 = intacta, 0 = papel)          largura — as colunas da faixa que mudaram
  borda  — as colunas com o resto entre 0,1 e 0,9, ÷ 2                    fundo   — px que o vermelho desceu abaixo
  acima  — px que o vermelho subiu acima da faixa                         orla_v  — a orla de cima no eixo − a intacta

Só entra no vector de uma FERRAMENTA (o `meta.json` da bancada diz `"ferramenta": true`; um oráculo, pela pasta do
Rebelle: BLEND/SMUDGE/ERASER/WATER/DRY): nos pincéis que pintam fica NaN e a distância entre eles NÃO muda (a média é
sobre os grupos que os dois têm). A ESCALA é PRÓPRIA (`matriz/escala_ferramenta.json`, congelada; `--recalibra` refá-la):
1,4826 · MAD de cada medida sobre as fotos das ferramentas do Rebelle, com o chão de [`CHAO`].

uso: matriz_ferramenta.py --recalibra          refaz matriz/escala_ferramenta.json a partir dos oráculos
     matriz_ferramenta.py <foto-ou-prefixo>…   imprime o vector de cada um
"""
import glob
import json
import os
import sys

import numpy as np
from PIL import Image

AQUI = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, AQUI)
from exame_regua import acha_foto  # noqa: E402

PINCEIS = os.path.dirname(AQUI)
ESCALA = os.path.join(PINCEIS, "matriz", "escala_ferramenta.json")
TOOLS = ("BLEND", "SMUDGE", "ERASER", "WATER", "DRY")
# O chão de cada medida (a escala nunca abaixo dele): o passo de 1 px nas larguras, 0,02 no resto.
CHAO = {"resto": 0.02, "largura": 1.0, "borda": 1.0, "fundo": 2.0, "acima": 2.0, "orla_v": 1.0}


def identidade(png):
    a = np.asarray(Image.open(png).convert("RGB")).astype(float)
    g = a[:, :, 1]
    out = {}
    for banda, y0 in (("molhada", 150), ("seca", 470)):
        ref = float(np.median(g[y0, 60:200]))
        ref_rgb = np.median(a[y0, 60:200], axis=0)

        def resto_de(v):
            return np.clip((255 - v) / max(1.0, 255 - ref), 0, 1)

        for xc in (500, 750):
            linha = resto_de(g[y0, xc - 80:xc + 81])
            k = f"{xc}.{banda}"
            out[f"{k}.resto"] = float(linha[78:83].mean())
            out[f"{k}.largura"] = float((np.abs(a[y0, xc - 80:xc + 81] - ref_rgb).max(axis=1) > 15).sum())
            out[f"{k}.borda"] = float(((linha > 0.1) & (linha < 0.9)).sum()) / 2
            col = g[:, xc - 20:xc + 21].min(axis=1)
            out[f"{k}.fundo"] = float(max([y - (y0 + 30) for y in range(y0 + 31, min(y0 + 150, g.shape[0]))
                                          if col[y] < 128] or [0]))
            out[f"{k}.acima"] = float(max([(y0 - 30) - y for y in range(max(y0 - 90, 0), y0 - 30)
                                          if col[y] < 200] or [0]))

            def orla(x):
                c = resto_de(g[y0 - 50:y0 + 1, x])
                return float(((c > 0.1) & (c < 0.9)).sum())

            out[f"{k}.orla_v"] = orla(xc) - orla(200)
    return out


def vector(caminho):
    """{chave: valor} da cena «sobre» (A e B) de uma pasta da bancada ou de um prefixo de oráculo; {} sem foto."""
    v = {}
    for foto in ("A", "B"):
        p = acha_foto(caminho, "sobre", foto)
        if p:
            for k, x in identidade(p).items():
                v[f"{k}.{foto}"] = x
    return v


def familia(chave):
    return chave.split(".")[2]


def oraculos():
    """As fotos «sobre» das ferramentas do Rebelle (o refeito em `rebelle_mistura` no lugar do antigo)."""
    base = os.path.join(PINCEIS, "fixtures")
    achados = {}
    for raiz in ("rebelle_bateria", "rebelle_mistura"):
        for t in TOOLS:
            for f in glob.glob(os.path.join(base, raiz, t, "*_sobre_A.png")):
                achados[(t, os.path.basename(f))] = f[: -len("_sobre_A.png")]
    return sorted(achados.values())


def recalibra():
    vs = [vector(p) for p in oraculos()]
    chaves = sorted({k for v in vs for k in v})
    X = np.array([[v.get(k, np.nan) for k in chaves] for v in vs])
    med = np.nanmedian(X, 0)
    mad = 1.4826 * np.nanmedian(np.abs(X - med), 0)
    esc = {k: max(float(m) if np.isfinite(m) else 0.0, CHAO[familia(k)]) for k, m in zip(chaves, mad)}
    os.makedirs(os.path.dirname(ESCALA), exist_ok=True)
    json.dump({"n_oraculos": len(vs), "chao": CHAO, "escala": esc}, open(ESCALA, "w"), indent=0)
    print(f"▸ {ESCALA}: {len(chaves)} medidas sobre {len(vs)} fotos de ferramentas do Rebelle")


if __name__ == "__main__":
    if sys.argv[1:] == ["--recalibra"]:
        recalibra()
    else:
        for c in sys.argv[1:]:
            print(c, json.dumps(vector(c)))
