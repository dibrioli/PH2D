"""A régua do portão por foto da família `basicos` (`crates/ph2d-tool-painter/tests/it/familias/basicos.rs`).

As MESMAS contas do teste Rust (`Identidade::de`), sobre as fotos dos oráculos na cena `gestos` da bateria
(`ferramentas/bateria.json`: recto_lento y=90 x 60→560 · rampa_0_1 y=400, p = (x−60)/500 · vertical x=700
y 440→610 · espiral na caixa x 700–920, y 170–373). A cobertura de um píxel é máx|c − 255| / (255 − 23): o azul
da bateria (23, 101, 179) cheio vale 1. A largura e a orla são relativas ao pico da própria coluna.

uso: python3 -I basicos_foto.py <foto gestos .png|.ppm> …   → uma linha por foto com as 6 medidas
"""

import sys

import numpy as np
from PIL import Image

CHEIO = 255.0 - 23.0


def cobertura(caminho):
    a = np.asarray(Image.open(caminho).convert("RGB")).astype(np.float64)
    return np.abs(a - 255.0).max(axis=2) / CHEIO


def coluna(c, x, y0, y1):
    return c[y0:y1, x]


def largura(v):
    """A largura a ¼ do PICO da própria coluna (px): a geometria, sem a opacidade."""
    m = v.max()
    return float((v >= 0.25 * m).sum()) if m > 0.02 else 0.0


def borda(v):
    """Os píxeis da orla (entre 10 % e 90 % do pico) de cada lado: 0 = binário, ~1 = um píxel de AA."""
    m = v.max()
    return float(((v > 0.1 * m) & (v < 0.9 * m)).sum()) / 2.0 if m > 0.02 else 0.0


def mede(caminho):
    c = cobertura(caminho)
    rec = [coluna(c, x, 30, 150) for x in range(200, 421, 20)]
    # a orla em duas fases da grelha: o recto lento e o topo da curva em S (y ≈ 195,9)
    topo_s = [coluna(c, x, 150, 250) for x in range(295, 326, 5)]
    b = 0.5 * (float(np.mean([borda(v) for v in rec])) + float(np.mean([borda(v) for v in topo_s])))
    opac = float(np.mean([v.max() for v in rec]))
    leve = [coluna(c, x, 340, 460) for x in (205, 210, 215)]
    forte = [coluna(c, x, 340, 460) for x in (530, 535, 540)]
    rl = np.mean([largura(v) for v in leve]) / max(1.0, np.mean([largura(v) for v in forte]))
    rc = np.mean([v.max() for v in leve]) / max(1e-6, np.mean([v.max() for v in forte]))
    hor = np.mean([largura(v) for v in rec])
    ver = np.mean([largura(c[y, 640:761]) for y in (500, 525, 550)])
    chato = hor / max(1.0, ver)
    tecto = float(c[170:374, 700:921].max()) / max(1e-6, opac)
    return {"borda": b, "opacidade": opac, "rampa_largura": float(rl), "rampa_cobertura": float(rc),
            "chato": float(chato), "tecto": tecto}


if __name__ == "__main__":
    for f in sys.argv[1:]:
        m = mede(f)
        print(f.rsplit("/", 2)[-2:], " ".join(f"{k} {v:.3f}" for k, v in m.items()))
