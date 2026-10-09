"""A régua da COBERTURA da orla e da RAIZ da gota (auditoria da W1c, 09/10) — a mesma conta sobre o oráculo
(`fixtures/rebelle_gotas/s_*.png`) e o nosso (PPM de `tests/it/auditoria_borda_w1c.rs`). Portada para o
portão em Rust no mesmo ficheiro de teste.

A régua de `borda.py` mede a POSIÇÃO da orla (salto, rms de uma cúbica): uma escada de 1 px perfeita passa
«lisa» nela. Esta mede o que o dono vê ao ampliar:

- `binaria%`  — colunas da descida em que a orla é 0/1: o último pixel ≥ 0,85·L e o de baixo ≤ 0,15·L
                (L = o pico nos 5 px de cima da orla).
- `meio_tom%` — colunas com um pixel PARCIAL (0,2·L < d < 0,8·L) na passagem orla → papel.
- `escada`    — rms (px) da posição SUB-pixel da orla (`yb − 1 + d[yb]/L + d[yb+1]/L`) contra uma recta
                local de 9 colunas: a escada de 1 px num declive de 0,73 dá ~0,3; uma orla com meio-tom, menos.
- `salto/B0`  — mediana de `d[yb] − d[yb+1]`: o contraste do degrau (a faixa saturada contra o papel).
- `pontos`    — px de tinta (> 25) de 3 a 20 px abaixo da orla, por 100 colunas, fora dos fios.
- raiz de cada fio que cruza y=330: `degrau` = |orla à direita − orla à esquerda| extrapoladas à coluna
  do fio (rectas sobre ±4..20 colunas); `larg@k` = largura a meia altura k px abaixo da raiz; `desvio@k` =
  coluna do fio a k px da raiz − coluna a y=330. Controlo: o mesmo `degrau` em pontos da descida longe dos fios.

uso: python3 -I borda_cobertura.py B0:img [B0:img ...]
     python3 -I borda_cobertura.py --amplia SAIDA.png ESCALA x0,y0,x1,y1 rotulo:img [rotulo:img ...]
"""
import math
import os
import sys

import numpy as np
from PIL import Image, ImageDraw

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import borda as B  # noqa: E402

DESCIDA = range(440, 633)
KS = (1, 2, 4, 8, 16, 32)


def cobertura(d, b0, xs=DESCIDA):
    t = 0.12 * b0
    bin_, meio, salto, pos, ybs = 0, 0, [], [], []
    for x in xs:
        yb = B.orla(d, x, t)
        L = float(d[yb - 4 : yb + 1, x].max())
        a, b = float(d[yb, x]), float(d[yb + 1, x])
        bin_ += a >= 0.85 * L and b <= 0.15 * L
        meio += any(0.2 * L < float(d[y, x]) < 0.8 * L for y in range(yb - 1, yb + 3))
        salto.append((a - b) / b0)
        pos.append(yb - 1 + min(a / L, 1.0) + min(b / L, 1.0))
        ybs.append(yb)
    pos = np.array(pos)
    res = []
    for i in range(4, len(pos) - 4):
        w = np.arange(-4, 5)
        p = np.polyfit(w, pos[i - 4 : i + 5], 1)
        res.append(pos[i] - p[1])
    n = len(xs)
    return {
        "binaria%": round(100.0 * bin_ / n, 1),
        "meio_tom%": round(100.0 * meio / n, 1),
        "escada": round(math.sqrt(float(np.mean(np.square(res)))), 3),
        "salto/B0": round(float(np.median(salto)), 2),
    }, ybs


def corrida(d, x, y, t):
    if d[y, x] <= t:
        return 0
    l = x
    while l > 0 and d[y, l - 1] > t:
        l -= 1
    r = x
    while r < d.shape[1] - 1 and d[y, r + 1] > t:
        r += 1
    return r - l + 1


def fwhm(d, y, c):
    lo, hi = max(0, c - 6), min(d.shape[1], c + 7)
    row = d[y, lo:hi].astype(float)
    m = row.max()
    k = int(np.argmax(row))
    a = k
    while a > 0 and row[a - 1] >= m / 2:
        a -= 1
    b = k
    while b < len(row) - 1 and row[b + 1] >= m / 2:
        b += 1
    return b - a + 1, lo + (a + b) / 2.0, m


def recta(d, t, xs):
    xs = [x for x in xs if 1 <= x < d.shape[1] - 1]
    ys = [B.orla(d, x, t) for x in xs]
    return np.polyfit(xs, ys, 1) if len(xs) >= 4 else None


