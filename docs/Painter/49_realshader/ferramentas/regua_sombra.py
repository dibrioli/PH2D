# Régua M6/M10: escurecimento do PAPEL à direita da espátula vertical (tracos.py: Knife Smooth azul,
# x=700, y 60..420), em passos de 5 px de ecrã a partir da borda, contra o papel a +100..+120 px.
# Acha a espátula pela cor (funciona no crop de capturas/*.jpg e na janela inteira de out/*.png).
# uso: python3 -I regua_sombra.py foto [foto ...]
import sys
import numpy as np
from PIL import Image


def azul(a):
    r, g, b = a[..., 0].astype(int), a[..., 1].astype(int), a[..., 2].astype(int)
    return (b > 150) & (b - r > 80)


def mede(caminho):
    a = np.asarray(Image.open(caminho).convert("RGB"))
    m = azul(a)
    # só a espátula vertical tem uma faixa contínua de azul com > 300 px de altura
    corrida = np.zeros(a.shape[1], int)
    atual = np.zeros(a.shape[1], int)
    for linha in m:
        atual = np.where(linha, atual + 1, 0)
        corrida = np.maximum(corrida, atual)
    xs = np.where(corrida > 300)[0]
    if not len(xs):
        return None
    x0, x1 = xs.min(), xs.max()
    linhas = np.where(m[:, x0:x1 + 1].mean(axis=1) > 0.5)[0]
    y0, y1 = linhas.min(), linhas.max()
    meio = range(y0 + (y1 - y0) // 4, y1 - (y1 - y0) // 4)
    lum = a.astype(float) @ np.array([0.2126, 0.7152, 0.0722])
    perfis = []
    for y in meio:
        bordas = np.where(m[y, x0:x1 + 40])[0]
        if not len(bordas):
            continue
        e = x0 + bordas.max()
        ref = np.median(lum[y, e + 100:e + 121])
        perfis.append([ref - lum[y, e + d] for d in range(1, 61)])
    p = np.median(np.array(perfis), axis=0)
    passos = [round(float(p[i:i + 5].mean()), 1) for i in range(0, 60, 5)]
    return x1 - x0 + 1, len(perfis), passos


for f in sys.argv[1:]:
    r = mede(f)
    print(f.split("/")[-1], "sem espátula vertical" if r is None else f"largura={r[0]}px linhas={r[1]} sombra/5px={r[2]}")
