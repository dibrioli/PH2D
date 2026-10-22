"""A régua da família `guache` sobre fotos da cena `gestos` (bateria: recto_lento y=90 x 60→560; vertical x=700).

- `opac`  — o pico médio das colunas do recto (cobertura 0..1; o azul cheio da bateria = 1);
- `orla`  — a tinta nos 3 px de dentro da borda ÷ a do terço do meio (> 1 = o fio escuro da aguarela);
- `grao`  — desvio ÷ média da tinta no miolo do recto, píxel a píxel;
- `falha` — a fracção de papel (tinta < 0,3 do miolo) DENTRO do envelope do recto (o seco: arrasto granuloso);
- `rampa` — a largura da borda (px entre 20 % e 80 % do miolo, média de cima e de baixo): a borda macia da lavagem;
- `larg`  — a largura do recto (px ≥ 25 % do pico).

uso: python3 -I guache_foto.py <gestos_A.png> …
"""
import sys

import numpy as np
from PIL import Image

CHEIO = 255.0 - 23.0


def cobertura(caminho):
    a = np.asarray(Image.open(caminho).convert("RGB")).astype(np.float64)
    return np.abs(a - 255.0).max(axis=2) / CHEIO


def mede(caminho):
    c = cobertura(caminho)
    xs = range(200, 421, 4)
    cols = [c[30:150, x] for x in xs]
    pico = np.array([v.max() for v in cols])
    perfil = c[30:150, 200:421].mean(axis=1)
    m = perfil.max()
    dentro = np.nonzero(perfil >= 0.25 * m)[0]
    a, b = int(dentro[0]), int(dentro[-1])
    t = max(1, (b - a) // 3)
    miolo = c[30 + a + t:30 + b - t + 1, 200:421]
    mm = float(np.median(miolo))
    env = c[30 + a + 2:30 + b - 1, 200:421]
    borda = np.concatenate([perfil[a:a + 3], perfil[b - 2:b + 1]]).mean()
    # a largura da borda: de 20 % a 80 % do miolo, em cima e em baixo
    p = perfil / max(1e-6, perfil[a + t:b - t + 1].mean())
    cima = np.argmax(p >= 0.8) - np.argmax(p >= 0.2)
    q = p[::-1]
    baixo = np.argmax(q >= 0.8) - np.argmax(q >= 0.2)
    return {
        "opac": float(pico.mean()),
        "orla": float(borda / max(1e-6, perfil[a + t:b - t + 1].mean())),
        "grao": float(miolo.std() / max(1e-6, miolo.mean())),
        "falha": float((env < 0.3 * mm).mean()),
        "rampa": float((cima + baixo) / 2),
        "larg": float(b - a + 1),
    }


if __name__ == "__main__":
    for f in sys.argv[1:]:
        m = mede(f)
        nome = "/".join(f.rsplit("/", 2)[-2:])
        print(f"{nome[-58:]:>58}", " ".join(f"{k} {v:6.3f}" for k, v in m.items()))
