#!/usr/bin/env python3
"""A régua da MISTURA com escorrimento (prompt 16): os quatro sintomas do smoke do dono, medidos numa foto 1:1 da
bancada `mistura_wet_drip*.{py,sh}` (tela 1000×640: vermelho y ≈ 370–430; azul E x 150–450 acima dele, azul D
x 550–850 com a orla de baixo dentro dele). As mesmas coordenadas servem a uma tela PPM/PNG da sonda do motor, se ela
usar a mesma geometria.

Luz linear = sRGB → linear (IEC 61966-2-1); lum = Rec. 709. Tinta = 255 − min(rgb) > 25.

  (a) fios: por fio (coluna com tinta abaixo do traço), a PONTA (as 4 últimas células de tinta, ±1 coluna) contra o
      CORPO (mediana do fio longe das pontas) e, onde o fio E atravessa o vermelho (y 384–418), a junção contra o
      vermelho dos LADOS do fio (5–10 px à esquerda e à direita, as mesmas linhas). Orla de baixo: o perfil vertical
      (mediana das colunas sem fio) — as 3 linhas de baixo contra o miolo; em D, também a linha MAIS ESCURA de cada
      coluna em y 360–430 (a orla do azul dentro do vermelho) contra o miolo do azul e o vermelho de baixo dele.
  (b) fronteira: perfil horizontal nas linhas do vermelho (y 395–420) através das duas fronteiras laterais do azul D;
      a fronteira = onde R − B muda de sinal; patamares = lum média a 20–40 px de cada lado; pico = máx de lum a
      ±20 px; «clareia» = pico acima dos DOIS patamares. E o vertical em D.
  (c) pontilhado: fracção das células de tinta com max_c |v − mediana 3×3| > N, N = 16 · 32 · 48 (níveis 0–255), na
      máscara inteira e no interior (máscara erodida 2 px — tira a franja de grão de papel da borda); e só as
      MAIS CLARAS que a mediana (min_c (v − med) > N: o sintoma do dono são pontos claros).
  (d) mais clara que as duas tintas: a aguada dilui com o branco do papel, logo o teste por canal contra as tintas
      puras não serve (um azul pálido tem G 210 > os 30–50 das duas). O teste é o da GAMA: em sRGB codificado (a diluição do Rebelle é
      uma fracção de branco CONSTANTE nos três canais em sRGB: o azul pálido [150,160,188] = 0,54 do caminho do
      [20,50,110] ao branco em R·G·B, 0,553 · 0,537 · 0,538; em luz linear não fecha), os vértices
      branco W, vermelho Rd, azul Bl (as cores do pincel) e o produto Rd⊙Bl (em luz linear: as duas sobrepostas) fazem um
      tetraedro; toda a mistura por diluição, sobreposição ou cobertura cai dentro dele. Uma célula de tinta fora dele
      com a coordenada de W ≥ 0 (não é «mais escura que o produto») é uma cor que NENHUMA das duas tintas dá — o
      ciano-claro. Imprime a % fora (tolerância 0,03 e 0,10 nas coordenadas) e a pior.
      `--sem-ui` tira o verde-lima das setas e o cinza do cursor das capturas do dono (o controlo).

uso: mistura_wet_drip_regua.py [--so-gama] [--sem-ui] <foto.png>…
"""
import sys

import numpy as np
from PIL import Image

TINTA = 25


def lin(a):
    a = np.asarray(a, dtype=float) / 255.0
    return np.where(a <= 0.04045, a / 12.92, ((a + 0.055) / 1.055) ** 2.4)


def lum(lc):
    return 0.2126 * lc[..., 0] + 0.7152 * lc[..., 1] + 0.0722 * lc[..., 2]


def med3(c):
    p = np.pad(c, 1, mode="edge")
    st = np.stack([p[dy:dy + c.shape[0], dx:dx + c.shape[1]] for dy in range(3) for dx in range(3)])
    return np.median(st, axis=0)


def erode(m, r):
    out = m.copy()
    for _ in range(r):
        p = np.pad(out, 1, constant_values=False)
        out = out & p[:-2, 1:-1] & p[2:, 1:-1] & p[1:-1, :-2] & p[1:-1, 2:]
    return out


