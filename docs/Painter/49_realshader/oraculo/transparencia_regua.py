"""Régua da TRANSPARENCY (dono 27/10, `1 − Alpha` do Principled): o produto × o Cycles com o Alpha na MESMA malha,
sobre o papel Cold texturizado (a cena de `medida_oleo_real_transparencia`: Dabber azul grosso, Relief 0,6).

    python3 transparencia_regua.py prepara <sonda>            # albedo/cobertura/papel .png para o cycles.py
    python3 transparencia_regua.py compara <sonda> <saida> <folha.jpg>

`<sonda>/nos_t{0,05,1}.ppm` = a tela final do produto (composite + luz do relevo); `<sonda>/papel.ppm` = a mesma
tela sem tinta; `<saida>/alfa_transp{0,05,1}_cor.exr` = o Cycles (linear → sRGB). Regiões pela cobertura da sonda:
  miolo   — cobertura ≥ 0,99, a ≥ 6 células da orla
  sombra  — papel nu (cobertura 0) a ≤ 30 células da tinta onde o Cycles T 0 escurece o papel > 5 %
Números: a luminância LINEAR ÷ a do papel sem tinta (miolo, sombra), a sombra projectada `1 − L/L_papel` e a
razão dela a T 0; o grão no miolo = correlação do passa-alto da luminância com o do papel.
"""

import os
import sys

import numpy as np
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


def passa_alto(v, r=3):
    k = np.zeros_like(v)
    n = 0
    for dy in range(-r, r + 1):
        for dx in range(-r, r + 1):
            k += np.roll(np.roll(v, dy, 0), dx, 1)
            n += 1
    return v - k / n


if sys.argv[1] == "prepara":
    son = sys.argv[2]
    e = os.path.join(son, "entrada")
    Image.fromarray(pnm(f"{e}/albedo.ppm").astype(np.uint8)).save(f"{e}/albedo.png")
    Image.fromarray(pnm(f"{e}/cobertura.pgm").astype(np.uint8)).save(f"{e}/cobertura.png")
    Image.fromarray(pnm(f"{son}/papel.ppm").astype(np.uint8)).save(f"{e}/papel.png")
    print("[transparencia] entradas .png prontas")
    sys.exit(0)

import OpenImageIO as oiio  # noqa: E402

son, sai, folha = sys.argv[2], sys.argv[3], sys.argv[4]
cob = pnm(f"{son}/entrada/cobertura.pgm") / 255.0
papel = lin(pnm(f"{son}/papel.ppm"))
lp = lum(papel)
tinta = cob > 0.0
miolo = erode(cob >= 0.99, 6)
TS = [("t0", "transp0", 0.0), ("t05", "transp05", 0.5), ("t1", "transp1", 1.0)]
nos = {k: lin(pnm(f"{son}/nos_{k}.ppm")) for k, _, _ in TS}
cy = {k: oiio.ImageBuf(f"{sai}/alfa_{c}_cor.exr").get_pixels(oiio.FLOAT)[..., :3].astype(np.float64) for k, c, _ in TS}
perto = dilata(tinta, 30) & ~dilata(tinta, 1)
sombra = perto & (lum(cy["t0"]) < 0.95 * lp)
print(f"miolo {miolo.sum()} células · sombra projectada (Cycles T 0) {sombra.sum()} células")
hp_papel = passa_alto(lp)
base = {}
for k, _, t in TS:
    for quem, im in (("nós", nos[k]), ("Cycles", cy[k])):
        l = lum(im)
        r_miolo = float(np.mean(l[miolo] / lp[miolo]))
        s = float(np.mean(1.0 - l[sombra] / lp[sombra]))
        base.setdefault(quem, s if t == 0.0 else base.get(quem))
        hp = passa_alto(l)
        grao = float(np.corrcoef(hp[miolo], hp_papel[miolo])[0, 1])
        amp = float(np.std(hp[miolo]) / np.std(hp_papel[miolo]))
        print(
            f"T {t:.1f} {quem:6s}: miolo L/L_papel {r_miolo:.3f} · grão corr {grao:.2f} amp {amp:.2f} · "
            f"sombra projectada {s:.3f} (÷ T0 {s / base[quem]:.2f})"
        )

# A folha: em cima nós, em baixo o Cycles; T 0 · 0,5 · 1, recorte da pincelada.
y0, y1, x0, x1 = 150, 260, 90, 610
cel = []
for linha in (nos, cy):
    cel.append([srgb(linha[k])[y0:y1, x0:x1].astype(np.uint8) for k, _, _ in TS])
h, w = y1 - y0, x1 - x0
m = 22
out = Image.new("RGB", (3 * w + 4 * 8, 2 * (h + m) + 8), (40, 40, 40))
d = ImageDraw.Draw(out)
for j, nome in enumerate(("nos (produto)", "Cycles (Alpha = 1 - T)")):
    for i, (_, _, t) in enumerate(TS):
        x, y = 8 + i * (w + 8), 4 + j * (h + m)
        d.text((x, y), f"{nome} · Transparency {t:g}", fill=(230, 230, 230))
        out.paste(Image.fromarray(cel[j][i]), (x, y + m - 4))
out.save(folha, quality=92)
print(f"[transparencia] {folha}")
