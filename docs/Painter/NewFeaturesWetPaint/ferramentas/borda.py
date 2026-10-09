"""A régua da borda do S (W1c): a mesma conta sobre o oráculo (PNG, `fixtures/rebelle_gotas/s_*.png`) e o
nosso (PPM das sondas `borda_do_s`/`sonda_lei_das_gotas`). Portada para o portão em
`crates/ph2d-tool-painter/tests/it/borda_do_s.rs` (`mede_borda`, `fios_do_s`).

Tudo em proporção a B0 = o escuro do corpo do MESMO motor com a tela direita (o Rebelle ~102, o nosso
Wet Round ~213): os dois pigmentam diferente, a forma da borda é o que se compara.

uso: python3 -I borda.py B0:img [B0:img ...]
"""
import math
import sys

import numpy as np
from PIL import Image

X0, X1, YC, AMP = 100.0, 900.0, 180.0, 85.0
T0, T1 = -math.pi / 2, 2 * math.pi + math.pi / 4


def yc(x):
    t = T0 + (T1 - T0) * (x - X0) / (X1 - X0)
    return YC - AMP * math.sin(t)


def escuro(path):
    a = np.asarray(Image.open(path).convert("RGB"), dtype=np.int32)
    return 255 - a.min(axis=2)


def run_h(d, x, y, t):
    if d[y, x] <= t:
        return 0
    l = x
    while l > 0 and d[y, l - 1] > t:
        l -= 1
    r = x
    while r < d.shape[1] - 1 and d[y, r + 1] > t:
        r += 1
    return r - l + 1


def orla(d, x, t):
    """A linha de tinta mais baixa (corrida horizontal > 8: não um fio) a até 75 px abaixo do eixo, sem um
    buraco de mais de 3 linhas desde o eixo."""
    c = int(round(yc(x)))
    yb, buraco = c, 0
    for y in range(c, min(c + 75, d.shape[0])):
        if d[y, x] > t and run_h(d, x, y, t) > 8:
            yb, buraco = y, 0
        else:
            buraco += 1
            if buraco > 3:
                break
    return yb


def coluna(d, x, b0):
    t = 0.12 * b0
    yb = orla(d, x, t)
    c = int(round(yc(x)))
    corpo = float(np.median(d[c - 10 : c + 11, x]))
    pico = float(d[yb - 4 : yb + 1, x].max())
    prof = d[c - 10 : yb + 1, x].astype(float)
    k = np.convolve(prof, np.ones(3) / 3, mode="same") if len(prof) >= 3 else prof
    amp = pico - corpo
    rampa = None
    if amp >= 0.15 * b0:
        n90, n10 = corpo + 0.9 * amp, corpo + 0.1 * amp
        i = len(k) - 2
        while i > 0 and k[i] >= n90:
            i -= 1
        j = i
        while j > 0 and k[j] >= n10:
            j -= 1
        rampa = i - j
    # a faixa chapada: linhas contíguas desde a orla com escuro ≥ B0
    faixa, i = 0, yb
    while i > c - 10 and d[i, x] >= b0:
        faixa += 1
        i -= 1
    # degrau: um salto ≥ 0,4·B0 em 2 px ACIMA da linha da orla (fora dos 4 px dela)
    z = d[c - 10 : max(c - 9, yb - 4), x].astype(float)
    degrau = len(z) > 2 and np.max(z[2:] - z[:-2]) >= 0.4 * b0
    return yb, corpo, pico, rampa, faixa, degrau


def mede(path, b0, xs=range(440, 633)):
    d = escuro(path)
    cols = [coluna(d, x, b0) for x in xs]
    yb = np.array([c[0] for c in cols], dtype=float)
    xx = np.array(list(xs), dtype=float)
    res = yb - np.polyval(np.polyfit(xx, yb, 3), xx)
    rampas = [c[3] for c in cols if c[3] is not None]
    vale = coluna(d, 682, b0)
    pat, corrida = 0, 1
    for u, v in zip(yb, yb[1:]):
        if u == v:
            corrida += 1
        else:
            pat += corrida >= 6
            corrida = 1
    return {
        "patamares": pat,
        "corpo/B0": float(np.median([c[1] for c in cols])) / b0,
        "pico/B0": float(np.median([c[2] for c in cols])) / b0,
        "rampa": float(np.median(rampas)) if rampas else None,
        "faixa>=B0": float(np.median([c[4] for c in cols])),
        "degraus%": 100.0 * sum(c[5] for c in cols) / len(cols),
        "salto": float(np.abs(np.diff(yb)).max()),
        "rms": math.sqrt(float((res**2).mean())),
        "vale": (round(vale[1] / b0, 2), round(vale[2] / b0, 2), vale[3], vale[4]),
    }


def fios(path, b0, y=330):
    """Os fios que cruzam a linha y: quantos, e a largura de cada um a meia altura (FWHM, px)."""
    d = escuro(path).astype(float)
    t = 0.12 * b0
    lin = d[y]
    out, x = [], 0
    while x < len(lin):
        if lin[x] > t:
            a = x
            while x < len(lin) and lin[x] > t:
                x += 1
            seg = lin[a:x]
            out.append(int((seg >= seg.max() / 2).sum()))
        x += 1
    return out


if __name__ == "__main__":
    for a in sys.argv[1:]:
        b0, p = a.split(":", 1)
        m = mede(p, float(b0))
        m["fios@330"] = fios(p, float(b0))
        print(f"{p.rsplit('/', 1)[-1]:<14}", {k: (round(v, 2) if isinstance(v, float) else v) for k, v in m.items()})


def comprimentos(path, b0, y=330):
    """Para cada fio que cruza y: onde ele acaba (a última linha com tinta na coluna do pico)."""
    d = escuro(path).astype(float)
    t = 0.12 * b0
    lin = d[y]
    out, x = [], 0
    while x < len(lin):
        if lin[x] > t:
            a = x
            while x < len(lin) and lin[x] > t:
                x += 1
            c = a + int(np.argmax(lin[a:x]))
            k = y
            while k + 1 < d.shape[0] and d[k + 1, max(0, c - 2) : c + 3].max() > t:
                k += 1
            out.append((c, k))
        x += 1
    return out


def funis(path, b0, y=330):
    """Para cada fio: escuro na raiz (máx 2..12 px abaixo da orla, ±2 colunas) ÷ mediana do fio (y..y+50)."""
    d = escuro(path).astype(float)
    t = 0.12 * b0
    out = []
    for c, _ in comprimentos(path, b0, y):
        yb = orla(d, c, t) if abs(yc(c) - 0) >= 0 else 0
        raiz = d[yb + 2 : yb + 13, c - 2 : c + 3].max()
        fio = float(np.median(d[y : y + 50, c - 1 : c + 2].max(axis=1)))
        out.append(round(raiz / max(fio, 1.0), 2))
    return out


def brechas_encosta(path, b0, xs=range(440, 633)):
    """Colunas da descida cuja linha da orla (as 4 linhas de cima da orla) tem um pixel claro (< 0,5·B0)."""
    d = escuro(path).astype(float)
    t = 0.12 * b0
    n = 0
    for x in xs:
        yb = orla(d, x, t)
        if d[yb - 3 : yb + 1, x].min() < 0.5 * b0:
            n += 1
    return n
