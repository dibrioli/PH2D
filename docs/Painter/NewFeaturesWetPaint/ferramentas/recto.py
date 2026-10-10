#!/usr/bin/env python3
"""A régua do RECTO (W1f, prompt 12): o escorrimento que fica com bordas rectas, «como se rectângulos
invisíveis o cortassem». Lê a orla de BAIXO de um traço (onde a tinta acaba de descer) e devolve:

- `plano` = a maior corrida de colunas seguidas com a MESMA linha de orla (±0 px);
- `planos≥N` = quantas corridas têm N colunas ou mais (N = `--plano`, 12 por omissão);
- `cantos` = quantas vezes a orla muda de linha em degrau ≥ 3 px de uma coluna para a seguinte (o L, o
  rectângulo pendurado) — um fio fino (≤ 8 px) não é orla, então não conta;
- `ponta` = a maior corrida de colunas com a MESMA última linha de tinta onde ela passa da orla (a ponta
  de um fio: quadrada = a largura do fio; a do Rebelle, redonda, 1–3);
- `quadrados` = quantos fios (≥ 8 colunas — os do Rebelle têm 3–7, a passar a orla por ≥ 6 px) têm ≥ 70 % das colunas na MESMA
  última linha (±1) — a ponta cortada a direito; a redonda tem poucas;
- `paredes` = a maior corrida de LINHAS com a mesma coluna nas paredes desses degraus (o lado do
  rectângulo).

Tinta = croma (max − min do rgb) > `--croma` (o cursor cinzento da captura não tem croma). A orla da
coluna x = a linha de tinta mais baixa cuja corrida horizontal passa de 8 px (um fio não é orla), ligada à
tinta de cima sem buraco de mais de 3 linhas. As colunas varridas: `--x a:b` (por omissão a extensão da
tinta menos 8 % de cada ponta — as pontas redondas do traço não são orla).

Uso: recto.py tela.png [...] [--x a:b] [--y a:b] [--croma 12] [--plano 12] [--csv]
"""
import argparse

import numpy as np
from PIL import Image

LARGO = 8  # uma corrida horizontal até 8 px é um fio, não orla
DEGRAU = 3  # o degrau que conta como canto


def tinta_de(path, croma):
    a = np.asarray(Image.open(path).convert("RGB"), dtype=np.int32)
    return (a.max(axis=2) - a.min(axis=2)) > croma


def corridas_h(t):
    """Para cada pixel de tinta, o comprimento da corrida horizontal que o contém."""
    h, w = t.shape
    out = np.zeros((h, w), dtype=np.int32)
    for y in range(h):
        row = t[y]
        x = 0
        while x < w:
            if not row[x]:
                x += 1
                continue
            x1 = x
            while x1 + 1 < w and row[x1 + 1]:
                x1 += 1
            out[y, x:x1 + 1] = x1 - x + 1
            x = x1 + 1
    return out


def orla(t, run, x, y0, y1):
    """A orla de baixo da coluna x: desce desde a primeira tinta larga, sem buraco > 3 linhas."""
    col = t[y0:y1, x] & (run[y0:y1, x] > LARGO)
    ys = np.nonzero(col)[0]
    if ys.size == 0:
        return None
    yb, buraco = ys[0], 0
    for y in range(ys[0], y1 - y0):
        if col[y]:
            yb, buraco = y, 0
        else:
            buraco += 1
            if buraco > 3:
                break
    return y0 + yb


