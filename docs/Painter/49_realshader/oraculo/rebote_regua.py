"""Régua K10 — a luz INDIRECTA no papel, os três casos de uma família, nosso × o Cycles da MESMA malha.

    python3 rebote_regua.py <saida_k10> <rebelle dabber_3_tamanhos_A.png | -> <entrada_k10/<variante>> …

`<entrada>/<cena>/{estudio,classico}/` de cada variante da sonda (`medida_oleo_real_oraculo`, `PH2D_OLEO_CENA=<cena>`), e
`<saida>/<cena>/{studio_sol10,sol10}_cor.exr` do `cycles.py`. Cenas e máscaras:

- `dabber3` (a): os VÃOS — papel (cobertura ≤ 25) dentro do casco da tinta (fecho morfológico de raio 6) no recto
  de 200 px (y 210..370). Luminância relativa (÷ papel longe) p10/p50 em byte sRGB e a fracção com
  `max(R,G,B) < 70` (a régua da E1, `compara.py` §1).
- `travessia` (b): o papel por baixo do planalto kL 1 (linhas 329..345, x 150..500 — a de `papel_regua_k8.py`).
- `lisos` (c) e `dono`: o escurecimento linear `1 − L/L_ref` do papel por faixa e lado (`halo_regua.py`).
"""
import sys

import numpy as np
from PIL import Image

import halo_regua as H

W = np.array([0.2126, 0.7152, 0.0722])


def srgb(x):
    x = np.clip(x, 0, 1)
    return 255 * np.where(x <= 0.0031308, 12.92 * x, 1.055 * x ** (1 / 2.4) - 0.055)


def nosso(d):
    rgb = np.asarray(Image.open(f"{d}/oleo.ppm").convert("RGB")).astype(np.float64)
    return H.lin(rgb), rgb


def cycles(f):
    import OpenImageIO as oiio

    lin = oiio.ImageBuf(f).get_pixels(oiio.FLOAT)[..., :3].astype(np.float64)
    return lin, srgb(lin)


def fecho(m, r):
    def dil(x):
        p = np.pad(x, r)
        o = np.zeros_like(x)
        for dy in range(-r, r + 1):
            for dx in range(-r, r + 1):
                if dx * dx + dy * dy <= r * r:
                    o |= p[r + dy : r + dy + x.shape[0], r + dx : r + dx + x.shape[1]]
        return o

    return ~dil(~dil(m))


def vaos(ents, sai, rebelle=None):
    cob = np.asarray(Image.open(f"{ents[0][1]}/dabber3/estudio/cobertura.pgm")).astype(np.int32)
    tinta = cob > 25
    casco = fecho(tinta, 6)
    banda = np.zeros_like(tinta)
    banda[210:370, :] = True
    m = casco & ~tinta & banda
    longe = H.distancia(tinta) >= 41
    print(f"## (a) dabber3 — vãos: {int(m.sum())} células de papel dentro do casco do recto de 200 (byte da fracção p10 · p50 · max<70)")
    for amb, cy in [("estudio", "studio_sol10"), ("classico", "sol10")]:
        fontes = [(f"{r} {amb}", nosso(f"{e}/dabber3/{amb}")) for r, e in ents]
        fontes.append((f"Cycles {amb}", cycles(f"{sai}/dabber3/{cy}_cor.exr")))
        for rot, (lin, rgb) in fontes:
            lum = lin @ W
            rel = lum / np.percentile(lum[longe], 50)
            b = srgb(rel[m])
            pretos = (rgb.max(-1)[m] < 70).mean() * 100
            print(f"  {rot:18s} {np.percentile(b, 10):4.0f} · {np.percentile(b, 50):4.0f} · {pretos:5.2f} %")
    if rebelle:
        rgb = np.asarray(Image.open(rebelle).convert("RGB")).astype(np.float64)
        mx, mn = rgb.max(-1), rgb.min(-1)
        t = (mx - mn) / np.maximum(mx, 1) > 0.12
        c = fecho(t, 6) & ~t
        c[: rgb.shape[0] // 2] = False
        lum = H.lin(rgb) @ W
        rel = lum / np.percentile(lum[H.distancia(t) >= 41], 50)
        b = srgb(rel[c])
        print(f"  {'Rebelle (foto)':18s} {np.percentile(b, 10):4.0f} · {np.percentile(b, 50):4.0f} · {(mx[c] < 70).mean() * 100:5.2f} %")


def planalto(ents, sai):
    rows = list(range(329, 346))
    print(f"## (b) travessia — papel sob o planalto kL 1, linhas {rows[0]}..{rows[-1]} (byte sRGB da fracção)")
    for amb, cy in [("estudio", "studio_sol10"), ("classico", "sol10")]:
        fontes = [(f"{r} {amb}", nosso(f"{e}/travessia/{amb}")) for r, e in ents]
        fontes.append((f"Cycles {amb}", cycles(f"{sai}/travessia/{cy}_cor.exr")))
        for rot, (lin, _) in fontes:
            lum = lin @ W
            ref = np.median(lum[440:455, 150:500])
            print(f"  {rot:18s} " + " ".join(f"{srgb(min(1, np.median(lum[y, 150:500]) / ref)):3.0f}" for y in rows))


def halo(cena, ents, sai):
    cob = f"{ents[0][1]}/{cena}/estudio/cobertura.pgm"
    print(f"## {cena} — escurecimento linear do papel, p50 por faixa (px)")
    for amb, cy in [("estudio", "studio_sol10"), ("classico", "sol10")]:
        fontes = [(f"{r} {amb}", f"{e}/{cena}/{amb}/oleo.ppm") for r, e in ents]
        fontes.append((f"Cycles {amb}", f"{sai}/{cena}/{cy}_cor.exr"))
        for rot, img in fontes:
            ls = H.mede(img, cob).splitlines()[2:]
            curto = " | ".join(" ".join(x.split("/")[0] for x in l.split()[1:]) for l in ls)
            print(f"  {rot:18s} {curto}")


if __name__ == "__main__":
    sai, reb = sys.argv[1], sys.argv[2]
    ents = [(e.rstrip("/").split("/")[-1], e) for e in sys.argv[3:]]
    vaos(ents, sai, None if reb == "-" else reb)
    planalto(ents, sai)
    halo("lisos", ents, sai)
    halo("dono", ents, sai)
