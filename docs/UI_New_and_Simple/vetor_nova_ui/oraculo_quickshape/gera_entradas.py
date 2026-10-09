#!/usr/bin/env python3
"""Gera os traços NOSSOS que o oráculo (Xournal++) vai reconhecer — um ficheiro por traço, em px da
tela virtual (1600×1000; a área útil é x 400..1400, y 150..900), ~4 px entre amostras como uma mão.

Cada traço VARIA UMA GRANDEZA (protocolo `docs/_ComoInvestigarApps/00_o_metodo.md` §3 passo 4) e o
cabeçalho do ficheiro diz qual. Determinístico (semente fixa) — correr duas vezes dá o mesmo corpus.
"""
import math
import os
import random
import sys

OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(os.path.dirname(__file__), "entradas")
os.makedirs(OUT, exist_ok=True)
rng = random.Random(20261008)
STEP = 4.0

ENTRADAS = []


def entrada(nome, varia, pts):
    ENTRADAS.append((nome, varia, pts))


def resample(poly, step=STEP):
    """Amostra uma polilinha a `step` px (como um rato/dedo a 60–120 Hz)."""
    out = [poly[0]]
    carry = 0.0
    for (x0, y0), (x1, y1) in zip(poly, poly[1:]):
        d = math.hypot(x1 - x0, y1 - y0)
        if d == 0:
            continue
        t = carry
        while t <= d:
            out.append((x0 + (x1 - x0) * t / d, y0 + (y1 - y0) * t / d))
            t += step
        carry = t - d
    if out[-1] != poly[-1]:
        out.append(poly[-1])
    return out


def wobble(pts, amp):
    """Ruído de mão: ±amp px, suave (média móvel de 5)."""
    if amp == 0:
        return pts
    n = len(pts)
    noise = [(rng.uniform(-amp, amp), rng.uniform(-amp, amp)) for _ in range(n)]
    sm = []
    for i in range(n):
        w = noise[max(0, i - 2): i + 3]
        sm.append((sum(a for a, _ in w) / len(w), sum(b for _, b in w) / len(w)))
    return [(x + dx, y + dy) for (x, y), (dx, dy) in zip(pts, sm)]


def polygon(cx, cy, r, n, rot=0.0, close=True, gap=0.0):
    corners = [(cx + r * math.cos(rot + 2 * math.pi * k / n), cy + r * math.sin(rot + 2 * math.pi * k / n)) for k in range(n)]
    poly = corners + ([corners[0]] if close else [])
    pts = resample(poly)
    if gap > 0:
        # deixa `gap` px por fechar no fim
        keep = []
        for p in pts:
            keep.append(p)
        total = 0.0
        while len(keep) > 2 and total < gap:
            a, b = keep[-2], keep[-1]
            total += math.hypot(b[0] - a[0], b[1] - a[1])
            keep.pop()
        pts = keep
    return pts


def rect(cx, cy, w, h, rot=0.0, gap=0.0):
    c, s = math.cos(rot), math.sin(rot)
    base = [(-w / 2, -h / 2), (w / 2, -h / 2), (w / 2, h / 2), (-w / 2, h / 2), (-w / 2, -h / 2)]
    poly = [(cx + x * c - y * s, cy + x * s + y * c) for x, y in base]
    pts = resample(poly)
    if gap > 0:
        total = 0.0
        while len(pts) > 2 and total < gap:
            a, b = pts[-2], pts[-1]
            total += math.hypot(b[0] - a[0], b[1] - a[1])
            pts.pop()
    return pts


def ellipse(cx, cy, rx, ry, a0=0.0, a1=2 * math.pi, rot=0.0):
    n = max(24, int(abs(a1 - a0) * max(rx, ry) / STEP))
    c, s = math.cos(rot), math.sin(rot)
    pts = []
    for k in range(n + 1):
        a = a0 + (a1 - a0) * k / n
        x, y = rx * math.cos(a), ry * math.sin(a)
        pts.append((cx + x * c - y * s, cy + x * s + y * c))
    return pts


def line(x0, y0, x1, y1):
    return resample([(x0, y0), (x1, y1)])


# ── LINHAS: varia o comprimento, a inclinação, o tremor ───────────────────────────────────────────
entrada("linha_horizontal_600", "comprimento 600 px, horizontal, sem tremor", line(500, 500, 1100, 500))
entrada("linha_diagonal_500", "inclinação 30°, 500 px", line(500, 400, 500 + 433, 400 + 250))
entrada("linha_vertical_400", "vertical, 400 px", line(900, 300, 900, 700))
entrada("linha_curta_60", "comprimento 60 px (perto do mínimo do oráculo, 40)", line(900, 500, 960, 500))
entrada("linha_curta_30", "comprimento 30 px (abaixo do mínimo do oráculo, 40)", line(900, 500, 930, 500))
for amp in (2, 6, 12, 24):
    entrada(f"linha_tremor_{amp:02d}", f"tremor ±{amp} px numa linha de 600 px", wobble(line(500, 500, 1100, 500), amp))
