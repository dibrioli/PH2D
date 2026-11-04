#!/usr/bin/env python3
"""Régua do «óleo sobre óleo» sobre os PNG do documento (c1/azul/amarelo) gravados pelo driver PH2D_OLEO_APP.
uso: python3 -I mede.py <dir_m>   (cada subpasta = uma corrida)
FUROS   : px cor de papel (dist RGB ao papel < 25) com >= 85 % de tinta na vizinhança 11x11 (miolo; a franja da borda fica de fora); componentes 8-conexas.
CAROÇOS : máximos locais 5x5 da luminância dentro da tinta erodida (7x7) com proeminência Y - média9x9 > 10;
          por 10 mil px de tinta. HP = desvio-padrão de Y - média9x9 na tinta erodida (rugosidade).
PÁLIDOS : px dentro da mancha (>=95 % tinta 11x11, o miolo — a orla de sombra sobre o papel fica de fora), não-papel, saturação < 0.3 e max canal > 190; componentes.
BRANCOS : px de tinta (não papel) mais claros que QUALQUER mistura azul–amarelo: min_t max_k(c - mix_t) > 20
          (a régua de borda_branca_tests.rs); no azul.png o segmento é só o azul (t=0).
"""
import sys, os
import numpy as np
from numpy.lib.stride_tricks import sliding_window_view as swv
from PIL import Image


def box(a, r):
    p = np.pad(a, r, mode="edge").astype(np.float64)
    c = p.cumsum(0).cumsum(1)
    c = np.pad(c, ((1, 0), (1, 0)))
    k = 2 * r + 1
    s = c[k:, k:] - c[:-k, k:] - c[k:, :-k] + c[:-k, :-k]
    return s / (k * k)


def comps(mask):
    h, w = mask.shape
    seen = np.zeros_like(mask, bool)
    n = 0
    ys, xs = np.nonzero(mask)
    for y0, x0 in zip(ys, xs):
        if seen[y0, x0]:
            continue
        n += 1
        st = [(y0, x0)]
        seen[y0, x0] = True
        while st:
            y, x = st.pop()
            for dy in (-1, 0, 1):
                for dx in (-1, 0, 1):
                    yy, xx = y + dy, x + dx
                    if 0 <= yy < h and 0 <= xx < w and mask[yy, xx] and not seen[yy, xx]:
                        seen[yy, xx] = True
                        st.append((yy, xx))
    return n


def load(p):
    return np.asarray(Image.open(p).convert("RGB")).astype(np.float64)


def mede(img, az, am):
    h, w, _ = img.shape
    borda = np.concatenate([img[:20].reshape(-1, 3), img[-20:].reshape(-1, 3), img[:, :20].reshape(-1, 3), img[:, -20:].reshape(-1, 3)])
    P = np.median(borda, 0)
    d = np.sqrt(((img - P) ** 2).sum(-1))
    tinta = d > 40
    papel = d < 25
    frac = box(tinta.astype(float), 3)
    furo = papel & (box(tinta.astype(float), 5) >= 0.85)
    nfuro = comps(furo)
    Y = 0.2126 * img[..., 0] + 0.7152 * img[..., 1] + 0.0722 * img[..., 2]
    ero = box(tinta.astype(float), 3) > 0.999
    hp = Y - box(Y, 4)
    mx = np.pad(swv(np.pad(Y, 2, mode="edge"), (5, 5)).max(axis=(-1, -2)), 0)
    pico = ero & (Y >= mx) & (hp > 10)
    mn = swv(np.pad(Y, 2, mode="edge"), (5, 5)).min(axis=(-1, -2))
    poco = ero & (Y <= mn) & (hp < -10)
    area = max(tinta.sum(), 1)
    # brancos
    c = img[tinta & ~papel]
    if am is None:
        e = (c - az).max(-1)
    else:
        e = np.full(len(c), np.inf)
        for t in np.linspace(0, 1, 51):
            e = np.minimum(e, (c - (t * am + (1 - t) * az)).max(-1))
    brancos = int((e > 20).sum())
    mxc = img.max(-1); mnc = img.min(-1)
    sat = (mxc - mnc) / np.maximum(mxc, 1)
    palido = (~papel) & (box(tinta.astype(float), 5) >= 0.95) & (sat < 0.3) & (mxc > 190)
    npal = comps(palido)
    return dict(area=int(area), furos_px=int(furo.sum()), furos=nfuro, picos=round(pico.sum() / area * 1e4, 1),
                pocos=round(poco.sum() / area * 1e4, 1), hp=round(float(hp[ero].std()) if ero.any() else 0, 1), brancos=brancos,
                brancos_pct=round(100 * brancos / area, 2), palido_px=int(palido.sum()), palidos=npal)


def main():
    raiz = sys.argv[1]
    print(f"{'corrida':18} {'img':8} {'area':>7} {'furos':>5} {'furo_px':>7} {'picos/1e4':>9} {'pocos/1e4':>9} {'HP':>5} {'brancos':>7} {'%':>5} {'palidos':>7} {'pal_px':>8}")
    for run in sorted(os.listdir(raiz)):
        d = os.path.join(raiz, run)
        if not os.path.exists(os.path.join(d, "azul.png")):
            print(run, "sem azul.png")
            continue
        azi = load(os.path.join(d, "azul.png"))
        m = (azi[..., 2] > azi[..., 0] + 60)
        az = np.median(azi[m], 0)
        ami = load(os.path.join(d, "amarelo.png")) if os.path.exists(os.path.join(d, "amarelo.png")) else None
        am = None
        if ami is not None:
            m2 = (ami[..., 0] > 180) & (ami[..., 1] > 160) & (ami[..., 2] < 110)
            am = np.median(ami[m2], 0)
        for nome, img, seg in (("c1", load(os.path.join(d, "c1.png")), None), ("azul", azi, None), ("amarelo", ami, am)):
            if img is None:
                continue
            r = mede(img, az, seg)
            print(f"{run:18} {nome:8} {r['area']:7} {r['furos']:5} {r['furos_px']:7} {r['picos']:9} {r['pocos']:9} {r['hp']:5} {r['brancos']:7} {r['brancos_pct']:5} {r['palidos']:7} {r['palido_px']:8}")
        print(f"{'':18} ref azul={az.astype(int).tolist()} amarelo={None if am is None else am.astype(int).tolist()}")


main()
