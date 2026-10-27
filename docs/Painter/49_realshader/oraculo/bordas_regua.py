#!/usr/bin/env python3
"""§BUG-bordas — a régua do SERRILHADO de uma borda de tinta azul sobre papel claro.

Por coluna, a posição sub-pixel da borda (o cruzamento de «azul» = clamp((B − R)/120) com ½, interpolado na
coluna), nas colunas onde a borda é quase horizontal (|declive| ≤ 0,7). O resíduo contra um ajuste LOCAL de 2.º
grau (Savitzky-Golay, janela 15): `pos` = RMS do resíduo (px) — mistura a escada com a forma rasgada do pincel.
⭐ `TRAVA` = |média de exp(2πi·posição)|: a posição da borda PRESA à grade de pixels (1 = escada perfeita: toda
coluna cruza no mesmo sítio do pixel; ~0 = contorno contínuo a sub-pixel). Cega à forma do traço, que não
conhece a grade. ⭐ `ESCADA` (px) e `LUZ fora/dentro` (níveis): a amplitude da parte do resíduo (posição; luminância
a ±1,5 px) que acompanha a FASE sub-pixel da linha ajustada — o que se repete a cada degrau da grade (a silhueta em
escada, a linha escura e os brilhos em degrau); a textura do pincel não conhece a grade e cancela-se. A luminância amostrada AO LONGO da
borda a ±1,5 px dela (bilinear), menos o mesmo ajuste local: `lum_fora` (o papel / a sombra ao pé) e
`lum_dentro` (a parede, os brilhos em degrau) = RMS (níveis 0..255). `zoom4` = o mesmo sobre a imagem ampliada 4×
bilinear (o que o dono vê a 400 %), posições em px da imagem original.

    python3 -I bordas_regua.py <img> <x0> <x1> <y0> <y1> [...mais caixas]
"""
import sys

import numpy as np
from PIL import Image

JANELA = 15


def ajuste_local(v):
    n, h = len(v), JANELA // 2
    out = np.full(n, np.nan)
    xs = np.arange(-h, h + 1)
    for i in range(h, n - h):
        janela = v[i - h:i + h + 1]
        if np.isnan(janela).any():
            continue
        out[i] = np.polyval(np.polyfit(xs, janela, 2), 0)
    return out


OFFS = (-3.0, -2.5, -2.0, -1.5, -1.0, -0.5, 0.0, 0.5, 1.0, 1.5, 2.0)


def sinal_de(lado):
    return -1 if lado == "cima" else 1


def faixa_por_fase(v, fase, n=8):
    """A amplitude (máx − mín) da média de `v` por fatia da FASE sub-pixel da linha: o que se repete a cada degrau."""
    v, fase = np.asarray(v), np.asarray(fase)
    if len(v) < 8 * n:
        return float("nan")
    b = np.minimum((fase * n).astype(int), n - 1)
    medias = [v[b == i].mean() for i in range(n) if (b == i).sum() >= 4]
    return float(max(medias) - min(medias))


def bil(img, x, y):
    h, w = img.shape
    x0, y0 = int(np.floor(x)), int(np.floor(y))
    tx, ty = x - x0, y - y0
    g = lambda a, b: img[min(max(b, 0), h - 1), min(max(a, 0), w - 1)]
    return (g(x0, y0) * (1 - tx) + g(x0 + 1, y0) * tx) * (1 - ty) + (g(x0, y0 + 1) * (1 - tx) + g(x0 + 1, y0 + 1) * tx) * ty


