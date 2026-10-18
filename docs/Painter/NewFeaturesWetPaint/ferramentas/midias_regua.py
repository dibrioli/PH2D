#!/usr/bin/env python3
"""A régua das MÍDIAS (prompt 17, Fase A/C): cada traço NOSSO contra a foto do Rebelle da sua mídia e contra o
CONTROLO (o Wet Round à mesma largura), nas métricas de `mede_traco.py` sobre o recto da bancada (03 §1).

A BARRA de uma métrica de identidade é o VALE (05 §3): passa quem fica do lado do Rebelle do ponto médio entre
o Rebelle e o controlo, e o controlo tem de ficar FORA (senão a métrica não distingue a mídia e não conta).

uso: midias_regua.py <dir dos nossos .png> <dir fixtures/rebelle> <controlo> <nosso:rebelle:m1,m2,…>…
  métricas: cob (cobertura do miolo) · tex (textura %) · borda (borda/miolo) · orla (px) · cv · per (fração do
  período) · larg (px) · cruz (luminância do CRUZAMENTO vermelho×azul ÷ a menor das duas fora dele: < 1 = escurece,
  como o marcador e a tinta do Rebelle; ~1 = o de cima tapa) · ponta (o RECUO da ponta, prompt 44: quanto
  a ponta a ±0,7 da meia-largura fica atrás da do eixo, ÷ a meia-largura, média das 4 pontas isoladas — o
  traço RÁPIDO e o vermelho recto; redonda ≈ 0,29, reta ≈ 0)
"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from mede_traco import mede  # noqa: E402

RECTO = (20, 60, 580, 120)
CHAVE = {"cob": "cobertura_miolo", "tex": "textura_pct", "borda": "borda_escura", "orla": "orla_10_90_px",
         "cv": "uniformidade_cv", "per": "periodo_fracao", "larg": "largura_mediana_px"}


def lum(c):
    c = np.asarray(c) / 255.0
    c = np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)
    return float(c @ [0.2126, 0.7152, 0.0722])


def cruzamento(png):
    """O miolo do cruzamento (x 294..306, y 474..486) contra o vermelho (x 800..860) e o azul (x 120..200) fora dele."""
    a = np.asarray(Image.open(png).convert("RGB")).astype(float)
    med = lambda x0, x1: a[474:487, x0:x1].reshape(-1, 3).mean(0)
    return lum(med(294, 307)) / max(min(lum(med(800, 860)), lum(med(120, 200))), 1e-6)


# As pontas isoladas da bancada: o traço RÁPIDO (y 300, x 600→960) e o vermelho recto (y 480, x 700→960). O
# recto de cima não serve: a ponta direita toca a curva em S. Recorte: x0, y0, x1, y1.
PONTAS = ((575, 260, 990, 340), (650, 455, 990, 505))
FRACAO_DA_PONTA = 0.7


def recuo_das_pontas(c):
    """(esquerda, direita) de um traço horizontal na cobertura `c` do recorte: a máscara é `c` acima de metade
    do p90 da tinta; o eixo e a meia-largura vêm das linhas com mais de metade da máxima; a ponta de uma
    linha é a 1.ª/última coluna onde ≥ 2 das 3 linhas da faixa estão na máscara."""
    tinta = c[c > 0.08]
    mask = c > 0.5 * np.percentile(tinta, 90)
    prof = mask.sum(1)
    linhas = np.where(prof > 0.5 * prof.max())[0]
    yc, hw = (linhas[0] + linhas[-1]) / 2, (linhas[-1] - linhas[0] + 1) / 2

    def pontas(dy):
        e, d = [], []
        for sg in ((0,) if dy == 0 else (-1, 1)):
            r0 = int(round(yc + sg * dy))
            # a linha cai numa FALHA entre fios (W2): a linha com tinta mais perto, até 3 px
            for o in (0, -1, 1, -2, 2, -3, 3):
                r = min(max(r0 + o, 1), mask.shape[0] - 2)
                xs = np.where(mask[r - 1:r + 2].sum(0) >= 2)[0]
                if len(xs):
                    break
            e.append(xs[0]); d.append(xs[-1])
        return np.mean(e), np.mean(d)

    eixo, fora = pontas(0), pontas(FRACAO_DA_PONTA * hw)
    return (fora[0] - eixo[0]) / hw, (eixo[1] - fora[1]) / hw


def ponta(png):
    a = np.asarray(Image.open(png).convert("RGB")).astype(float)
    r = []
    for x0, y0, x1, y1 in PONTAS:
        s = a[y0:y1, x0:x1]
        papel = np.median(np.concatenate([s[:2].reshape(-1, 3), s[-2:].reshape(-1, 3)]), 0)
        r.extend(recuo_das_pontas(np.abs(s - papel).max(2) / 255.0))
    return float(np.mean(r))


def metricas(png):
    r = mede(png, RECTO)
    if "erro" in r:
        return None
    m = {k: r[v] for k, v in CHAVE.items()}
    m["cruz"] = cruzamento(png)
    m["ponta"] = ponta(png)
    return m


def main():
    nossos, rebelle, controlo = sys.argv[1], sys.argv[2], sys.argv[3]
    c = metricas(controlo if controlo.endswith(".png") else os.path.join(nossos, controlo + ".png"))
    print(f"controlo {controlo}: " + " ".join(f"{k} {v:.3f}" for k, v in c.items()))
    for spec in sys.argv[4:]:
        nosso, reb, ms = spec.split(":")
        n = metricas(os.path.join(nossos, nosso + ".png"))
        r = metricas(os.path.join(rebelle, reb + ".png"))
        if n is None or r is None:
            print(f"{nosso:<10} vs {reb:<12} traço não encontrado")
            continue
        cols, dentro = [], True
        for m in ms.split(","):
            meio = 0.5 * (r[m] + c[m])
            distingue = abs(c[m] - r[m]) > 1e-9
            ok = distingue and abs(n[m] - r[m]) < abs(meio - r[m])
            dentro &= ok
            ctl = "" if distingue else " (não distingue)"
            cols.append(f"{m} {n[m]:.3f}·R {r[m]:.3f}·C {c[m]:.3f} {'✓' if ok else '✗'}{ctl}")
        print(f"{nosso:<10} vs {reb:<12} larg {n['larg']:.0f}/{r['larg']:.0f} · " + " · ".join(cols)
              + f" ⇒ {'DENTRO' if dentro else 'FORA'}")


if __name__ == "__main__":
    main()
