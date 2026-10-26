# Régua da folha das espátulas (tracos_facas.py, sessão F42): por espátula, a largura do traço, a
# cobertura, a fenda (2 % mais escuros) e a mediana por canal, e a sombra no papel por baixo da borda.
# Ecrã: 1 px de tela = 1,167 px de ecrã (README M6). uso: python3 -I regua_facas.py out/F42_base.png
import sys
import numpy as np
from PIL import Image

FACAS = ["Knife Smooth", "Knife Smooth 2", "Knife Smooth 3", "Knife Textured", "Knife Oily",
         "Knife Dry", "Knife Dry 2", "Knife Grainy", "Knife Soft", "Knife Stain"]
ESCALA = 1.167

a = np.asarray(Image.open(sys.argv[1]).convert("RGB")).astype(int)
r, g, b = a[..., 0], a[..., 1], a[..., 2]
tinta = (b > 60) & (b - r > 40)  # inclui a fenda escura (M1: (0,43,113))
papel = (r > 200) & (g > 200) & (b > 200)
lum = a @ np.array([0.2126, 0.7152, 0.0722])

# a folha: o retângulo de papel (colunas/linhas com muito papel)
cols = np.where(papel.mean(axis=0) > 0.3)[0]
rows = np.where(papel.mean(axis=1) > 0.3)[0]
x0, x1, y0, y1 = cols.min(), cols.max(), rows.min(), rows.max()
meio = (x0 + x1) // 2
print(f"folha ecrã x {x0}..{x1} y {y0}..{y1}  ({(x1 - x0) / ESCALA:.0f}×{(y1 - y0) / ESCALA:.0f} px de tela)")
print("| espátula | largura (px tela) | cobertura % | fenda RGB | mediana RGB | razão G,B | sombra por baixo /5px |")
print("|---|---|---|---|---|---|---|")
for i, nome in enumerate(FACAS):
    cx0, cx1 = (x0, meio) if i < 5 else (meio, x1)
    yc = y0 + (90 + 150 * (i % 5)) * ESCALA
    ya, yb = int(yc - 75 * ESCALA), int(yc + 75 * ESCALA)
    m = tinta[ya:yb, cx0:cx1]
    linhas = np.where(m.mean(axis=1) > 0.15)[0]
    if not len(linhas):
        print(f"| {nome} | — | 0 | | | | |")
        continue
    t0, t1 = ya + linhas.min(), ya + linhas.max()
    xs = np.where(m[linhas.min():linhas.max() + 1].mean(axis=0) > 0.3)[0]
    xa, xb = cx0 + xs.min() + 20, cx0 + xs.max() - 20  # sem as pontas
    caixa = a[t0:t1 + 1, xa:xb]
    mt = tinta[t0:t1 + 1, xa:xb]
    px = caixa[mt]
    l = px @ np.array([0.2126, 0.7152, 0.0722])
    fenda = px[l <= np.percentile(l, 2)].mean(axis=0)
    med = np.median(px, axis=0)
    rz = fenda / np.maximum(med, 1)
    # sombra no papel por baixo do traço: perfil por coluna a partir da última linha de tinta
    perfis = []
    for x in range(xa, xb, 3):
        col = tinta[t0:t1 + 25, x]
        e = t0 + np.where(col)[0].max() if col.any() else None
        if e is None or e + 60 >= a.shape[0]:
            continue
        ref = np.median(lum[e + 45:e + 60, x])
        perfis.append([ref - lum[e + d, x] for d in range(1, 31)])
    p = np.median(np.array(perfis), axis=0) if perfis else np.zeros(30)
    sombra = [round(float(p[k:k + 5].mean()), 1) for k in range(0, 30, 5)]
    print(f"| {nome} | {(t1 - t0 + 1) / ESCALA:.0f} | {100 * mt.mean():.0f} | {tuple(int(v) for v in fenda)} | "
          f"{tuple(int(v) for v in med)} | {rz[1]:.2f}, {rz[2]:.2f} | {sombra} |")
