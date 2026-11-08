# Gera as entradas sintéticas do oráculo do contorno (bordas suaves, anti-aliasing por supersampling 8×).
#     python3 -I docs/GameEngineFuncional/oraculo_contorno/gera_sinteticas.py
# Determinístico: mesma saída byte a byte (PIL, sem metadados de data).
import os, sys
import numpy as np
from PIL import Image

AQUI = os.path.dirname(os.path.abspath(sys.argv[0]))
SAI = os.path.join(AQUI, "entradas")
SS = 8  # supersampling

def grava(nome, w, h, dentro, cor=(220, 120, 40)):
    ys, xs = np.mgrid[0:h * SS, 0:w * SS]
    x = (xs + 0.5) / SS
    y = (ys + 0.5) / SS
    m = dentro(x, y).astype(np.float32)
    a = m.reshape(h, SS, w, SS).mean(axis=(1, 3))
    rgba = np.zeros((h, w, 4), np.uint8)
    rgba[..., 0], rgba[..., 1], rgba[..., 2] = cor
    rgba[..., 3] = np.round(a * 255).astype(np.uint8)
    Image.fromarray(rgba, "RGBA").save(os.path.join(SAI, nome), optimize=False)

# L: côncava com um canto interno de 90°.
grava("sint_L.png", 96, 96, lambda x, y: ((x > 12) & (x < 40) & (y > 10) & (y < 86)) | ((x > 12) & (x < 84) & (y > 58) & (y < 86)))
# Anel: tem BURACO (armadilha: a Godot devolve o buraco como outro polígono?).
grava("sint_anel.png", 96, 96, lambda x, y: ((x - 48) ** 2 + (y - 48) ** 2 < 40 ** 2) & ((x - 48) ** 2 + (y - 48) ** 2 > 22 ** 2))
# Crescente: côncava curva, sem buraco.
grava("sint_crescente.png", 96, 96, lambda x, y: ((x - 44) ** 2 + (y - 48) ** 2 < 38 ** 2) & ((x - 64) ** 2 + (y - 42) ** 2 > 32 ** 2))
# Moeda: disco convexo + uma ilha solta (brilho destacado) — duas ilhas.
grava("sint_moeda_ilha.png", 64, 64, lambda x, y: ((x - 30) ** 2 + (y - 34) ** 2 < 24 ** 2) | ((x - 57) ** 2 + (y - 7) ** 2 < 4 ** 2), (240, 200, 40))
print("ok")
