#!/usr/bin/env python3
"""§BUG-bordas-W3b — a régua dos PONTINHOS ESCUROS soltos na orla e da LARGURA da orla (foto do dono 7.png).

Tinta = «azul» = clamp((B − R)/120) ≥ ½ (tinta azul sobre papel claro). Orla = pixels a ≤ 3 px (Chebyshev) do
contorno da tinta, dos dois lados. Um PONTINHO é um pixel da orla com luminância (Rec.601) abaixo do p5 da tinta
INTERIOR (a > 4 px do contorno) e abaixo de METADE da mediana dela (o escuro que a foto do dono mostra: 20–55 sobre
~90; a textura do Rebelle tem salpicos mais escuros que o p5, nenhum abaixo de metade) E ≥ `DEGRAU` níveis abaixo da
mediana dos 8 vizinhos; os pixels assim juntam-se em
componentes 8-conexas e conta-se as de ≤ 4 px (as soltas — uma linha escura contínua não conta aqui, conta-a o
`bordas_regua.py`). `largura` = a mediana, nas colunas onde a borda é quase horizontal, da distância (px) entre os
cruzamentos de «azul» 0,1 e 0,9 — a orla borrada (o Smooth Edges de 6 px dá ~6; uma borda nítida ~1–2).

    python3 -I pontinhos_regua.py <img> [x0 x1 y0 y1] [--marca=<png com os soltos a vermelho>]
"""
import sys

import numpy as np
from PIL import Image

DEGRAU = 20.0
ORLA = 3


