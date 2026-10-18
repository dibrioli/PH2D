#!/usr/bin/env python3
"""A régua da ESCADA (prompt 50; doc 50_medicao): num traço de fios, quanto a imagem salta AO LONGO dos fios a 1 px.

Um fio liso (anti-aliased) é constante ao longo de si próprio; uma fronteira de fio que é um degrau por píxel
(a escada) não — num traço inclinado cada degrau corta o fio. A régua mede no miolo do traço (`c > 0,08`
erodido 6 px; `c = max|px − papel| / 255`, como `mede_traco.py`), na direcção LOCAL dos fios (o tensor de
estrutura: gradiente de `c` suavizado a σ 1,5, tensor a σ 6), com a imagem interpolada bilinear:

tudo na LUMINÂNCIA `Y` (`luma`, `0..255`):

- `escada` = rms(ΔY ao longo dos fios, 1 px) — o degrau que corta o fio; o controlo é o MESMO pincel no recto
  horizontal (onde os fios caem nas linhas de píxeis e nenhum degrau se vê) e o Rebelle na mesma cena;
- `através` = rms(ΔY através, 1 px);
- `contraste` = rms(Y − G₄Y): a textura fina que se vê;
- `fino` = rms(Y − G₁Y) ÷ rms(G₁Y − G₄Y): a energia a 1 px sobre a de 1–4 px;
- `largura` = o atraso (px, através) em que a autocorrelação de `Y − G₄Y` cai a ½ — a largura do fio.

uso: escada.py <imagem> <região> [<região> …]   — região = nome:x0,y0,x1,y1 (o miolo dentro dela)
     escada.py --tabela <dir>                     — os PPM da sonda `sonda_a_escada` em <dir> (e subpastas), as
                                                    fotos do Rebelle (`fixtures/rebelle/{oil,oil40c,oil103,incl40,incl103}`)
                                                    e a foto do dono (49_smoke_dono_pixelado.png)
"""
import sys
from pathlib import Path

import numpy as np
from PIL import Image


def carrega(p):
    p = Path(p)
    if p.suffix == ".ppm":
        b = p.read_bytes()
        cab = b.split(b"\n", 3)
        w, h = map(int, cab[1].split())
        return np.frombuffer(cab[3], np.uint8)[: w * h * 3].reshape(h, w, 3).astype(float)
    return np.asarray(Image.open(p).convert("RGB")).astype(float)


def gauss(a, s):
    r = int(3 * s + 1)
    k = np.exp(-0.5 * (np.arange(-r, r + 1) / s) ** 2)
    k /= k.sum()
    p = np.pad(a, r, mode="edge")
    p = np.apply_along_axis(lambda v: np.convolve(v, k, "valid"), 0, p)
    return np.apply_along_axis(lambda v: np.convolve(v, k, "valid"), 1, p)


def bilinear(a, x, y):
    h, w = a.shape
    x = np.clip(x, 0, w - 1.001)
    y = np.clip(y, 0, h - 1.001)
    x0, y0 = np.floor(x).astype(int), np.floor(y).astype(int)
    fx, fy = x - x0, y - y0
    return (a[y0, x0] * (1 - fx) * (1 - fy) + a[y0, x0 + 1] * fx * (1 - fy)
            + a[y0 + 1, x0] * (1 - fx) * fy + a[y0 + 1, x0 + 1] * fx * fy)


def cobertura(rgb):
    borda = np.concatenate([rgb[:2].reshape(-1, 3), rgb[-2:].reshape(-1, 3)])
    papel = np.median(borda, axis=0)
    return np.abs(rgb - papel).max(axis=2) / 255.0


def luma(rgb):
    """A luminância sRGB (Rec. 709, `0..255`): o que o olho vê. ⚠️ A `cobertura` (o canal que mais se afasta do
    papel) SATURA na tinta escura — o azul da bancada já tem R = 23, e uma sombra preta só a move de 0,91 a 1,0 —, por
    isso a régua da textura de 18/10 leu o relevo a 5,87 contra 5,86 com 3× o contraste do Rebelle (50_medicao §1)."""
    return 0.2126 * rgb[..., 0] + 0.7152 * rgb[..., 1] + 0.0722 * rgb[..., 2]


def erode(m, n):
    for _ in range(n):
        m = m & np.roll(m, 1, 0) & np.roll(m, -1, 0) & np.roll(m, 1, 1) & np.roll(m, -1, 1)
    return m


