#!/usr/bin/env python3
"""A régua do Over Oil (J9, 30/10): a aguada amarela por cima do óleo azul, nosso × Rebelle, POR PASSO.

uso: over_oil_regua.py <nome> <óleo só> <a meio> <assentada> [<nome> <óleo> <meio> <fim> …]   (PNG/PPM 240×160)

Máscaras (de cada cena, nunca de coordenadas fixas): o ÓLEO = píxeis AZUIS da foto «óleo só» (B − R > 60, B > 120); as COLUNAS da aguada = colunas onde a faixa y 35..60 (papel, acima do óleo) ganhou amarelo
(B − R da média < −20 contra o papel). O CRUZAMENTO = óleo ∩ colunas, sem as 3 colunas de cada lado.
Medidas no cruzamento: cor média · desvio da luminância (os veios: o relevo do óleo a ver-se) · `amarelo` = o quanto
a cor andou do óleo para a aguada (0 = só óleo, 1 = só aguada; projecção em RGB) · `escuros` = píxeis com a
luminância abaixo do percentil 5 do óleo só (mais escuros que as duas tintas), na banda de ±4 px da borda do óleo
dentro das colunas. Borda: `transição` = em cada coluna, quantos píxeis vão de 10 % a 90 % do salto de G do papel
amarelo para o cruzamento (média), e `patamar` = o comprimento médio dos patamares da linha onde a aguada-sobre-papel
acaba (a ESCADA de células: plano por toda a célula; uma orla natural muda quase a cada coluna).
"""
import sys

import numpy as np
from PIL import Image


def le(f):
    return np.asarray(Image.open(f).convert("RGB")).astype(float)


def lum(a):
    return a @ np.array([0.2126, 0.7152, 0.0722])


def patamar(ys):
    """O comprimento médio dos patamares (colunas seguidas com a mesma linha de borda): a ESCADA. Uma orla natural
    muda de linha quase a cada coluna (~1–2); a escada de células fica plana por toda a célula."""
    if len(ys) < 2:
        return float("nan")
    runs, n = [], 1
    for a, b in zip(ys, ys[1:]):
        if a == b:
            n += 1
        else:
            runs.append(n)
            n = 1
    runs.append(n)
    return float(np.mean(runs))


def mede(oleo, meio, fim):
    o, out = le(oleo), {}
    papel = np.median(o[2:12, 2:12].reshape(-1, 3), axis=0)
    mo = (o[..., 2] - o[..., 0] > 60) & (o[..., 2] > 120)  # o AZUL (a textura do papel do Rebelle passa os 40)
    lo5 = np.percentile(lum(o)[mo], 5)
    oleo_cor = o[mo].mean(0)
    for passo, f in (("meio", meio), ("fim", fim)):
        a = le(f)
        topo = a[35:60].mean(0)
        cols = np.where((topo[:, 2] - topo[:, 0]) - (papel[2] - papel[0]) < -20)[0]
        if len(cols) < 8:
            out[passo] = None
            continue
        c0, c1 = cols.min() + 3, cols.max() - 3
        cruz = mo.copy()
        cruz[:, :c0] = False
        cruz[:, c1 + 1:] = False
        px = a[cruz]
        agua = topo[c0:c1 + 1].mean(0)
        d = agua - oleo_cor
        amarelo = float(((px.mean(0) - oleo_cor) @ d) / (d @ d))
        # a borda de cima do óleo em cada coluna
        meios, trans, esc = [], [], 0
        for x in range(c0, c1 + 1):
            ys = np.where(mo[:, x])[0]
            if len(ys) == 0:
                continue
            y0 = ys.min()
            g = a[max(0, y0 - 12):y0 + 12, x, 1]
            hi, lo = a[max(0, y0 - 14):y0 - 6, x, 1].mean(), a[y0 + 4:y0 + 12, x, 1].mean()
            if abs(hi - lo) < 10:
                continue
            f_ = (g - lo) / (hi - lo)
            dentro = np.where((f_ > 0.1) & (f_ < 0.9))[0]
            trans.append(len(dentro))
            # onde a aguada-sobre-papel acaba, descendo a coluna (a borda do que a aguada fez no óleo)
            pa = a[max(0, y0 - 14):y0 + 12, x]
            ok = (pa[:, 2] < pa[:, 0] - 15) & (lum(pa) > 0.8 * lum(agua))
            fora = np.where(~ok)[0]
            meios.append(max(0, y0 - 14) + (int(fora[0]) if len(fora) else len(pa)))
            for yb in (y0, ys.max()):
                faixa = a[max(0, yb - 4):yb + 5, x]
                esc += int((lum(faixa) < lo5).sum())
        out[passo] = dict(cor=px.mean(0).round().astype(int).tolist(), desvio=float(lum(px).std()),
                          amarelo=amarelo, transicao=float(np.mean(trans)) if trans else float("nan"),
                          degrau=patamar(meios), escuros=esc, n=int(cruz.sum()),
                          colunas=int(c1 - c0 + 1))
    return dict(oleo=oleo_cor.round().astype(int).tolist(), desvio_oleo=float(lum(o)[mo].std()), passos=out)


def main():
    v = sys.argv[1:]
    for k in range(0, len(v), 4):
        nome, r = v[k], mede(*v[k + 1:k + 4])
        print(f"## {nome}: óleo só {r['oleo']} desvio {r['desvio_oleo']:.1f}")
        for p, m in r["passos"].items():
            if m is None:
                print(f"   {p}: sem colunas da aguada")
                continue
            print(f"   {p:<4} cor {m['cor']} desvio {m['desvio']:5.1f} amarelo {m['amarelo']:.2f} · borda: transição "
                  f"{m['transicao']:.1f} px patamar {m['degrau']:.1f} · escuros {m['escuros']} · {m['n']} px, {m['colunas']} col")


if __name__ == "__main__":
    main()