def mede(caminho, caixa=None):
    a = np.asarray(Image.open(caminho).convert("RGB"), dtype=np.float64)
    if caixa:
        x0, x1, y0, y1 = caixa
        a = a[y0:y1, x0:x1]
    r, g, b = a[..., 0], a[..., 1], a[..., 2]
    azul = np.clip((b - r) / 120.0, 0, 1)
    lum = 0.299 * r + 0.587 * g + 0.114 * b
    tinta = azul >= 0.5
    h, w = tinta.shape

    def desloca(m, dx, dy, fill):
        o = np.full_like(m, fill)
        ys, yd = (slice(max(dy, 0), h), slice(0, h - max(dy, 0))) if dy >= 0 else (slice(0, h + dy), slice(-dy, h))
        xs, xd = (slice(max(dx, 0), w), slice(0, w - max(dx, 0))) if dx >= 0 else (slice(0, w + dx), slice(-dx, w))
        o[ys, xs] = m[yd, xd]
        return o

    viz = [(dx, dy) for dx in (-1, 0, 1) for dy in (-1, 0, 1) if dx or dy]
    contorno = np.zeros_like(tinta)
    for dx, dy in viz:
        contorno |= tinta & ~desloca(tinta, dx, dy, True)
    # distância de Chebyshev ao contorno por dilatações
    perto = contorno.copy()
    dist = np.where(contorno, 0, 99)
    for k in range(1, 6):
        nova = perto.copy()
        for dx, dy in viz:
            nova |= desloca(perto, dx, dy, False)
        dist = np.where(nova & ~perto, k, dist)
        perto = nova
    orla = dist <= ORLA
    interior = tinta & (dist > 4)
    if interior.sum() < 50:
        return None
    p5 = np.percentile(lum[interior], 5)
    meio = 0.5 * np.median(lum[interior])
    vizs = np.stack([desloca(lum, dx, dy, np.nan) for dx, dy in viz])
    med = np.nanmedian(vizs, axis=0)
    cand = orla & (lum < min(p5, meio)) & (lum <= med - DEGRAU)
    # componentes 8-conexas
    vista = np.zeros_like(cand)
    soltos, linhas, onde = 0, 0, []
    ys, xs = np.nonzero(cand)
    for y, x in zip(ys, xs):
        if vista[y, x]:
            continue
        pilha, n, comp = [(y, x)], 0, []
        vista[y, x] = True
        while pilha:
            cy, cx = pilha.pop()
            n += 1
            comp.append((cy, cx))
            for dx, dy in viz:
                ny, nx = cy + dy, cx + dx
                if 0 <= ny < h and 0 <= nx < w and cand[ny, nx] and not vista[ny, nx]:
                    vista[ny, nx] = True
                    pilha.append((ny, nx))
        if n <= 4:
            soltos += 1
            onde += comp
        else:
            linhas += 1
    # A orla EXTERNA (o que as setas do dono apontam): a ≤ 4 px do PAPEL ABERTO (papel com < 20 % de tinta na
    # janela 9×9); o resto são os salpicos do interior da textura (o Rebelle tem-nos: 81 no Dabber inteiro, 1 na orla).
    pad = np.pad(tinta, 4)
    jan = np.zeros((h, w))
    for dy in range(9):
        for dx in range(9):
            jan += pad[dy:dy + h, dx:dx + w]
    aberto = ~tinta & (jan / 81.0 < 0.2)
    perto = aberto.copy()
    for _ in range(4):
        nova = perto.copy()
        for dx, dy in viz:
            nova |= desloca(perto, dx, dy, False)
        perto = nova
    externos = sum(1 for y, x in onde if perto[y, x])
    # largura da orla nas colunas: cada transição vertical de azul 0,9 → 0,1 (para baixo e para cima)
    larg = []
    for x in range(w):
        c = azul[:, x]
        for y in range(1, h - 1):
            if c[y - 1] >= 0.5 > c[y] or c[y - 1] < 0.5 <= c[y]:
                desce = c[y - 1] >= 0.5
                passo = 1 if desce else -1
                y_dentro = y - 1 if desce else y
                # anda para dentro até 0,9 e para fora até 0,1
                i = y_dentro
                while 0 <= i < h and c[i] < 0.9 and abs(i - y_dentro) < 12:
                    i -= passo
                o = y_dentro
                while 0 <= o < h and c[o] > 0.1 and abs(o - y_dentro) < 12:
                    o += passo
                if 0 <= i < h and 0 <= o < h and c[i] >= 0.9 and c[o] <= 0.1:
                    # os cruzamentos interpolados: 0,9 entre i e i+passo; 0,1 entre o−passo e o
                    def cruza(p, q, alvo):
                        return p + (q - p) * ((c[p] - alvo) / (c[p] - c[q])) if c[p] != c[q] else q

                    larg.append(abs(cruza(o - passo, o, 0.1) - cruza(i, i + passo, 0.9)))
    largura = float(np.median(larg)) if larg else float("nan")
    return {
        "soltos": soltos,
        "externos": externos,
        "linhas": linhas,
        "px_cand": int(cand.sum()),
        "orla_px": int(orla.sum()),
        "p5": float(p5),
        "largura": largura,
        "onde": onde,
    }


def main():
    caminho = sys.argv[1]
    nums = [v for v in sys.argv[2:] if not v.startswith("--")]
    caixa = tuple(int(v) for v in nums[:4]) if len(nums) >= 4 else None
    m = mede(caminho, caixa)
    marca = dict(a.split("=", 1) for a in sys.argv[1:] if a.startswith("--marca="))
    if m and marca:
        im = Image.open(caminho).convert("RGB")
        px = im.load()
        ox, oy = (caixa[0], caixa[2]) if caixa else (0, 0)
        for y, x in m["onde"]:
            px[x + ox, y + oy] = (255, 0, 0)
        im.save(marca["--marca"])
    if m is None:
        print(f"{caminho}: sem tinta")
        return
    print(
        f"{caminho.split('/')[-1]:<34} SOLTOS na orla {m['externos']:4d} (todos {m['soltos']:4d}) · linhas {m['linhas']:3d} · px {m['px_cand']:5d}"
        f" / orla {m['orla_px']:6d} · p5 {m['p5']:.0f} · LARGURA 10–90 {m['largura']:.2f} px"
    )


if __name__ == "__main__":
    main()
