#!/usr/bin/env python3
"""Régua da CURVA das setas do Quadro contra capturas do Miro que o dono mandou (06/10).

O Miro não corre por script (conta, web): as capturas são a fixture, e esta régua ajusta a nossa
família de curvas aos píxeis do traço. Reimprime os números que estão no `ph2d-board-route`:

- `curva_duas_caixas.png`: uma cúbica entre duas caixas, tangentes horizontais ⇒ o braço da ponta
  (`END_ARM`, fracção da distância entre as pontas).
- `curva_com_pontos.png`: seis estações (círculos ocos) e cinco meios de trecho (bolinhas cheias) ⇒
  com braços livres por trecho (as direcções do Catmull-Rom) e a melhor lei global para o braço num
  ponto (`POINT_ARM`, fracção do próprio trecho).

uso: python3 -I mede_curva_miro.py
"""
import math
import os
import sys

from PIL import Image

AQUI = os.path.join(os.path.dirname(os.path.abspath(__file__)), "capturas_miro")


def bez(s, t):
    a, c1, c2, b = s
    u = 1 - t
    return tuple(u**3 * a[k] + 3 * u * u * t * c1[k] + 3 * u * t * t * c2[k] + t**3 * b[k] for k in (0, 1))


def unit(v):
    n = math.hypot(*v)
    return (v[0] / n, v[1] / n)


def rms(pts, poly):
    e = [min(math.dist(p, q) for q in poly) for p in pts]
    return (sum(v * v for v in e) / len(e)) ** 0.5


def duas_caixas():
    px = Image.open(os.path.join(AQUI, "curva_duas_caixas.png")).convert("L").load()
    pts = [(x, y) for x in range(440, 700, 2) for y in range(150, 400) if px[x, y] < 100]
    p0, p1 = (715.0, 201.0), (408.0, 353.0)
    best = None
    for k in range(100, 220, 2):
        s = (p0, (p0[0] - k, p0[1]), (p1[0] + k, p1[1]), p1)
        e = rms(pts, [bez(s, i / 300) for i in range(301)])
        if best is None or e < best[0]:
            best = (e, k)
    d = math.dist(p0, p1)
    print(f"duas caixas: braço {best[1]} (rms {best[0]:.2f} px) · distância {d:.1f} ⇒ END_ARM ≈ {best[1] / d:.3f}")


def com_pontos():
    px = Image.open(os.path.join(AQUI, "curva_com_pontos.png")).convert("RGB").load()
    est = [(399.6, 138.0), (330.2, 137.9), (295.1, 223.5), (211.9, 119.5), (212.0, 239.3), (95.2, 290.8)]
    meios = [(361.6, 130.8), (317.8, 188.4), (254.6, 167.0), (198.3, 167.7), (174.4, 282.0)]
    marcas = est + meios
    traco = [
        (x, y)
        for y in range(90, 320)
        for x in range(85, 410)
        if (lambda r, g, b: b > 110 and b - r > 50 and r + g + b < 600)(*px[x, y])
        and all(math.dist((x, y), m) > 9 for m in marcas)
    ]
    n = len(est)
    ida, chegada = (-1.0, 0.0), (-1.0, 0.0)
    corda = [math.dist(est[i], est[i + 1]) for i in range(n - 1)]

    def tan(i):
        if i == 0:
            return ida
        if i == n - 1:
            return chegada
        return unit((est[i + 1][0] - est[i - 1][0], est[i + 1][1] - est[i - 1][1]))

    def curva(a, e):
        segs = []
        for i in range(n - 1):
            p, q = est[i], est[i + 1]
            la = (e if i == 0 else a) * corda[i]
            lb = (e if i + 1 == n - 1 else a) * corda[i]
            ta, tb = tan(i), tan(i + 1)
            segs.append((p, (p[0] + ta[0] * la, p[1] + ta[1] * la), (q[0] - tb[0] * lb, q[1] - tb[1] * lb), q))
        return [bez(s, k / 150) for s in segs for k in range(151)]

    for a in (1 / 3, 0.35, 0.4, 0.45, 0.5):
        print(f"com pontos: POINT_ARM {a:.2f} · rms {rms(traco[::2], curva(a, 0.45)):.2f} px (END_ARM 0,45)")


if __name__ == "__main__":
    sys.exit(duas_caixas() or com_pontos())
