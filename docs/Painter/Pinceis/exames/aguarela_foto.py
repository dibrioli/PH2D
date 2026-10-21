"""A régua do portão por foto da família `aguarela` (`crates/ph2d-tool-painter/tests/it/familias/aguarela.rs`).

As MESMAS contas do teste Rust (`Identidade::de`), sobre as fotos dos oráculos na cena `gestos` (B, seca) da bateria
(`ferramentas/bateria.json`: recto_lento y=90 x 60→560 · rampa_0_1 y=400, p = (x−60)/500 · vertical x=700 y 440→610).
A cobertura de um píxel é máx|c − 255| / (255 − 23): o azul da bateria (23, 101, 179) cheio vale 1.

- `opacidade` — o pico médio das colunas do recto lento;
- `rampa_largura` / `rampa_cobertura` — a largura e o pico na pressão ~0,3 ÷ na ~0,95 (a rampa 0→1);
- `chato` — a largura do recto ÷ a do vertical (a lâmina fixa na tela; 1 = redondo);
- `orla` — a tinta nos 3 px de dentro da borda ÷ a do terço do meio (> 1 = o fio escuro da lavagem);
- `grao` — o desvio ÷ a média da tinta do miolo do recto, píxel a píxel (o papel e as cerdas).

uso: python3 -I aguarela_foto.py <foto gestos_B .png|.ppm> …   → uma linha por foto
"""

import sys

import numpy as np
from PIL import Image

CHEIO = 255.0 - 23.0


def cobertura(caminho):
    a = np.asarray(Image.open(caminho).convert("RGB")).astype(np.float64)
    return np.abs(a - 255.0).max(axis=2) / CHEIO


def largura(v):
    m = v.max()
    return float((v >= 0.25 * m).sum()) if m > 0.02 else 0.0


def orla(v):
    m = v.max()
    if m <= 0.02:
        return 0.0
    dentro = np.nonzero(v >= 0.25 * m)[0]
    a, b = int(dentro[0]), int(dentro[-1])
    if b - a < 9:
        return 0.0
    borda = np.concatenate([v[a:a + 3], v[b - 2:b + 1]]).mean()
    t = (b - a) // 3
    return float(borda / max(1e-6, v[a + t:b - t + 1].mean()))


def mede(caminho):
    c = cobertura(caminho)
    rec = [c[30:150, x] for x in range(200, 421, 20)]
    leve = [c[340:460, x] for x in (205, 210, 215)]
    forte = [c[340:460, x] for x in (530, 535, 540)]
    hor = np.mean([largura(v) for v in rec])
    ver = np.mean([largura(c[y, 640:761]) for y in (500, 525, 550)])
    # o miolo do recto: as linhas do terço do meio da faixa, x 200–420
    perfil = c[30:150, 200:421].mean(axis=1)
    m = perfil.max()
    dentro = np.nonzero(perfil >= 0.25 * m)[0] if m > 0.02 else np.array([0, 0])
    a, b = 30 + int(dentro[0]), 30 + int(dentro[-1])
    t = max(1, (b - a) // 3)
    miolo = c[a + t:b - t + 1, 200:421]
    return {
        "opacidade": float(np.mean([v.max() for v in rec])),
        "rampa_largura": float(np.mean([largura(v) for v in leve]) / max(1.0, np.mean([largura(v) for v in forte]))),
        "rampa_cobertura": float(np.mean([v.max() for v in leve]) / max(1e-6, np.mean([v.max() for v in forte]))),
        "chato": float(hor / max(1.0, ver)),
        "orla": float(np.mean([orla(v) for v in rec])),
        "grao": float(miolo.std() / max(1e-6, miolo.mean())),
    }


if __name__ == "__main__":
    for f in sys.argv[1:]:
        m = mede(f)
        print(f.rsplit("/", 2)[-2:], " ".join(f"{k} {v:.3f}" for k, v in m.items()))
