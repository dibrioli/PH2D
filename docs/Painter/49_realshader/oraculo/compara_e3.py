"""Régua E3-B (W2, 27/10): o material do Óleo Real × o Principled do Cycles, MATERIAL A MATERIAL, sobre a
MESMA tinta (a cena do dono, `medida_oleo_real_oraculo` com `PH2D_OLEO_VARIANTES` de `materiais_e3.py`).

    python3 docs/Painter/49_realshader/oraculo/compara_e3.py <sonda> <saida_cycles> [mat,…]

`<sonda>/<mat>/` = a nossa variante, `<saida_cycles>/mat_<mat>_{cor,brilho}.exr` = a do Cycles. Máscara = a
tinta (cobertura > 0,9). Números em LINEAR, presos a `[0, 1]` dos dois lados (o nosso realce enche a folga;
o Cycles passa de 1):
  cor   — luminância da imagem: p50 p99 p99,9 · área > 0,3
  Δ     — o que o material ACRESCENTA à luminância do neutro da mesma luz: média · p99 · correlação nós×Cycles
  brilho — só o especular (base preta; não vale para metal nem tint): p50 p99 p99,9 · área > 0,3
"""

import os
import sys

import numpy as np
import OpenImageIO as oiio

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import materiais_e3  # noqa: E402

son, sai = sys.argv[1], sys.argv[2]
SO = sys.argv[3].split(",") if len(sys.argv) > 3 else list(materiais_e3.MATERIAIS)


def pnm(caminho):
    b = open(caminho, "rb").read()
    partes = b.split(maxsplit=4)
    w, h = int(partes[1]), int(partes[2])
    return np.frombuffer(partes[4], dtype=np.uint8).reshape(h, w, -1).squeeze().astype(np.float64)


def lin(s):
    s = s / 255.0
    return np.where(s <= 0.04045, s / 12.92, ((s + 0.055) / 1.055) ** 2.4)


def exr(nome):
    return oiio.ImageBuf(f"{sai}/{nome}.exr").get_pixels(oiio.FLOAT)[..., :3].astype(np.float64)


LUM = np.array([0.2126, 0.7152, 0.0722])
tinta = pnm(f"{son}/neutro/cobertura.pgm") / 255.0 > 0.9


def nos_lum(m):
    return np.clip(lin(pnm(f"{son}/{m}/oleo.ppm")), 0, 1) @ LUM


def cy_lum(m):
    return np.clip(exr(f"mat_{m}_cor"), 0, 1) @ LUM


def q(v, ps=(50, 99, 99.9)):
    return " ".join(f"{np.percentile(v[tinta], p):6.3f}" for p in ps)


def area(v):
    return 100 * (v[tinta] > 0.3).mean()


print(f"células de tinta {tinta.sum()}")
print(f"{'material':<14} {'':<7} {'cor p50 p99 p99.9':<22} área>0,3 | {'Δ média p99 · corr':<24} | brilho p50 p99 p99.9 · área")
for m in SO:
    base = "neutro_hdri" if materiais_e3.luz(m) == "hdri_studio" else "neutro"
    n, c = nos_lum(m), cy_lum(m)
    dn, dc = n - nos_lum(base), c - cy_lum(base)
    corr = np.corrcoef(dn[tinta], dc[tinta])[0, 1] if dn[tinta].std() > 1e-6 and dc[tinta].std() > 1e-6 else float("nan")
    espec = (pnm(f"{son}/{m}/t_espec.pgm") - 1) / 254.0 * 2
    bc = np.clip(exr(f"mat_{m}_brilho"), 0, None).mean(-1)
    brilho = m.startswith(("metal", "tint"))
    for rot, v, d, b in (("nós", n, dn, espec), ("Cycles", c, dc, bc)):
        txt_b = "—" if brilho else f"{q(b)} · {area(b):5.2f} %"
        print(
            f"{m:<14} {rot:<7} {q(v)} {area(v):6.2f} % | {d[tinta].mean():+.4f} {np.percentile(d[tinta], 99):+.3f}"
            f"{'' if rot == 'Cycles' else f' · {corr:+.2f}':<8} | {txt_b}"
        )

# A translucidez POR PASSAGEM (a fracção que chega ao papel): onde ela é 0 o Cycles muda +0,01 com a Thin Wall e
# o plano do papel por baixo (um desvio do arranjo do oráculo, não do lóbulo) — o lóbulo lê-se nas faixas > 0.
gh, gw = tinta.shape
pas = np.fromfile(f"{son}/neutro/t_passagem.f32", dtype="<f4").reshape(gh, gw)
for m in [x for x in SO if x.startswith("transl")]:
    base = "neutro_hdri" if materiais_e3.luz(m) == "hdri_studio" else "neutro"
    dn, dc = nos_lum(m) - nos_lum(base), cy_lum(m) - cy_lum(base)
    for lo, hi in ((0, 0.001), (0.001, 0.05), (0.05, 0.3), (0.3, 1.01)):
        k = tinta & (pas >= lo) & (pas < hi)
        print(f"{m:<14} passagem [{lo},{hi}) n={k.sum():6d}: Δ nós {dn[k].mean():+.4f} · Cycles {dc[k].mean():+.4f}")