entrada("linha_arco_leve", "linha com flecha de 20 px (quase recta)", ellipse(800, 500 + 2260, 2270, 2270, math.radians(-97.6), math.radians(-82.4)))

# ── RECTÂNGULOS: varia a rotação, o fecho, a proporção, o tremor ──────────────────────────────────
entrada("rect_alinhado_400x250", "400×250, alinhado, fechado", rect(800, 500, 400, 250))
entrada("quadrado_300", "quadrado 300, alinhado", rect(800, 500, 300, 300))
for rot in (10, 25, 45):
    entrada(f"rect_rodado_{rot:02d}", f"400×250 rodado {rot}°", rect(800, 500, 400, 250, math.radians(rot)))
for gap in (15, 40, 90):
    entrada(f"rect_aberto_{gap:02d}", f"400×250 alinhado, por fechar {gap} px", rect(800, 500, 400, 250, 0.0, gap))
for amp in (3, 8, 16):
    entrada(f"rect_tremor_{amp:02d}", f"400×250 alinhado, tremor ±{amp} px", wobble(rect(800, 500, 400, 250), amp))
entrada("rect_estreito_500x60", "500×60 (proporção 8:1)", rect(800, 500, 500, 60))
entrada("rect_pequeno_70x50", "70×50 (pequeno)", rect(800, 500, 70, 50))

# ── POLÍGONOS: varia o número de lados ────────────────────────────────────────────────────────────
for n in (3, 5, 6, 8):
    entrada(f"poligono_{n}_lados", f"{n} lados regulares, raio 200", polygon(800, 500, 200, n, -math.pi / 2))
entrada("triangulo_tremor_06", "triângulo raio 200, tremor ±6 px", wobble(polygon(800, 500, 200, 3, -math.pi / 2), 6))
entrada("triangulo_aberto_40", "triângulo raio 200, por fechar 40 px", polygon(800, 500, 200, 3, -math.pi / 2, True, 40))

# ── CÍRCULOS E ELIPSES: varia o raio, a proporção, o fecho, a rotação ─────────────────────────────
entrada("circulo_r150", "círculo raio 150, fechado", ellipse(800, 500, 150, 150))
entrada("circulo_r40", "círculo raio 40 (pequeno)", ellipse(800, 500, 40, 40))
entrada("circulo_tremor_06", "círculo raio 150, tremor ±6 px", wobble(ellipse(800, 500, 150, 150), 6))
entrada("circulo_volta_e_meia", "círculo raio 150, uma volta e meia", ellipse(800, 500, 150, 150, 0.0, 3 * math.pi))
entrada("elipse_2para1", "elipse 300×150, alinhada", ellipse(800, 500, 300, 150))
entrada("elipse_2para1_rodada_30", "elipse 300×150 rodada 30°", ellipse(800, 500, 300, 150, 0.0, 2 * math.pi, math.radians(30)))
entrada("arco_meio", "meia circunferência raio 200 (aberta)", ellipse(800, 500, 200, 200, math.pi, 2 * math.pi))
entrada("arco_quarto", "um quarto de circunferência raio 200", ellipse(800, 500, 200, 200, math.pi, 1.5 * math.pi))
entrada("circulo_aberto_40", "círculo raio 150, por fechar ~40 px", ellipse(800, 500, 150, 150, 0.0, 2 * math.pi - 40 / 150))

# ── O QUE NÃO É FORMA: zigzag, rabisco, S, L, V ───────────────────────────────────────────────────
entrada("zigzag_4", "4 segmentos em zigzag (polilinha aberta)", resample([(500, 400), (650, 650), (800, 400), (950, 650), (1100, 400)]))
entrada("forma_L", "dois segmentos em L (aberto)", resample([(600, 300), (600, 700), (1000, 700)]))
entrada("forma_V", "dois segmentos em V (aberto)", resample([(600, 300), (800, 700), (1000, 300)]))
entrada("curva_S", "um S suave", ellipse(800, 400, 100, 100, -math.pi / 2, math.pi / 2) + ellipse(800, 600, 100, 100, 1.5 * math.pi, 0.5 * math.pi)[1:])
rab = [(800, 500)]
for _ in range(120):
    x, y = rab[-1]
    rab.append((min(1300, max(500, x + rng.uniform(-40, 40))), min(850, max(200, y + rng.uniform(-40, 40)))))
entrada("rabisco", "rabisco aleatório (semente fixa)", resample(rab))

for nome, varia, pts in ENTRADAS:
    with open(os.path.join(OUT, nome + ".txt"), "w", encoding="utf-8") as f:
        f.write(f"# entrada: {nome}\n# varia: {varia}\n# unidade: px da tela virtual 1600x1000\n# pontos: {len(pts)}\n")
        for x, y in pts:
            f.write(f"{x:.1f} {y:.1f}\n")
print(f"{len(ENTRADAS)} entradas em {OUT}")