def degrau_em(d, t, c):
    pl = recta(d, t, range(c - 20, c - 3))
    pr = recta(d, t, range(c + 4, c + 21))
    if pl is None or pr is None:
        return None
    return abs(np.polyval(pr, c) - np.polyval(pl, c))


def raizes(d, b0, y=330):
    t = 0.12 * b0
    out = []
    for c, _fim in B.comprimentos_d(d, b0, y):
        cc, yy = c, y
        while yy > 1:
            yy -= 1
            seg = d[yy, max(0, cc - 2) : cc + 3]
            if seg.max() <= t:
                break
            cc = max(0, cc - 2) + int(np.argmax(seg))
            if corrida(d, cc, yy, t) > 8:
                break
        raiz = yy
        cols = {}
        for k in KS:
            if raiz + k < y:
                w, centro, _ = fwhm(d, raiz + k, cc)
                cols[k] = (w, centro)
        _, c330, _ = fwhm(d, y, c)
        out.append(
            {
                "x": c,
                "raiz": raiz,
                "degrau": None if (g := degrau_em(d, t, c)) is None else round(float(g), 1),
                "larg": {k: v[0] for k, v in cols.items()},
                "desvio": {k: round(v[1] - c330, 1) for k, v in cols.items()},
            }
        )
    return out


def pontos(d, ybs, fios_x, xs=DESCIDA):
    n = 0
    for x, yb in zip(xs, ybs):
        if any(abs(x - f) <= 4 for f in fios_x):
            continue
        n += int((d[yb + 3 : yb + 21, x] > 25).sum())
    return round(100.0 * n / len(xs), 1)


def mede(path, b0):
    d = B.escuro(path)
    cob, ybs = cobertura(d, b0)
    rz = raizes(d, b0)
    t = 0.12 * b0
    fios_x = [r["x"] for r in rz]
    cob["pontos"] = pontos(d, ybs, fios_x)
    ctrl = [degrau_em(d, t, c) for c in range(450, 625, 12) if all(abs(c - f) > 24 for f in fios_x)]
    ctrl = [g for g in ctrl if g is not None]
    cob["degrau_ctrl"] = round(float(np.median(ctrl)), 2) if ctrl else None
    graus = [r["degrau"] for r in rz if r["degrau"] is not None]
    cob["degrau_raiz"] = round(float(np.median(graus)), 2) if graus else None
    for k in KS:
        ws = [r["larg"][k] for r in rz if k in r["larg"]]
        cob[f"larg@{k}"] = float(np.median(ws)) if ws else None
    for k in (2, 8, 32):
        ds = [abs(r["desvio"][k]) for r in rz if k in r["desvio"]]
        cob[f"|desvio|@{k}"] = round(float(np.median(ds)), 1) if ds else None
    cob["fios"] = len(rz)
    return cob, rz


def amplia(saida, escala, caixa, pares):
    x0, y0, x1, y1 = caixa
    ims = []
    for rot, p in pares:
        im = Image.open(p).convert("RGB").crop((x0, y0, x1, y1))
        im = im.resize(((x1 - x0) * escala, (y1 - y0) * escala), Image.BILINEAR)
        ims.append((rot, im))
    w = sum(im.width for _, im in ims) + 12 * (len(ims) - 1)
    h = ims[0][1].height + 22
    out = Image.new("RGB", (w, h), (255, 255, 255))
    dr = ImageDraw.Draw(out)
    x = 0
    for rot, im in ims:
        out.paste(im, (x, 22))
        dr.text((x + 4, 4), rot, fill=(0, 0, 0))
        x += im.width + 12
    out.save(saida)


def _comprimentos_d(d, b0, y=330):
    t = 0.12 * b0
    lin = d[y]
    out, x = [], 0
    while x < len(lin):
        if lin[x] > t:
            a = x
            while x < len(lin) and lin[x] > t:
                x += 1
            out.append((a + int(np.argmax(lin[a:x])), None))
        x += 1
    return out


B.comprimentos_d = _comprimentos_d

if __name__ == "__main__":
    if sys.argv[1] == "--amplia":
        saida, escala, caixa = sys.argv[2], int(sys.argv[3]), [int(v) for v in sys.argv[4].split(",")]
        amplia(saida, escala, caixa, [a.split(":", 1) for a in sys.argv[5:]])
        sys.exit(0)
    for a in sys.argv[1:]:
        b0, p = a.split(":", 1)
        m, rz = mede(p, float(b0))
        print(f"{p.rsplit('/', 1)[-1]:<22}", m)
        if os.environ.get("RAIZES"):
            for r in rz:
                print("    ", r)
