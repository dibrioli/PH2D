"""Régua K7 — as paredes do Knife Smooth 2 na cena `PH2D_OLEO_CENA=smooth2` de `medida_oleo_real_oraculo`.

Uso: python3 -I parede_regua.py <pasta> [<pasta> …]   (cada pasta = a saída `PH2D_OLEO_PNG` de uma corrida)

Por traço (knifeLevel 0 · 0,3 · 1 em y 90 · 200 · 310), a mediana em x 150..500 do perfil TRANSVERSAL (±40 linhas):
- altura da forma (`h_forma.f32`): células de cada parede entre 10 % e 90 % do miolo (centro ±3), largura a 50 %;
- cobertura (`cobertura.pgm`): largura > 50 % e > 5 % (a largura que se vê × o Rebelle 42 px no tamanho 40);
- o perfil da luminância do `oleo.ppm` no traço kL 1 (para pôr ao lado do Rebelle `knife_knife_smooth_2_gestos_A`).
"""
import sys
import numpy as np
from PIL import Image

GW, GH = 700, 460


def perfil(img, yc, meia=40):
    return np.median(img[yc - meia:yc + meia + 1, 150:500], axis=1)


def paredes(p, meia=40):
    topo = np.median(p[meia - 3:meia + 4])
    base = np.median(np.r_[p[:5], p[-5:]])
    f = (p - base) / (topo - base)
    dentro = lambda s: int(((s >= 0.1) & (s <= 0.9)).sum())
    w50 = np.where(f >= 0.5)[0]
    return dentro(f[:meia]), dentro(f[meia + 1:]), (int(w50.max() - w50.min() + 1) if len(w50) else 0), topo


for d in sys.argv[1:]:
    h = np.fromfile(f"{d}/h_forma.f32", dtype="<f4").reshape(GH, GW)
    c = np.asarray(Image.open(f"{d}/cobertura.pgm")).astype(float)
    a = np.asarray(Image.open(f"{d}/oleo.ppm").convert("RGB")).astype(float)
    lum = 0.2126 * a[..., 0] + 0.7152 * a[..., 1] + 0.0722 * a[..., 2]
    for kl, y in ((0, 90), (0.3, 200), (1, 310)):
        cima, baixo, w50, topo = paredes(perfil(h, y))
        q = perfil(c, y)
        print(f"{d}: kL {kl:<3} paredes {cima}/{baixo} cél · largura@50% {w50} · miolo {topo:.2f} px"
              f" · cobertura >50% {int((q > 127).sum())} >5% {int((q > 12).sum())}")
    print(f"{d}: luminância kL 1", np.round(np.median(lum[290:336, 150:500], axis=1)).astype(int).tolist())