def mede(rgb, regiao):
    x0, y0, x1, y1 = regiao
    c = cobertura(rgb)
    miolo = erode(c > 0.08, 6)
    dentro = np.zeros_like(miolo)
    dentro[y0:y1, x0:x1] = True
    miolo &= dentro
    g = gauss(c, 1.5)
    gx = np.zeros_like(g)
    gy = np.zeros_like(g)
    gx[:, 1:-1] = 0.5 * (g[:, 2:] - g[:, :-2])
    gy[1:-1] = 0.5 * (g[2:] - g[:-2])
    jxx, jxy, jyy = gauss(gx * gx, 6), gauss(gx * gy, 6), gauss(gy * gy, 6)
    ang = 0.5 * np.arctan2(2 * jxy, jxx - jyy)  # a direcção do gradiente: a NORMAL dos fios
    ys, xs = np.nonzero(miolo)
    if len(xs) < 200:
        return None
    a = ang[ys, xs]
    nx, ny = np.cos(a), np.sin(a)
    tx, ty = -ny, nx
    xf, yf = xs.astype(float), ys.astype(float)
    lum = luma(rgb)
    d = lambda ux, uy: bilinear(lum, xf + 0.5 * ux, yf + 0.5 * uy) - bilinear(lum, xf - 0.5 * ux, yf - 0.5 * uy)
    rms = lambda v: float(np.sqrt(np.mean(v * v)))
    dt, dn = d(tx, ty), d(nx, ny)
    g1, g4 = gauss(lum, 1.0), gauss(lum, 4.0)
    res = lum - g4
    # a autocorrelação através: o resíduo amostrado a k px na normal
    r0 = res[ys, xs]
    ac = [1.0]
    for k in range(1, 9):
        rk = bilinear(res, xf + k * nx, yf + k * ny)
        ac.append(float(np.mean(r0 * rk) / max(np.mean(r0 * r0), 1e-12)))
    largura = next((k - 1 + (ac[k - 1] - 0.5) / (ac[k - 1] - ac[k]) for k in range(1, 9) if ac[k] < 0.5), 8.0)
    # o ângulo dos fios (graus, 0 = horizontal) — a mediana, para conferir contra o gesto
    fio = np.degrees(np.arctan2(ty, tx))
    fio = np.where(fio > 90, fio - 180, np.where(fio < -90, fio + 180, fio))
    return {
        "escada": rms(dt),
        "atraves": rms(dn),
        "fino": rms((lum - g1)[ys, xs]) / max(rms((g1 - g4)[ys, xs]), 1e-12),
        "largura": largura,
        "contraste": rms(r0),
        "angulo": float(np.median(np.abs(fio))),
        "n": len(xs),
    }


# As regiões das cenas (tela 1000×640): a bancada (o recto da linha 1 = o CONTROLO horizontal; a curva em S) e
# os inclinados (8° e 25°, o meio de cada um).
BANCADA = {"recto": (110, 40, 510, 220), "curva": (620, 20, 940, 230)}
INCLINADOS = {"8°": (250, 100, 750, 330), "25°": (180, 300, 480, 560)}
# A foto do dono (49_smoke_dono_pixelado.png, ~103 px, ~8°): o traço de cima, à esquerda do cursor.
DONO = {"dono 8°": (80, 60, 440, 300)}


def linha(nome, regs, rgb):
    out = []
    for rn, r in regs.items():
        m = mede(rgb, r)
        if m is None:
            out.append(f"{nome:<28} {rn:<8} (miolo vazio)")
            continue
        out.append(f"{nome:<34} {rn:<8} escada {m['escada']:5.2f} · através {m['atraves']:5.2f} · contraste {m['contraste']:5.2f}"
                   f" · fino {m['fino']:.3f} · largura {m['largura']:.2f} px · fios a {m['angulo']:4.1f}° · n {m['n']}")
    return out


def tabela(d):
    d = Path(d)
    raiz = Path(__file__).resolve().parent.parent
    for f in sorted(d.glob("**/escada_*.ppm")):
        regs = INCLINADOS if "_incl_" in f.name else BANCADA
        print("\n".join(linha(f.stem, regs, carrega(f))))
    for nome, regs in [("oil.png", BANCADA), ("oil40c.png", BANCADA), ("oil103.png", BANCADA),
                       ("incl40.png", INCLINADOS), ("incl103.png", INCLINADOS)]:
        p = raiz / "fixtures" / "rebelle" / nome
        print("\n".join(linha("REBELLE " + nome, regs, carrega(p))))
    dono = raiz / "49_smoke_dono_pixelado.png"
    print("\n".join(linha("DONO 18/10 (o relevo)", DONO, carrega(dono))))


def main():
    if sys.argv[1] == "--tabela":
        tabela(sys.argv[2])
        return
    rgb = carrega(sys.argv[1])
    regs = {}
    for r in sys.argv[2:]:
        n, v = r.split(":")
        regs[n] = tuple(map(int, v.split(",")))
    print("\n".join(linha(Path(sys.argv[1]).stem, regs, rgb)))


if __name__ == "__main__":
    main()
