# Coerência do grão com a TELA: a mesma reta (y = 110, x 150..650) pintada ida (L→R) e volta (R→L) em corridas
# separadas; correlação dos resíduos (k − média da linha) no miolo. ~1 = o grão é da tela/papel; ~0 = sorteado.
import numpy as np


def coer(ka, kb, x0=0):
    ja, jb = ka[65:155, x0 + 150:x0 + 650], kb[65:155, x0 + 150:x0 + 650]
    prof = (ja.mean(1) + jb.mean(1)) / 2
    m = prof >= 0.8 * prof.max()
    ra = ja[m] - ja[m].mean(1, keepdims=True)
    rb = jb[m] - jb[m].mean(1, keepdims=True)
    return round(float(np.corrcoef(ra.ravel(), rb.ravel())[0, 1]), 2)