def fios(d, x0, x1, ylo):
    cols = [x for x in range(x0, x1) if (d[ylo + 15:ylo + 60, x] > TINTA).sum() > 30]
    grp = []
    for x in cols:
        if grp and x - grp[-1][-1] <= 2:
            grp[-1].append(x)
        else:
            grp.append([x])
    return [g[len(g) // 2] for g in grp], set(x for g in grp for x in range(g[0] - 2, g[-1] + 3))


def rgb(v):
    return "[" + ",".join(f"{int(round(c)):3d}" for c in v) + "]"


def perfil_vertical(a, d, x0, x1, excl, y0, y1):
    xs = [x for x in range(x0, x1) if x not in excl]
    sub = a[y0:y1, xs]
    return np.median(sub, axis=1), np.median(d[y0:y1, xs], axis=1)


def srgb(lc):
    lc = np.clip(lc, 0, 1)
    return np.where(lc <= 0.0031308, lc * 12.92, 1.055 * lc ** (1 / 2.4) - 0.055)


W = np.ones(3)
RD = np.array([200, 30, 30]) / 255.0
BL = np.array([20, 50, 110]) / 255.0
PROD = srgb(lin([200, 30, 30]) * lin([20, 50, 110]))
VERT = np.stack([RD - W, BL - W, PROD - W], axis=1)


def gama(a, m, rot):
    x = a[m] / 255.0 - W
    lam = np.linalg.solve(VERT, x.T).T
    w = 1 - lam.sum(1)
    neg = np.minimum(lam.min(1), 0)
    out = {t: ((neg < -t) & (w >= -t)).mean() * 100 for t in (0.03, 0.10)}
    k = int(np.argmin(np.where(w >= -0.03, neg, 0)))
    pior = a[m][k]
    ok = w >= -0.03
    cia = ((lam[:, 0] < -0.10) & ok).mean() * 100
    print(f"  gama ({rot}, {m.sum()} células de tinta): fora e mais clara {out[0.03]:.3f} % (tol 0,03) · "
          f"{out[0.10]:.3f} % (tol 0,10) · das quais CIANO (coord. do vermelho < −0,10) {cia:.3f} % · "
          f"pior {rgb(pior)} coord (Rd, Bl, Rd⊙Bl) {np.round(lam[k], 2)} W {w[k]:.2f}")


def ui_mascara(a):
    lima = (a[..., 1] > 150) & (a[..., 0] < 190) & (a[..., 2] < 120) & (a[..., 1] - a[..., 2] > 60)
    p = np.pad(lima, 3)
    viz = np.zeros_like(lima)
    for dy in range(7):
        for dx in range(7):
            viz |= p[dy:dy + lima.shape[0], dx:dx + lima.shape[1]]
    return viz


def mede_gama(path, sem_ui):
    a = np.asarray(Image.open(path).convert("RGB")).astype(float)
    m = (255 - a.min(2)) > TINTA
    if sem_ui:
        m &= ~ui_mascara(a)
    print(f"== {path.rsplit('/', 1)[-1]}")
    gama(a, m, "tela inteira")
    meds = np.stack([med3(a[..., c]) for c in range(3)], axis=-1)
    clar = (a - meds).min(2)
    mi = erode(m, 2)
    print("  pontos mais claros que a mediana 3×3, interior: % N16 · N32 · N48 = "
          + " · ".join(f"{(clar[mi] > n).mean() * 100:.2f}" for n in (16, 32, 48)))


def mede(path):
    a = np.asarray(Image.open(path).convert("RGB")).astype(float)
    d = 255 - a.min(2)
    L = lin(a)
    print(f"== {path.rsplit('/', 1)[-1]}")
    # (a) fios E (abaixo de y 240) e D (abaixo de y 432)
    for lab, (x0, x1, ylo) in {"E": (140, 460, 240), "D": (540, 860, 432)}.items():
        xs, _ = fios(d, x0, x1, ylo)
        for xc in xs:
            col = d[ylo:, xc - 1:xc + 2].max(1)
            ys = np.where(col > TINTA)[0] + ylo
            if len(ys) == 0:
                continue
            fim = int(ys.max())
            corpo_ys = [y for y in ys if ylo + 15 <= y <= fim - 15 and not (366 <= y <= 434)]
            corpo = np.median(a[corpo_ys, xc], 0) if corpo_ys else None
            linha = f"  fio {lab} x{xc:3d} fim y{fim:3d}"
            if fim < 630:
                pts = ys[-4:]
                ponta = a[pts, xc].mean(0)
                linha += f" ponta {rgb(ponta)} lum {lum(lin(ponta)):.3f}"
            else:
                linha += " (sai da tela)"
            if corpo is not None:
                linha += f" corpo {rgb(corpo)} lum {lum(lin(corpo)):.3f}"
            if lab == "E" and fim > 418:
                j = a[384:418, xc].mean(0)
                lados = np.concatenate([a[384:418, xc - 10:xc - 4], a[384:418, xc + 5:xc + 11]], axis=1)
                ref = np.median(lados.reshape(-1, 3), 0)
                linha += f" | junção {rgb(j)} lum {lum(lin(j)):.3f} · vermelho dos lados {rgb(ref)} lum {lum(lin(ref)):.3f}"
            print(linha)
    # orla de baixo
    for lab, (x0, x1, ylo, y0, y1) in {"E": (200, 400, 240, 150, 300), "D": (600, 800, 432, 300, 450)}.items():
        _, excl = fios(d, x0 - 60, x1 + 60, ylo)
        p, pd = perfil_vertical(a, d, x0, x1, excl, y0, y1)
        ys = np.where(pd > TINTA)[0]
        if len(ys) == 0:
            continue
        t, b = int(ys[0]), int(ys[-1])
        baixo = p[b - 2:b + 1].mean(0)
        miolo = np.median(p[t + 10:b - 10], 0) if b - t > 25 else p[(t + b) // 2]
        print(f"  orla de baixo {lab}: y {y0 + t}–{y0 + b} · 3 de baixo {rgb(baixo)} lum {lum(lin(baixo)):.3f} · "
              f"miolo {rgb(miolo)} lum {lum(lin(miolo)):.3f} · escuro baixo/miolo "
              f"{(255 - baixo.min()) / max(1.0, 255 - miolo.min()):.2f}")
        if lab == "D":
            xs = [x for x in range(x0, x1) if x not in excl]
            sub = a[360:431, xs]
            sl = lum(lin(sub))
            k = sl.argmin(0)
            esc = np.median(sub[k, np.arange(len(xs))], 0)
            debaixo = np.median(a[422:428, xs].reshape(-1, 3), 0)
            print(f"  linha mais escura D (y 360–430): mediana y {360 + int(np.median(k))} {rgb(esc)} lum {lum(lin(esc)):.3f}"
                  f" · vermelho de baixo (y 422–427) {rgb(debaixo)} lum {lum(lin(debaixo)):.3f}")
    # (b) fronteira: horizontais nas linhas do vermelho
    hp = np.median(a[395:421], axis=0)
    hl = lum(lin(hp))
    rb = hp[:, 0] - hp[:, 2]
    for lab, (xa, xb) in {"esq": (500, 620), "dir": (800, 880)}.items():
        troca = [x for x in range(xa, xb) if np.sign(rb[x]) != np.sign(rb[x + 1])]
        if not troca:
            print(f"  fronteira {lab}: sem troca de R − B em x {xa}–{xb}")
            continue
        xf = troca[0]
        pa = hl[xf - 40:xf - 20].mean(); pb = hl[xf + 21:xf + 41].mean()
        k = xf - 20 + int(np.argmax(hl[xf - 20:xf + 21]))
        print(f"  fronteira {lab} x{xf}: patamares lum {pa:.3f} · {pb:.3f}, pico {hl[k]:.3f} em x{k} "
              f"{rgb(hp[k])} → acima dos dois por {hl[k] - max(pa, pb):+.3f}")
    _, excl = fios(d, 540, 860, 432)
    p, pd = perfil_vertical(a, d, 600, 800, excl, 330, 450)
    pl = lum(lin(p))
    print("  perfil vertical D (y 330–449, lum a cada 6):", " ".join(f"{v:.2f}" for v in pl[::6]))
    # (c) pontilhado
    m = d > TINTA
    meds = np.stack([med3(a[..., c]) for c in range(3)], axis=-1)
    dev = np.abs(a - meds).max(2)
    clar = (a - meds).min(2)
    regs = {"E azul só": (slice(150, 300), slice(140, 460)), "D azul/vermelho": (slice(300, 445), slice(540, 870)),
            "vermelho só": (slice(360, 440), slice(465, 526))}
    for lab, (sy, sx) in regs.items():
        mm = m[sy, sx]; mi = erode(m, 2)[sy, sx]; dv = dev[sy, sx]
        f = [f"{(dv[mm] > n).mean() * 100:5.2f}/{(dv[mi] > n).mean() * 100:5.2f}" for n in (16, 32, 48)]
        cl = [f"{(clar[sy, sx][mi] > n).mean() * 100:5.2f}" for n in (16, 32, 48)]
        print(f"  pontilhado {lab}: % N16 {f[0]} · N32 {f[1]} · N48 {f[2]} (máscara/interior; {mm.sum()} células)"
              f" · só mais claras, interior: {' · '.join(cl)}")
    gama(a, d > TINTA, "tela")


if __name__ == "__main__":
    args = sys.argv[1:]
    so_gama = "--so-gama" in args
    sem_ui = "--sem-ui" in args
    for p in [x for x in args if not x.startswith("--")]:
        mede_gama(p, sem_ui) if so_gama else mede(p)
