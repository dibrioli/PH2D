#!/usr/bin/env python3
"""Régua da ESPESSURA e do CANTO do traço à mão numa captura do Excalidraw que o dono mandou (07/10,
`capturas_excalidraw/formas_finas_do_dono.png`): «os traços do Excalidraw são mais finos, delicados e
artesanais». Mede, nunca a olho:

- a caixa do rectângulo e o RAIO do canto dele (px) — o raio do Excalidraw é `min(lado menor/4, 32)`
  em unidades do mundo (W1, `ph2d_board_geom::corner_radius`): se o raio medido é MENOR que ¼ do lado
  menor em px, o tecto 32 manda e a escala da captura é `raio_px / 32`;
- a TINTA de cada traço (soma da escuridão numa coluna que atravessa uma aresta direita, em px) e o
  número de picos (traço duplo do rough.js) — a espessura em mundo é `tinta / picos / escala`.

uso: python3 -I mede_traco_excalidraw.py <captura.png>
"""
import sys
from PIL import Image

im = Image.open(sys.argv[1]).convert("L")
W, H = im.size
px = im.load()
dark = lambda x, y: 1.0 - px[x, y] / 255.0

def box(x0, y0, x1, y1, thr=0.5):
    xs, ys = [], []
    for y in range(y0, y1):
        for x in range(x0, x1):
            if dark(x, y) > thr:
                xs.append(x); ys.append(y)
    return min(xs), min(ys), max(xs), max(ys)

# O rectângulo: a região de cima à direita da captura (acima do topo da elipse, y ≈ 285).
rx0, ry0, rx1, ry1 = box(int(W * 0.37), 0, W, int(H * 0.41))
print(f"rectângulo: x {rx0}..{rx1}  y {ry0}..{ry1}  ({rx1 - rx0} × {ry1 - ry0} px)")

def ink_column(x, ya, yb):
    col = [dark(x, y) for y in range(ya, yb)]
    tinta = sum(col)
    picos = sum(1 for i in range(1, len(col) - 1) if col[i] > 0.35 and col[i] >= col[i - 1] and col[i] > col[i + 1])
    return tinta, picos

# A aresta de cima, longe dos cantos: colunas no terço do meio.
tintas, picos = [], []
for x in range(rx0 + (rx1 - rx0) // 3, rx1 - (rx1 - rx0) // 3, 3):
    t, p = ink_column(x, ry0 - 6, ry0 + 10)
    tintas.append(t); picos.append(p)
tm = sum(tintas) / len(tintas)
pm = sum(picos) / len(picos)
print(f"aresta de cima: tinta média {tm:.2f} px por coluna, picos médios {pm:.2f} (n={len(tintas)})")

# O raio do canto de cima à esquerda: a 1.ª coluna (a partir da esquerda) em que a aresta de cima já
# está a menos de 1,5 px do topo.
def top_y(x):
    for y in range(ry0 - 3, ry0 + (ry1 - ry0) // 2):
        if dark(x, y) > 0.5:
            return y
    return None
flat = next(x for x in range(rx0, rx1) if (top_y(x) or 10**9) <= ry0 + 1.5)
r_px = flat - rx0
lado = min(rx1 - rx0, ry1 - ry0)
print(f"raio do canto ≈ {r_px} px  (¼ do lado menor = {lado / 4:.1f} px)")
if r_px < lado / 4 - 3:
    s = r_px / 32.0
    print(f"⇒ o tecto 32 manda: escala {s:.3f} px por unidade; espessura de um traço ≈ {tm / max(pm, 1) / s:.2f} unidades")
else:
    # O raio é ¼ do lado ⇒ o lado menor em mundo é < 128 (senão o tecto 32 mandava) ⇒ a escala é
    # pelo menos `lado_px / 128`, e a espessura de um traço no máximo `tinta / picos / escala`.
    s_min = lado / 128.0
    print(f"⇒ o raio é ¼ do lado: escala ≥ {s_min:.2f} px por unidade; espessura de um traço ≤ {tm / max(pm, 1) / s_min:.2f} unidades")
