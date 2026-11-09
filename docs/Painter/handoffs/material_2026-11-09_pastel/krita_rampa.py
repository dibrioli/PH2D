# A curva cobertura × pressão do Krita h) Chalk Soft (rodada 2, 09/11): a reta da coluna (b) (a (a) do Krita perde-se),
# rampa 0,1→1 (bateria_pastel_rampa*.json de prepara_rampa.py). Uso: python3 -I krita_rampa.py <scratch>
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from carrega import COL, k_de_rgb_sobre_branco  # noqa: E402
from lei import NUCLEO, PS, x_de_p  # noqa: E402


def k_krita(png):
    a = np.asarray(Image.open(png).convert('RGB')).astype(np.float32)
    return k_de_rgb_sobre_branco(a[:, COL:2 * COL])


def curva_sem_papel(k):
    out = []
    for p in PS:
        x = x_de_p(p)
        kk = k[NUCLEO, x - 25:x + 25].ravel()
        out.append(dict(p=p, cob=round(float(kk.mean()), 3), pintada=round(float((kk >= 0.05).mean()), 3),
                        cv=round(float(kk.std() / max(kk.mean(), 1e-9)), 2)))
    return out


if __name__ == '__main__':
    S = sys.argv[1]
    k1 = k_krita(f'{S}/krita_rampa/h__Chalk_Soft_rampa.png')
    k2 = k_krita(f'{S}/krita_rampa_x2/h__Chalk_Soft_rampa_x2.png')
    for n, k in (('rampa', k1), ('rampa_x2', k2)):
        c = curva_sem_papel(k)
        print(n, 'cob', [x['cob'] for x in c], 'pint', [x['pintada'] for x in c], 'cv', [x['cv'] for x in c])
    # coerência do grão entre a 1.ª e a 2.ª passada: o resíduo do x2 menos o do x1 correlaciona com o x1?
    for p in (0.5, 0.65, 0.8):
        x = x_de_p(p)
        a, b = k1[NUCLEO, x - 25:x + 25].ravel(), k2[NUCLEO, x - 25:x + 25].ravel()
        print(f'p {p}: corr(1 passada, 2 passadas) = {np.corrcoef(a, b)[0, 1]:.2f}; '
              f'píxeis vazios na 1.ª que a 2.ª pinta: {float(((a < 0.05) & (b >= 0.05)).sum() / max((a < 0.05).sum(), 1)):.2f}')
