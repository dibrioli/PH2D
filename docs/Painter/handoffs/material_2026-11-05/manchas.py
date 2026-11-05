#!/usr/bin/env python3
"""Régua das manchas do miolo (Bug #62): manchas.py <pasta com os .ppm de sonda_manchas_liner> [saida.png] [filtro].
s = |fundo − pixel| (soma RGB). miolo = distância à curva < 0,5·raio. Tira o ramp lento (caixa de 41 px) para só contar a mancha.
cv9 = CV a 9 px · cv3 = CV a 3 px · esc% = área do miolo acima de 1,15× o local · claro% = abaixo de 0,85×."""
import glob, os, sys
import numpy as np
from PIL import Image, ImageDraw, ImageFont

def caixa(a, n):
    p = np.pad(a, n // 2, mode="edge")
    c = np.cumsum(np.cumsum(np.pad(p, ((1, 0), (1, 0))), 0), 1)
    return (c[n:, n:] - c[:-n, n:] - c[n:, :-n] + c[:-n, :-n]) / (n * n)

def curva(n=600):
    u = np.linspace(0, 1, n)[:, None]
    p = [np.array(q, float) for q in ([70, 250], [190, 20], [330, 340], [450, 110])]
    return (1 - u) ** 3 * p[0] + 3 * (1 - u) ** 2 * u * p[1] + 3 * (1 - u) * u * u * p[2] + u ** 3 * p[3]

def distancia(h, w):
    yy, xx = np.mgrid[:h, :w]
    c = curva()
    d = np.full((h, w), 1e9)
    for cx, cy in c[::4]:
        d = np.minimum(d, (xx - cx) ** 2 + (yy - cy) ** 2)
    return np.sqrt(d)

def mede(arq, raio=33):
    a = np.asarray(Image.open(arq).convert("RGB"), float)
    h, w, _ = a.shape
    d = distancia(h, w)
    fundo = np.median(a[d > raio + 25], axis=0)
    s = np.abs(fundo - a).sum(2)
    m = d < 0.5 * raio
    # cortar as pontas da curva
    c = curva()
    pontas = np.minimum(np.hypot(*(np.mgrid[:h, :w][::-1] - c[0][:, None, None])), np.hypot(*(np.mgrid[:h, :w][::-1] - c[-1][:, None, None])))
    m &= pontas > 1.5 * raio
    ref = caixa(s, 41)
    r = s / np.maximum(ref, 1e-6)
    r3, r9 = caixa(r, 3), caixa(r, 9)
    return dict(tinta=round(s[m].mean(), 1), cv3=round(r3[m].std() / r3[m].mean(), 3), cv9=round(r9[m].std() / r9[m].mean(), 3),
                esc=round(100 * (r9[m] > 1.15).mean(), 2), claro=round(100 * (r9[m] < 0.85).mean(), 2)), r9

if __name__ == "__main__":
    pasta = sys.argv[1]
    filtro = sys.argv[3] if len(sys.argv) > 3 else ""
    arqs = sorted(f for f in glob.glob(pasta + "/*.ppm") if filtro in f)
    print("imagem".ljust(58), "tinta cv3 cv9 esc% claro%")
    for f in arqs:
        r, _ = mede(f)
        print(os.path.basename(f)[:-4].ljust(58), *r.values())
    # Ao longo do eixo: tira o perfil através do traço. cvEixo = CV de s a 9 px sobre a linha central (sem as pontas), em %.
    print("\nimagem".ljust(59), "cvEixo% min/mediana max/mediana")
    c = curva(2000)[300:1700]
    for f in arqs:
        a = np.asarray(Image.open(f).convert("RGB"), float)
        fundo = np.median(a[:12, :12], axis=(0, 1))
        s = caixa(np.abs(fundo - a).sum(2), 9)
        v = s[c[:, 1].round().astype(int), c[:, 0].round().astype(int)]
        print(os.path.basename(f)[:-4].ljust(58), round(100 * v.std() / v.mean(), 2), round(v.min() / np.median(v), 3), round(v.max() / np.median(v), 3))
