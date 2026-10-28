"""Régua dos DOIS MODOS da Transmission (dono 28/10: «vou preferir o mais escuro do Blender, com segunda opção, sem
tirar a atual»): Blender (`tr*`) e Clear glass (`vc*`) × o Cycles com o Principled Transmission Weight na MESMA malha
(bloco ALFA do `cycles.py`, `tr05`/`tr1`), papel creme. A mesma régua do `transmissao_regua.py` (miolo cobertura ≥ 0,99
erodida 6; sombra projectada no papel nu ÷ a opaca), e a varredura da calibração (`nos_m<m>_<05|1>.ppm`, o modo
intermédio `m` = `k = K_BLENDER + (VIDRO_K − K_BLENDER)·m`).

    PH2D_OLEO_CREME=1 PH2D_OLEO_MODOS=… PH2D_OLEO_PNG=<sonda> cargo test -p ph2d-tool-painter --profile smoke --lib \\
        medida_oleo_real_transparencia -- --ignored
    python3 transmissao_modos_regua.py <sonda> <saida do cycles> <folha.jpg> [K_BLENDER VIDRO_K]
"""

import glob
import re
import sys

import numpy as np
import OpenImageIO as oiio
from PIL import Image, ImageDraw


def pnm(c):
    b = open(c, "rb").read()
    p = b.split(maxsplit=4)
    w, h = int(p[1]), int(p[2])
    return np.frombuffer(p[4], dtype=np.uint8).reshape(h, w, -1).squeeze().astype(np.float64)


def lin(s):
    s = s / 255.0
    return np.where(s <= 0.04045, s / 12.92, ((s + 0.055) / 1.055) ** 2.4)


def srgb(v):
    v = np.clip(v, 0.0, 1.0)
    return 255.0 * np.where(v <= 0.0031308, 12.92 * v, 1.055 * v ** (1 / 2.4) - 0.055)


def lum(rgb_lin):
    return rgb_lin @ np.array([0.2126, 0.7152, 0.0722])


def erode(m, r):
    o = m.copy()
    for dy in range(-r, r + 1):
        for dx in range(-r, r + 1):
            o &= np.roll(np.roll(m, dy, 0), dx, 1)
    return o


def dilata(m, r):
    o = m.copy()
    for dy in range(-r, r + 1, 2):
        for dx in range(-r, r + 1, 2):
            o |= np.roll(np.roll(m, dy, 0), dx, 1)
    return o


son, sai, folha = sys.argv[1], sys.argv[2], sys.argv[3]
kb, kv = (float(sys.argv[4]), float(sys.argv[5])) if len(sys.argv) > 5 else (None, None)
cob = pnm(f"{son}/entrada/cobertura.pgm") / 255.0
papel = lin(pnm(f"{son}/papel.ppm"))
lp = lum(papel)
miolo = erode(cob >= 0.99, 6)
tinta = cob > 0.0
perto = dilata(tinta, 30) & ~dilata(tinta, 1)
nos = {k: lin(pnm(f"{son}/nos_{k}.ppm")) for k in ("t0", "tr05", "tr1", "vc05", "vc1")}
cy = {
    k: oiio.ImageBuf(f"{sai}/alfa_{c}_cor.exr").get_pixels(oiio.FLOAT)[..., :3].astype(np.float64)
    for k, c in (("t0", "transp0"), ("tr05", "tr05"), ("tr1", "tr1"))
}
sombra = perto & (lum(cy["t0"]) < 0.95 * lp)
print(f"miolo {miolo.sum()} células · sombra projectada (Cycles T 0) {sombra.sum()} células")


def numeros(im):
    s = srgb(im)
    mx, mn = s.max(axis=-1), s.min(axis=-1)
    sat = float(np.mean(((mx - mn) / np.maximum(mx, 1.0))[miolo]))
    cro = float(np.mean((mx - mn)[miolo]))
    l = lum(im)
    return float(np.mean(l[miolo] / lp[miolo])), sat, cro, float(np.mean(1.0 - l[sombra] / lp[sombra]))


base = {"nós": numeros(nos["t0"])[3], "Cycles": numeros(cy["t0"])[3]}
for k, nome, d, quem in (
    ("t0", "opaca", nos, "nós"),
    ("t0", "opaca", cy, "Cycles"),
    ("tr05", "Blender 0,5", nos, "nós"),
    ("tr05", "Transmission 0,5", cy, "Cycles"),
    ("vc05", "Clear glass 0,5", nos, "nós"),
    ("tr1", "Blender 1", nos, "nós"),
    ("tr1", "Transmission 1", cy, "Cycles"),
    ("vc1", "Clear glass 1", nos, "nós"),
):
    L, sat, cro, s = numeros(d[k])
    print(
        f"{nome:18s} {quem:6s}: L/L_papel {L:.3f} · saturação {sat:.3f} · croma {cro:5.1f} · "
        f"sombra {s:.3f} (÷ opaca {s / base[quem]:.2f})"
    )

