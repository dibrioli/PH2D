"""Régua da TRANSMISSION e do RELEVO DO PAPEL (W3b, dono 27/10: «a tinta era viva»): o produto × o Cycles com o
Principled na MESMA malha (bloco ALFA do `cycles.py`, `tr05`/`tr1` com o «Transmission Weight»), ao lado do Alpha
(`transp*`), sobre o papel Cold texturizado (a cena de `medida_oleo_real_transparencia`: Dabber azul grosso, Relief 0,6).

    python3 transparencia_regua.py prepara <sonda>         # as entradas do cycles.py
    python3 transmissao_regua.py <sonda> <saida> <folha.jpg>

Números no miolo (cobertura ≥ 0,99 erodida 6): L linear ÷ papel; a SATURAÇÃO média `(max − min)/max` em sRGB (a tinta
«viva»); o croma `max − min`; o grão = a correlação do passa-alto da luminância com o do papel sem tinta (o relevo da própria
tinta domina a amplitude; no Cycles o papel vê-se refractado). Sombra projectada no papel nu (a região do `transparencia_regua.py`) e a razão a T 0.
O Relevo do papel não tem variante no Cycles (seria a malha da tinta + o dente do papel): só os nossos números.
"""

import sys

import numpy as np
from PIL import Image, ImageDraw

# As mesmas réguas do `transparencia_regua.py` (que corre ao ser importado).
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


def passa_alto(v, r=3):
    k = np.zeros_like(v)
    n = 0
    for dy in range(-r, r + 1):
        for dx in range(-r, r + 1):
            k += np.roll(np.roll(v, dy, 0), dx, 1)
            n += 1
    return v - k / n


import OpenImageIO as oiio  # noqa: E402

son, sai, folha = sys.argv[1], sys.argv[2], sys.argv[3]
cob = pnm(f"{son}/entrada/cobertura.pgm") / 255.0
papel = lin(pnm(f"{son}/papel.ppm"))
lp = lum(papel)
miolo = erode(cob >= 0.99, 6)
tinta = cob > 0.0
perto = dilata(tinta, 30) & ~dilata(tinta, 1)
nos = {}
for k in ("t0", "t05", "t1", "tr05", "tr1", "rp05", "rp1"):
    nos[k] = lin(pnm(f"{son}/nos_{k}.ppm"))
cy = {}
for k, c in (("t0", "transp0"), ("t05", "transp05"), ("t1", "transp1"), ("tr05", "tr05"), ("tr1", "tr1")):
    cy[k] = oiio.ImageBuf(f"{sai}/alfa_{c}_cor.exr").get_pixels(oiio.FLOAT)[..., :3].astype(np.float64)
sombra = perto & (lum(cy["t0"]) < 0.95 * lp)
print(f"miolo {miolo.sum()} células · sombra projectada (Cycles T 0) {sombra.sum()} células")


def numeros(im):
    s = srgb(im)
    mx, mn = s.max(axis=-1), s.min(axis=-1)
    sat = float(np.mean(((mx - mn) / np.maximum(mx, 1.0))[miolo]))
    cro = float(np.mean((mx - mn)[miolo]))
    l = lum(im)
    hp = passa_alto(l)
    grao = float(np.corrcoef(hp[miolo], passa_alto(lp)[miolo])[0, 1])
    return float(np.mean(l[miolo] / lp[miolo])), sat, cro, grao, float(np.mean(1.0 - l[sombra] / lp[sombra]))


grao_papel = float(np.std(passa_alto(lp)[miolo]) / np.mean(lp[miolo]))
print(f"grão relativo do papel sem tinta no miolo: {grao_papel:.4f}")
base = {}
for k, nome in (
    ("t0", "opaca"),
    ("t05", "Alpha 0,5"),
    ("t1", "Alpha 1"),
    ("tr05", "Transmission 0,5"),
    ("tr1", "Transmission 1"),
    ("rp05", "Relevo do papel 0,5"),
    ("rp1", "Relevo do papel 1"),
):
    for quem, d in (("nós", nos), ("Cycles", cy)):
        if k not in d:
            continue
        L, sat, cro, g, s = numeros(d[k])
        base.setdefault(quem, s)
        print(
            f"{nome:20s} {quem:6s}: L/L_papel {L:.3f} · saturação {sat:.3f} · croma {cro:5.1f} · "
            f"grão corr {g:.2f} · sombra {s:.3f} (÷ T0 {s / base[quem]:.2f})"
        )

# A folha: um bloco por controlo (Alpha · Transmission · Paper Relief), cada um com nós em cima e o Cycles em
# baixo a 0 · 0,5 · 1; no fim um recorte a 4× da orla de baixo (o grão do papel nu ao lado da tinta).
y0, y1, x0, x1 = 150, 260, 90, 610
h, w = y1 - y0, x1 - x0
tw, th = int(w * 0.6), int(h * 0.6)
blocos = [("Alpha", ("t0", "t05", "t1")), ("Transmission", ("t0", "tr05", "tr1")), ("Paper Relief", ("t0", "rp05", "rp1"))]
m = 18
linhas = [(f"{rot} · {nome} 0 / 0,5 / 1", d, ks) for nome, ks in blocos for rot, d in (("nos", nos), ("Cycles", cy))]
zy, zx, zs = 228, 300, 48
out = Image.new("RGB", (3 * tw + 32, len(linhas) * (th + m) + zs * 4 + m + 16), (40, 40, 40))
d = ImageDraw.Draw(out)
for j, (titulo, linha, ks) in enumerate(linhas):
    y = 4 + j * (th + m)
    d.text((8, y), titulo, fill=(230, 230, 230))
    for i, k in enumerate(ks):
        x = 8 + i * (tw + 8)
        if k in linha:
            im = srgb(linha[k])[y0:y1, x0:x1].astype(np.uint8)
            out.paste(Image.fromarray(im).resize((tw, th)), (x, y + m - 2))
        else:
            d.text((x + 10, y + m + th // 2), "(sem variante no Cycles)", fill=(160, 160, 160))
y = 4 + len(linhas) * (th + m)
d.text((8, y), "a 4x: nos opaca · nos Paper Relief 1 · nos Transmission 1 · Cycles Transmission 1", fill=(230, 230, 230))
zw = (out.width - 40) // 4
for i, im in enumerate((nos["t0"], nos["rp1"], nos["tr1"], cy["tr1"])):
    q = srgb(im)[zy : zy + zs, zx : zx + zw // 4].astype(np.uint8)
    out.paste(Image.fromarray(q).resize((zw, zs * 4), Image.NEAREST), (8 + i * (zw + 8), y + m - 2))
out.save(folha, quality=92)
print(f"[transmissao] {folha}")
