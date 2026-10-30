#!/usr/bin/env python3
"""SIMULAÇÃO da cura do Over Oil (J9, 30/10) por retoque da imagem da sonda, antes de construir (METODOLOGIA 2).

A lei medida no Rebelle (`over_oil_regua.py` + alfa por píxel, canal B): a aguada sobre o óleo é «cobrir com
transparência» (`óleo·(1 − α) + aguada·α`, doc 47) com o MESMO α que tem no papel a meio do traço (0,35 × 0,38); assentada,
α cai para ~0,55× no óleo (0,20 × 0,36) e espalha-se 0,07–0,45: a água foge dos picos e junta-se nos vales.

Variantes (cada uma a meio e assentada):
  A  atual (o Over Oil Mix de fábrica, como sai da sonda)
  B  véu: no óleo, α-sobre com o α da coluna no papel logo acima do óleo (y 35..60) — contínuo, sem célula, sem K-M
  C  B + vales ao assentar: α × 2·(1 − h)·0,55/média, h = a luminância do óleo só normalizada (o relevo: claro = pico)
  D  C + a mesma lei no papel da coluna à volta do óleo (o #54: a faixa creme onde a tinta escorreu sem óleo)
uso: over_oil_simula.py <antes> <meio> <fim> <papel r,g,b> <saída.png>
"""
import sys

import numpy as np
from PIL import Image

Y = np.array([220.0, 200.0, 40.0])


def le(f):
    return np.asarray(Image.open(f).convert("RGB")).astype(float)


def alfa_coluna(a, papel):
    al = np.clip((a[35:60] - papel) / (Y - papel), 0, 1)[..., 2]
    return al.mean(0)  # por coluna, com as estrias do pincel


def simula(o, a, papel, assentada, vales, todo_papel):
    out = a.copy()
    mo = np.abs(o - papel).sum(2) > 24  # o óleo, até a orla fina (a que fazia o preto), alargado 3 px
    for _ in range(3):
        mo = mo | np.roll(mo, 1, 0) | np.roll(mo, -1, 0) | np.roll(mo, 1, 1) | np.roll(mo, -1, 1)
    al = np.broadcast_to(alfa_coluna(a, papel), o.shape[:2]).copy()
    al_papel = al.copy()  # no papel não há relevo: o α da coluna
    if assentada and vales:
        lo = o @ np.array([0.2126, 0.7152, 0.0722])
        h = (lo - np.percentile(lo[mo], 5)) / max(1.0, np.percentile(lo[mo], 95) - np.percentile(lo[mo], 5))
        k = np.clip(2.0 * (1.0 - h), 0.0, 2.0)
        al = al * k * 0.55 / max(1e-6, k[mo].mean())
    veu = o * (1 - al[..., None]) + Y * al[..., None]
    out[mo] = veu[mo]
    if todo_papel:
        # o #54: a aguada no papel nunca mais clara que o papel e a aguada (o creme): α-sobre o papel
        faixa = np.zeros(o.shape[:2], bool)
        ys = np.where(mo.any(1))[0]
        faixa[max(0, ys.min() - 4):ys.max() + 30] = True
        faixa &= (alfa_coluna(a, papel) > 0.03)[None, :]
        faixa &= ~mo
        # só os píxeis ERRADOS: o creme (mais azul que o papel e a aguada juntos) e os escuros (abaixo das duas)
        L = np.array([0.2126, 0.7152, 0.0722])
        esperado = o * (1 - al_papel[..., None]) + Y * al_papel[..., None]
        errado = (a[..., 2] > np.maximum(papel[2], Y[2]) + 15) | (a @ L < np.minimum(papel @ L, esperado @ L) - 20)
        faixa &= errado
        out[faixa] = (o * (1 - al_papel[..., None]) + Y * al_papel[..., None])[faixa]
    return np.clip(out, 0, 255).astype(np.uint8)


def main():
    antes, meio, fim, papel, saida = sys.argv[1:6]
    papel = np.array([float(c) for c in papel.split(",")])
    o, m, f = le(antes), le(meio), le(fim)
    linhas = []
    for nome, lei in [("A", None), ("B", (False, False)), ("C", (True, False)), ("D", (True, True))]:
        if lei is None:
            par = [m.astype(np.uint8), f.astype(np.uint8)]
        else:
            v, t = lei
            par = [simula(o, m, papel, False, v, t), simula(o, f, papel, True, v, t)]
        linhas.append(np.concatenate(par, axis=1))
        Image.fromarray(par[0]).save(saida.replace(".png", f"_{nome}_meio.png"))
        Image.fromarray(par[1]).save(saida.replace(".png", f"_{nome}_fim.png"))
    Image.fromarray(np.concatenate(linhas, axis=0)).save(saida)


if __name__ == "__main__":
    main()
