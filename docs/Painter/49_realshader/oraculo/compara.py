"""Régua POR TERMO: o nosso Óleo Real × o oráculo Cycles sobre a MESMA tinta (W1c).

    python3 docs/Painter/49_realshader/oraculo/compara.py <entrada> <saida>

Lê os `t_*.pgm` e `oleo.ppm` da sonda `medida_oleo_real_oraculo` e os EXR de `cycles.py`; imprime a tabela.
"""

import sys

import numpy as np
import OpenImageIO as oiio

ent, sai = sys.argv[1], sys.argv[2]
# A variante do Cycles com a MESMA receita de luz do modo medido (sol10 = Clássico, studio_sol10 = Estúdio).
LUZ = sys.argv[3] if len(sys.argv) > 3 else "sol10"


def pnm(nome):
    b = open(f"{ent}/{nome}", "rb").read()
    partes = b.split(maxsplit=4)
    w, h = int(partes[1]), int(partes[2])
    dados = np.frombuffer(partes[4], dtype=np.uint8)
    return dados.reshape(h, w, -1).squeeze().astype(np.float64)


def exr(nome):
    return oiio.ImageBuf(f"{sai}/{nome}.exr").get_pixels(oiio.FLOAT)[..., :3].astype(np.float64)


def lin(s):
    s = s / 255.0
    return np.where(s <= 0.04045, s / 12.92, ((s + 0.055) / 1.055) ** 2.4)


def srgb(x):
    x = np.clip(x, 0, 1)
    return 255 * np.where(x <= 0.0031308, 12.92 * x, 1.055 * x ** (1 / 2.4) - 0.055)


cob = pnm("cobertura.pgm") / 255.0
tinta = cob > 0.9
termo = lambda n, k: (pnm(f"t_{n}.pgm") - 1) / 254.0 * k  # noqa: E731
luz, e, espec = termo("luz", 1), termo("e", 2), termo("espec", 2)
nosso = pnm("oleo.ppm")
alb = lin(pnm("albedo.ppm"))
vermelho = tinta & (alb[..., 0] > 0.4) & (alb[..., 1] < 0.05)
# A fenda é a do ORÁCULO (o céu que o Cycles vê), nunca um termo nosso: a máscara não pode ser o produto.
vc = exr("ceu_cinza_direto").mean(-1) / 0.3
cava = tinta & (vc < 0.55)
fenda = vermelho & cava
print(f"células de tinta {tinta.sum()} · vermelho {vermelho.sum()} · fenda vermelha (V Cycles<0,55) {fenda.sum()}")


def q(v, m, ps=(10, 50, 90)):
    return " ".join(f"{np.percentile(v[m], p):6.3f}" for p in ps)


print("\n## 1. Pontos pretos — a fenda VERMELHA (p10 p50 p90 da luminância sRGB 0..255)")
lum = lambda im: im @ np.array([0.2126, 0.7152, 0.0722])  # noqa: E731
for rot, im in [
    ("PH2D", nosso),
    ("Cycles céu+sol10", srgb(exr("sol10_cor"))),
    ("Cycles HDRI estúdio", srgb(exr("hdri_studio_cor"))),
    ("Cycles estúdio+sol10", srgb(exr("studio_sol10_cor"))),
]:
    preto = vermelho & (im.max(axis=-1) < 70)
    print(
        f"  {rot:<22} fenda: {q(lum(im), fenda)} · R na fenda p50 {np.percentile(im[..., 0][fenda], 50):5.0f}"
        f" · pixels «pretos» (max<70) no vermelho {100 * preto.sum() / vermelho.sum():5.2f} %"
    )

print("\n## 2. Rebote — o que os ricochetes acrescentam ao difuso (cinza 0,5): (com − directo)/com")
for l in ("sol10", "hdri_studio"):
    com, dir_ = exr(f"{l}_cinza").mean(-1), exr(f"{l}_cinza_direto").mean(-1)
    reb = (com - dir_) / np.maximum(com, 1e-6)
    print(f"  {l:<12} tinta: {q(reb, tinta)} · fenda: {q(reb, cava)}")

print(f"\n## 3. Difuso relativo E (plano iluminado = 1): PH2D t_e × Cycles {LUZ} cinza÷0,5")
ce = exr(f"{LUZ}_cinza").mean(-1) / 0.5
ced = exr(f"{LUZ}_cinza_direto").mean(-1) / 0.5
for rot, m in [("tinta", tinta), ("fenda V<0,55", cava), ("sombra luz<0,5", tinta & (luz < 0.5))]:
    print(f"  {rot:<16} PH2D {q(e, m)} · Cycles {q(ce, m)} · Cycles só directo {q(ced, m)}")
c = np.corrcoef(e[tinta], ce[tinta])[0, 1]
print(f"  correlação PH2D×Cycles na tinta: {c:.3f}")

# W2 E2: o PAPEL ao pé das paredes (cobertura < 0,02 a até 8 células da tinta): a luz que recebe, 1 − sombra
# (o `oleo.ppm` grava 255·(1 − sombra) no papel branco) × o Cycles cinza÷0,5 sobre a mesma malha.
perto = np.zeros_like(tinta)
for dy in range(-8, 9):
    for dx in range(-8, 9):
        if dx * dx + dy * dy <= 64:
            perto |= np.roll(np.roll(tinta, dy, 0), dx, 1)
papel = (cob < 0.02) & perto
nosso_papel = nosso.mean(-1) / 255.0
print(f"\n## 3b. Papel ao pé das paredes ({papel.sum()} células): luz PH2D 1−sombra × Cycles {LUZ} cinza÷0,5")
print(f"  PH2D {q(nosso_papel, papel, (5, 10, 50))} · Cycles {q(ce, papel, (5, 10, 50))}  (p5 p10 p50)")
cp = np.corrcoef(nosso_papel[papel], ce[papel])[0, 1]
print(f"  correlação no papel: {cp:.3f}")

print("\n## 4. Brilho (linear, somado): PH2D t_espec × Cycles base preta (p50 p99 p99.9 · área > 0,3)")
for rot, b in [
    ("PH2D", espec),
    ("Cycles céu+sol10", exr("sol10_brilho").mean(-1)),
    ("Cycles HDRI estúdio", exr("hdri_studio_brilho").mean(-1)),
    ("Cycles estúdio+sol10", exr("studio_sol10_brilho").mean(-1)),
]:
    print(f"  {rot:<22} {q(b, tinta, (50, 99, 99.9))} · área>0,3 {100 * (b[tinta] > 0.3).mean():5.2f} %")

print("\n## 5. Realce que passa de 1 (fracção de pixels de tinta com algum canal > 1, linear)")
for l in ("sol10", "hdri_studio", "studio_sol10", "hdri_interior", "hdri_courtyard"):
    im = exr(f"{l}_cor")
    print(f"  {l:<16} {100 * (im.max(-1)[tinta] > 1).mean():5.2f} %  · p99.9 {np.percentile(im.max(-1)[tinta], 99.9):5.2f}")