# O miolo dos gates do produto (`miolo_e_papel_nu`, gravado pela sonda): as constantes do gate
# `o_modo_blender_casa_com_o_cycles_e_o_clear_glass_e_o_de_antes` são o Cycles NESTA máscara.
try:
    mp = pnm(f"{son}/miolo_produto.pgm") > 127
    for k, nome, d, quem in (
        ("tr05", "Blender 0,5", nos, "nós"),
        ("tr05", "Transmission 0,5", cy, "Cycles"),
        ("tr1", "Blender 1", nos, "nós"),
        ("tr1", "Transmission 1", cy, "Cycles"),
        ("vc1", "Clear glass 1", nos, "nós"),
    ):
        im = d[k]
        s = srgb(im)
        mx, mn = s.max(axis=-1), s.min(axis=-1)
        print(
            f"[miolo do produto] {nome:18s} {quem:6s}: L/L_papel {float(np.mean((lum(im) / lp)[mp])):.3f} · "
            f"saturação {float(np.mean(((mx - mn) / np.maximum(mx, 1.0))[mp])):.3f}"
        )
except FileNotFoundError:
    pass

# A varredura: o erro relativo de L e da saturação a 0,5 e a 1 contra o Cycles (a soma dos quatro |Δ| relativos).
alvo = {v: numeros(cy[f"tr{v}"]) for v in ("05", "1")}
ms = sorted({float(re.search(r"nos_m([0-9.]+)_1\.ppm", f).group(1)) for f in glob.glob(f"{son}/nos_m*_1.ppm")})
if ms:
    print("# varredura (m → k): L/papel · saturação a 0,5 | a 1 · Σ|Δ|/Cycles (L e saturação)")
for m in [0.0] + ms:
    r = {}
    for v in ("05", "1"):
        r[v] = numeros(nos[f"tr{v}"] if m == 0.0 else lin(pnm(f"{son}/nos_m{m:g}_{v}.ppm")))
    erro = sum(abs(r[v][i] - alvo[v][i]) / alvo[v][i] for v in ("05", "1") for i in (0, 1))
    k = f" k {kb + (kv - kb) * m:.3f}" if kb is not None else ""
    print(
        f"m {m:.4f}{k}: 0,5 {r['05'][0]:.3f} · {r['05'][1]:.3f} | 1 {r['1'][0]:.3f} · {r['1'][1]:.3f} · "
        f"croma 1 {r['1'][2]:5.1f} · Σ {erro:.3f}"
    )

# A folha: Transmission 0 / 0,5 / 1 em três fileiras (Blender · Clear glass · Cycles) e, em baixo, a orla a 4×
# (Blender 1 · Clear glass 1 · Cycles 1).
y0, y1, x0, x1 = 150, 260, 90, 610
h, w = y1 - y0, x1 - x0
tw, th = int(w * 0.6), int(h * 0.6)
m = 18
linhas = [
    ("Blender · Transmission 0 / 0,5 / 1", (nos["t0"], nos["tr05"], nos["tr1"])),
    ("Clear glass · Transmission 0 / 0,5 / 1", (nos["t0"], nos["vc05"], nos["vc1"])),
    ("Cycles (Principled) · Transmission 0 / 0,5 / 1", (cy["t0"], cy["tr05"], cy["tr1"])),
]
zy, zx, zs = 228, 300, 48
out = Image.new("RGB", (3 * tw + 32, len(linhas) * (th + m) + zs * 4 + m + 16), (40, 40, 40))
d = ImageDraw.Draw(out)
for j, (titulo, ims) in enumerate(linhas):
    y = 4 + j * (th + m)
    d.text((8, y), titulo, fill=(230, 230, 230))
    for i, im in enumerate(ims):
        q = srgb(im)[y0:y1, x0:x1].astype(np.uint8)
        out.paste(Image.fromarray(q).resize((tw, th)), (8 + i * (tw + 8), y + m - 2))
y = 4 + len(linhas) * (th + m)
d.text((8, y), "a 4x na orla, Transmission 1: Blender · Clear glass · Cycles", fill=(230, 230, 230))
zw = (out.width - 32) // 3
for i, im in enumerate((nos["tr1"], nos["vc1"], cy["tr1"])):
    q = srgb(im)[zy : zy + zs, zx : zx + zw // 4].astype(np.uint8)
    out.paste(Image.fromarray(q).resize((zw, zs * 4), Image.NEAREST), (8 + i * (zw + 8), y + m - 2))
out.save(folha, quality=92)
print(f"[modos] {folha}")