def mede(path, xr=None, yr=None, croma=12, n_plano=12):
    t = tinta_de(path, croma)
    h, w = t.shape
    y0, y1 = yr if yr else (0, h)
    run = corridas_h(t)
    if xr is None:
        cols = np.nonzero((t[y0:y1] & (run[y0:y1] > LARGO)).any(axis=0))[0]
        a, b = int(cols[0]), int(cols[-1])
        m = int(0.08 * (b - a))
        xr = (a + m, b - m)
    xs = range(xr[0], xr[1] + 1)
    o = [orla(t, run, x, y0, y1) for x in xs]
    planos, cantos, paredes = [], [], []
    k = 0
    while k < len(o):
        if o[k] is None:
            k += 1
            continue
        j = k
        while j + 1 < len(o) and o[j + 1] == o[k]:
            j += 1
        planos.append((j - k + 1, xr[0] + k, o[k]))
        k = j + 1
    for k in range(len(o) - 1):
        if o[k] is None or o[k + 1] is None:
            continue
        d = o[k + 1] - o[k]
        if abs(d) >= DEGRAU:
            x = xr[0] + k + (1 if d > 0 else 0)  # a coluna do lado de baixo do degrau
            lo, hi = sorted((o[k], o[k + 1]))
            # a parede: as linhas do degrau em que a coluna x é a fronteira da tinta
            lado = -1 if d > 0 else 1
            parede = sum(
                1 for y in range(lo, hi + 1)
                if t[y, x] and not t[y, x + lado]
            )
            cantos.append((xr[0] + k, d))
            paredes.append(parede)
    planos.sort(reverse=True)
    # As PONTAS dos fios: a tinta mais baixa de cada coluna onde ela passa da orla (um fio); uma ponta
    # quadrada é uma corrida de colunas com a mesma linha final (a redonda do Rebelle: 1–3).
    fundo = []
    for k, x in enumerate(xs):
        ys = np.nonzero(t[y0:y1, x])[0]
        fundo.append(y0 + ys[-1] if ys.size and o[k] is not None and y0 + ys[-1] > o[k] + DEGRAU else None)
    ponta, k = 0, 0
    while k < len(fundo):
        if fundo[k] is None:
            k += 1
            continue
        j = k
        while j + 1 < len(fundo) and fundo[j + 1] == fundo[k]:
            j += 1
        ponta = max(ponta, j - k + 1)
        k = j + 1
    # As PONTAS QUADRADAS: na tinta mais baixa de cada coluna (com fios), um fio = corrida de colunas
    # que passam a orla (mediana ±40 colunas) por ≥ 6 px; de um fio com ≥ 8 colunas, a fracção das
    # colunas que chegam à MESMA última linha (a redonda: poucas; a quadrada: quase todas).
    baixo = []
    for x in xs:
        ys = np.nonzero(t[y0:y1, x])[0]
        baixo.append(y0 + ys[-1] if ys.size else None)
    base = []
    for k in range(len(baixo)):
        jan = [v for v in o[max(0, k - 40):k + 41] if v is not None]
        base.append(sorted(jan)[len(jan) // 2] if jan else None)
    fios_q = []
    k = 0
    while k < len(baixo):
        if baixo[k] is None or base[k] is None or baixo[k] < base[k] + 6:
            k += 1
            continue
        j = k
        while j + 1 < len(baixo) and baixo[j + 1] is not None and base[j + 1] is not None \
                and baixo[j + 1] >= base[j + 1] + 6:
            j += 1
        w = j - k + 1
        if w >= 8:
            fundo_f = max(baixo[k:j + 1])
            fr = sum(1 for v in baixo[k:j + 1] if v >= fundo_f - 1) / w
            fios_q.append((xr[0] + k, w, fr))
        k = j + 1
    quadradas = sum(1 for f in fios_q if f[2] >= 0.7)
    return {
        "fios_q": fios_q,
        "quadradas": quadradas,
        "ponta": ponta,
        "x": xr,
        "plano": planos[0] if planos else (0, 0, 0),
        "planos_n": sum(1 for p in planos if p[0] >= n_plano),
        "top": planos[:5],
        "cantos": cantos,
        "parede": max(paredes, default=0),
        "orla": o,
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("telas", nargs="+")
    ap.add_argument("--x")
    ap.add_argument("--y")
    ap.add_argument("--croma", type=int, default=12)
    ap.add_argument("--plano", type=int, default=12)
    ap.add_argument("--csv", action="store_true")
    a = ap.parse_args()
    xr = tuple(int(v) for v in a.x.split(":")) if a.x else None
    yr = tuple(int(v) for v in a.y.split(":")) if a.y else None
    for p in a.telas:
        r = mede(p, xr, yr, a.croma, a.plano)
        nome = p.split("/")[-1]
        if a.csv:
            print(f"{nome},{r['plano'][0]},{r['planos_n']},{len(r['cantos'])},{r['parede']},{r['ponta']}")
            continue
        top = " ".join(f"{n}@x{x}y{y}" for n, x, y in r["top"])
        print(
            f"{nome:34} x {r['x'][0]}–{r['x'][1]} · plano {r['plano'][0]:3d} · planos≥{a.plano} "
            f"{r['planos_n']:2d} · cantos {len(r['cantos']):2d} · parede {r['parede']:3d} · ponta {r['ponta']:2d} · "
            f"fios≥8 {len(r['fios_q']):2d} quadrados {r['quadradas']:2d} · maiores: {top}"
        )
        if r["fios_q"]:
            print("    fios (x, largura, fracção na última linha ±1):",
                  " ".join(f"({x},{w},{f:.2f})" for x, w, f in r["fios_q"][:14]))
        if r["cantos"]:
            print("    cantos (x, degrau):", " ".join(f"({x},{d:+d})" for x, d in r["cantos"][:12]))


if __name__ == "__main__":
    main()
