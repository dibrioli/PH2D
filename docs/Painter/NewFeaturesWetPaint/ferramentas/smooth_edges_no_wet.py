"""O «Smooth Edges» do Watercolor (`watercolor_aa.rs::aa_coverage`) aplicado, FORA do produto, a uma tela do
Wet Paint (PPM das sondas) — para avaliar se a mesma lei cura a orla 0/1 da linha de contacto antes de a portar.

A lei, a mesma conta: o campo de cobertura é amostrado bilinear em 3×3 pontos a ±0,667 texel, endurecido por
`smoothstep(SS0, SS1)`; sem gradiente, ou sem vão (`mx − mn`), o pixel fica como está (byte-idêntico); com vão
cheio a FORMA é `alpha = média ÷ máx` (linear, sobre o pixel acabado) e o SOMBREADO é o da lavagem um pouco
para dentro (a amostra máxima); entre os dois, interpolado por `smoothstep(0, AA_SPAN_MIN, mx − mn)`.

O campo de cobertura do Wet Paint não existe como no Watercolor: aqui é a MANCHA (escuro > `t`), 0/1 por pixel.
O sombreado do pixel de fora é a cor do vizinho de dentro na direcção da amostra máxima (emulação: no produto
seria o pigmento da célula nessa amostra).

uso: python3 -I smooth_edges_no_wet.py entrada.ppm saida.ppm [limiar_escuro=25]
"""
import sys

import numpy as np
from PIL import Image

SS0, SS1 = 0.12, 0.60  # watercolor_render.rs (SS0, SS1)
AA_SS = (-0.667, 0.0, 0.667)
AA_SPAN_MIN = 0.20


def smoothstep(e0, e1, x):
    t = np.clip((x - e0) / (e1 - e0), 0.0, 1.0)
    return t * t * (3.0 - 2.0 * t)


def bilinear(src, fx, fy):
    h, w = src.shape
    fx = np.clip(fx, 0, w - 1)
    fy = np.clip(fy, 0, h - 1)
    x0 = np.floor(fx).astype(int)
    y0 = np.floor(fy).astype(int)
    x1 = np.minimum(x0 + 1, w - 1)
    y1 = np.minimum(y0 + 1, h - 1)
    tx, ty = fx - x0, fy - y0
    a, b, c, d = src[y0, x0], src[y0, x1], src[y1, x0], src[y1, x1]
    top = a + (b - a) * tx
    bot = c + (d - c) * tx
    return top + (bot - top) * ty, np.abs((b - a) * (1 - ty) + (d - c) * ty) + np.abs((c - a) * (1 - tx) + (d - b) * tx)


def aplica(rgb, t_escuro=25):
    rgb = rgb.astype(np.float64)
    escuro = 255.0 - rgb.min(axis=2)
    mask = (escuro > t_escuro).astype(np.float64)
    h, w = mask.shape
    gy, gx = np.mgrid[0:h, 0:w].astype(np.float64)
    val, grad = bilinear(mask, gx, gy)
    single = smoothstep(SS0, SS1, val)
    cs, offs = [], []
    for oy in AA_SS:
        for ox in AA_SS:
            cs.append(smoothstep(SS0, SS1, bilinear(mask, gx + ox, gy + oy)[0]))
            offs.append((ox, oy))
    cs = np.stack(cs)
    ss, mx, mn = cs.mean(0), np.maximum(cs.max(0), single), np.minimum(cs.min(0), single)
    k = cs.argmax(0)
    t = smoothstep(0.0, AA_SPAN_MIN, mx - mn)
    vivo = (grad > 0) & (mx > 0) & (t > 0)
    alpha = np.where(vivo, 1.0 + (np.divide(ss, mx, out=np.ones_like(ss), where=mx > 0) - 1.0) * t, 1.0)
    # O sombreado: dentro da mancha a cor do próprio pixel; fora, a do vizinho de dentro na direcção da
    # amostra máxima.
    ox = np.array([o[0] for o in offs])[k]
    oy = np.array([o[1] for o in offs])[k]
    sx = np.clip(np.rint(gx + np.sign(ox)), 0, w - 1).astype(int)
    sy = np.clip(np.rint(gy + np.sign(oy)), 0, h - 1).astype(int)
    fora = vivo & (mask == 0)
    cor = rgb.copy()
    cor[fora] = rgb[sy[fora], sx[fora]]
    papel = 255.0
    out = papel + (cor - papel) * alpha[..., None]
    return np.clip(np.rint(out), 0, 255).astype(np.uint8), int(vivo.sum())


if __name__ == "__main__":
    ent, sai = sys.argv[1], sys.argv[2]
    t = float(sys.argv[3]) if len(sys.argv) > 3 else 25.0
    out, n = aplica(np.asarray(Image.open(ent).convert("RGB")), t)
    Image.fromarray(out).save(sai)
    print(f"{sai}: {n} px tocados")