def bordas(rgb, caixa, k=1):
    x0, x1, y0, y1 = [v * k for v in caixa]
    rgb = rgb.astype(float)
    azul = np.clip((rgb[..., 2] - rgb[..., 0]) / 120.0, 0, 1)
    lum = 0.2126 * rgb[..., 0] + 0.7152 * rgb[..., 1] + 0.0722 * rgb[..., 2]
    res = {"pos": [], "lum_fora": [], "lum_dentro": [], "fase": []}
    presa = {"escada": [], "luz_fora": [], "luz_dentro": []}
    degrau = {d: ([], []) for d in OFFS}
    esc = ([], [])
    for lado in ("cima", "baixo"):
        e = np.full(x1 - x0, np.nan)
        for i, x in enumerate(range(x0, x1)):
            col = azul[y0:y1, x]
            ys = range(len(col) - 1) if lado == "cima" else range(len(col) - 2, -1, -1)
            for y in ys:
                a, b = (col[y], col[y + 1]) if lado == "cima" else (col[y + 1], col[y])
                if a < 0.5 <= b:
                    t = (0.5 - a) / (b - a)
                    e[i] = y0 + (y + t if lado == "cima" else y + 1 - t)
                    break
        dec = np.gradient(ajuste_local(e))
        f = ajuste_local(e)
        ok = (~np.isnan(f)) & (np.abs(dec) <= 0.7)
        r = (e - f)[ok]
        res["pos"].extend(r / k)
        res["fase"].extend(np.exp(2j * np.pi * e[ok] / k))
        fase = np.exp(2j * np.pi * f / k)
        presa["escada"].extend((r / k) * fase[ok])
        esc[0].extend(r / k)
        esc[1].extend((f[ok] / k) % 1.0)
        for d in OFFS:
            l = np.array([bil(lum, x0 + i, f[i] + sinal_de(lado) * d * k) if ok[i] else np.nan
                          for i in range(len(e))])
            lf = ajuste_local(l)
            m = ok & ~np.isnan(lf)
            degrau[d][0].extend((l - lf)[m])
            degrau[d][1].extend((f[m] / k) % 1.0)
        sinal = -1 if lado == "cima" else 1  # para FORA (o papel)
        for nome, off in (("lum_fora", 1.5), ("lum_dentro", -1.5)):
            l = np.array([bil(lum, x0 + i, e[i] + sinal * off * k) if not np.isnan(e[i]) else np.nan
                          for i in range(len(e))])
            lf = ajuste_local(l)
            m = ok & ~np.isnan(lf)
            res[nome].extend((l - lf)[m])
            presa["luz_" + nome[4:]].extend((l - lf)[m] * fase[m])
    trava = float(abs(np.mean(res.pop("fase"))))
    presa = {n: 2 * float(abs(np.mean(v))) for n, v in presa.items()}
    presa["DEGRAU"] = max(faixa_por_fase(*degrau[d]) for d in OFFS)
    presa["ESC"] = faixa_por_fase(*esc)
    return {n: float(np.sqrt(np.mean(np.square(v)))) if v else float("nan") for n, v in res.items()} | {
        "colunas": len(res["pos"]), "trava": trava} | presa


def mede(caminho, caixas):
    im = Image.open(caminho).convert("RGB")
    a = np.asarray(im)
    z = np.asarray(im.resize((im.width * 4, im.height * 4), Image.BILINEAR))
    out = []
    for c in caixas:
        out.append((c, bordas(a, c), bordas(z, c, 4)))
    return out


if __name__ == "__main__":
    caminho, nums = sys.argv[1], list(map(int, sys.argv[2:]))
    caixas = [tuple(nums[i:i + 4]) for i in range(0, len(nums), 4)]
    for c, r1, r4 in mede(caminho, caixas):
        print(f"{caminho.split('/')[-1]:<30} {str(c):<20} DEGRAU {r1['DEGRAU']:.2f} ESC {r1['ESC']:.3f} · 400% {r4['DEGRAU']:.2f} {r4['ESC']:.3f} | ESCADA {r1['escada']:.3f} LUZ {r1['luz_fora']:.2f}/{r1['luz_dentro']:.2f} "
              f"· 400% {r4['escada']:.3f} {r4['luz_fora']:.2f}/{r4['luz_dentro']:.2f} | TRAVA {r1['trava']:.3f} · 400% {r4['trava']:.3f} | 100%: pos {r1['pos']:.3f} fora {r1['lum_fora']:.2f} "
              f"dentro {r1['lum_dentro']:.2f} ({r1['colunas']} col) | 400%: pos {r4['pos']:.3f} "
              f"fora {r4['lum_fora']:.2f} dentro {r4['lum_dentro']:.2f}")
