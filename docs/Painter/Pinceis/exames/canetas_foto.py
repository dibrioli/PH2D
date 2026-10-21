"""canetas_foto.py <png|ppm>… — a identidade de uma caneta na cena `gestos` (o espelho da `Identidade` de
`familias/canetas.rs`: as MESMAS contas, para medir a foto do oráculo e conferir a paridade com o Rust)."""
import sys
import numpy as np
from PIL import Image
CHEIO = 255.0 - 23.0
def ident(path):
    a = np.asarray(Image.open(path).convert('RGB'), dtype=np.float64)
    c = np.abs(255.0 - a).max(axis=2) / CHEIO
    col = lambda x, y0, y1: c[y0:y1, x]
    pico = lambda v: float(v.max())
    def larg(v):
        m = pico(v); return float((v >= 0.25 * m).sum()) if m > 0.02 else 0.0
    def borda(v):
        m = pico(v); return float(((v > 0.1 * m) & (v < 0.9 * m)).sum()) / 2.0 if m > 0.02 else 0.0
    rec = [col(x, 30, 150) for x in range(200, 421, 20)]
    opac = float(np.mean([pico(v) for v in rec]))
    forte = float(np.mean([larg(col(x, 340, 460)) for x in (530, 535, 540)]))
    fio = float(np.mean([larg(col(x, 340, 460)) for x in (130, 135, 140)]))
    meio = float(np.mean([larg(col(x, 340, 460)) for x in (305, 310, 315)]))
    baixa = float(np.mean([larg(col(x, 340, 460)) for x in (205, 210, 215)]))
    alta = float(np.mean([larg(col(x, 340, 460)) for x in (405, 410, 415)]))
    # a orla de cima do recto lento: a 1.ª linha a ½ do pico, coluna a coluna; a rugosidade = |Δ| médio entre colunas
    topo = []
    for x in range(100, 501):
        v = col(x, 30, 150); m = pico(v)
        topo.append(int(np.argmax(v >= 0.5 * m)) if m > 0.02 else 0)
    rug = float(np.mean(np.abs(np.diff(topo))))
    leve = float(np.mean([pico(col(x, 340, 460)) for x in range(85, 186, 5)])) / max(opac, 1e-6)
    # o traço LEVE (rampa a 0,05–0,25) contra o cheio (0,88–0,97): a aspereza (|Δx| médio ÷ a tinta: furos nítidos ≫,
    # fiapos borrados ≪) e o espalhar (σ vertical da tinta ÷ o do cheio: a pressão leve que espalha a posição)
    def banda(x0, x1):
        b = c[340:460, x0:x1]
        ys = np.arange(340, 460)[:, None]
        w = b.sum(axis=0) + 1e-9
        mu = (b * ys).sum(axis=0) / w
        return np.abs(np.diff(b, axis=1)).mean() / max(b.mean(), 1e-6), np.sqrt((b * (ys - mu) ** 2).sum(axis=0) / w).mean()
    asp, sl = banda(85, 185)
    _, sf = banda(500, 545)
    return dict(borda=round(float(np.mean([borda(v) for v in rec])), 3), opacidade=round(opac, 3),
                rampa_fio=round(fio / max(forte, 1), 3), rampa_meio=round(meio / max(forte, 1), 3),
                rampa_baixa=round(baixa / max(forte, 1), 3), rampa_alta=round(alta / max(forte, 1), 3),
                rugosidade=round(rug, 3), leve=round(leve, 3),
                aspereza_leve=round(float(asp), 3), espalha_leve=round(float(sl / max(sf, 1e-6)), 3))
for p in sys.argv[1:]:
    print(p.split('/')[-1], ident(p))
