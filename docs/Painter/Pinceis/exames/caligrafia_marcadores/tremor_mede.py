"""tremor_mede.py <dir…> — o tremor que fica no traço: o desvio RMS do centro da tinta à recta (y=100) e à curva S,
e onde acaba o traço limpo (x do último píxel com tinta na linha y=500; o gesto acaba em x=939)."""
import sys, math
import numpy as np
from PIL import Image
for d in sys.argv[1:]:
    a = np.asarray(Image.open(f"{d}/tremor_A.ppm").convert("L"), np.float64)
    tinta = (255 - a) / 255
    def centro(y0, y1, x):
        c = tinta[y0:y1, x]
        s = c.sum()
        return (np.arange(y0, y1) * c).sum() / s if s > 0.5 else None
    rec = [centro(70, 131, x) for x in range(120, 880)]
    rec = np.array([c for c in rec if c is not None])
    r_rec = math.sqrt(((rec - 100) ** 2).mean())
    sdev = []
    for x in range(120, 880):
        c = centro(230, 371, x)
        if c is not None:
            sdev.append(c - (300 + 50 * math.sin(2 * math.pi * (x - 60) / 440)))
    sdev = np.array(sdev)
    r_s = math.sqrt(((sdev - sdev.mean()) ** 2).mean())
    linha = np.where(tinta[500, :] > 0.25)[0]
    fim = linha.max() if len(linha) else -1
    ini = linha.min() if len(linha) else -1
    larg = (tinta[480:521, 500] > 0.25).sum()
    print(f"{d.split('/')[-1]:>16}: tremor recto rms {r_rec:5.2f} px · S rms {r_s:5.2f} · limpo {ini}..{fim} · largura {larg}")
