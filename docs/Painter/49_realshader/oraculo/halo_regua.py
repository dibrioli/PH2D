"""Régua do HALO (K4): o escurecimento do PAPEL por faixa de distância à tinta.

    python3 halo_regua.py <imagem> [cobertura.pgm]

Sem cobertura a tinta é o que tem COR (saturação > 0,12: a bateria pinta azul sobre papel branco). O papel
de referência é o que fica a > 40 px da tinta (p50). Imprime, por faixa (px), `1 − L/L_ref` (p50 e p90) da
luminância linear do papel — 0 = sem halo.
"""

import sys

import numpy as np
from PIL import Image

FAIXAS = [(1, 3), (3, 6), (6, 12), (12, 24), (24, 40)]


def lin(s):
    s = s / 255.0
    return np.where(s <= 0.04045, s / 12.92, ((s + 0.055) / 1.055) ** 2.4)


def distancia(tinta, ate=41):
    """Distância (px, xadrez 8-vizinhos) a `tinta`, até `ate`."""
    d = np.full(tinta.shape, ate, dtype=np.int32)
    d[tinta] = 0
    cur = tinta.copy()
    for k in range(1, ate):
        p = np.pad(cur, 1)
        viz = np.zeros_like(cur)
        for dy in (-1, 0, 1):
            for dx in (-1, 0, 1):
                viz |= p[1 + dy : 1 + dy + cur.shape[0], 1 + dx : 1 + dx + cur.shape[1]]
        novo = viz & ~cur
        d[novo] = k
        cur = viz
    return d


def lado_da_sombra(tinta, ate=24):
    """Papel com tinta PARA O LADO DA LUZ (topo-esquerda, azimute 225°) a menos de `ate` px: onde cai a sombra
    projectada. O resto do papel perto da tinta é o lado da luz (só a oclusão do céu)."""
    h, w = tinta.shape
    m = np.zeros_like(tinta)
    for k in range(1, ate):
        p = np.zeros_like(tinta)
        p[k:, k:] = tinta[:-k, :-k]
        m |= p
        p = np.zeros_like(tinta)
        p[k:, :] = tinta[:-k, :]
        m |= p
        p = np.zeros_like(tinta)
        p[:, k:] = tinta[:, :-k]
        m |= p
    return m


def luminancia(img):
    if img.endswith(".exr"):
        import OpenImageIO as oiio

        px = oiio.ImageBuf(img).get_pixels(oiio.FLOAT)[..., :3].astype(np.float64)
        return px @ np.array([0.2126, 0.7152, 0.0722]), None
    rgb = np.asarray(Image.open(img).convert("RGB")).astype(np.float64)
    return lin(rgb) @ np.array([0.2126, 0.7152, 0.0722]), rgb


def mede(img, cob=None):
    lum, rgb = luminancia(img)
    if cob is not None:
        tinta = np.asarray(Image.open(cob).convert("L")) > 25
    else:
        mx, mn = rgb.max(-1), rgb.min(-1)
        tinta = (mx - mn) / np.maximum(mx, 1) > 0.12
    d = distancia(tinta)
    ref = np.percentile(lum[d >= 41], 50)
    sombra = lado_da_sombra(tinta)
    linhas = [f"ref {ref:.3f}"]
    for rot, lado in [("tudo", np.ones_like(tinta)), ("luz", ~sombra), ("sombra", sombra)]:
        linha = []
        for a, b in FAIXAS:
            m = (d >= a) & (d < b) & lado
            if m.sum() < 20:
                continue
            x = 1 - lum[m] / ref
            linha.append(f"{a}-{b}: {np.percentile(x, 50):+.3f}/{np.percentile(x, 90):+.3f}")
        linhas.append(f"  {rot:6s} " + " · ".join(linha))
    return "\n".join(linhas)


if __name__ == "__main__":
    print(mede(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else None))
