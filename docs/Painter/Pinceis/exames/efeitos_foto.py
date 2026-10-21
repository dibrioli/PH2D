"""efeitos_foto.py <foto sobre_A> … — a identidade de um EFEITO (família efeitos, J4) na cena «sobre» da bateria: a cor
no cruzamento 2 (x 500) sobre a faixa de cima (y 150) e a de baixo (y 470), mediana 5×5, e o véu no papel — o pico do
α do traço «papel» (y 280..340, mediana das colunas x 280..320; α = (255 − R) / 232, o azul da bateria). As contas do
portão `familias::efeitos::cada_efeito_se_reconhece_pela_foto`.

  efeitos_foto.py <png|ppm> [--neon <gestos> <tamanhos>]   (--neon: o núcleo/halo/patamar do Glow, recto lento)
"""
import sys
import numpy as np
from PIL import Image


def efeito(f):
    a = np.asarray(Image.open(f).convert('RGB')).astype(float)
    med = lambda x, y: tuple(int(v) for v in np.median(a[y - 2:y + 3, x - 2:x + 3].reshape(-1, 3), axis=0))
    al = np.clip((255 - a[:, :, 0]) / 232, 0, 1)
    p = np.median(al[280:341, 280:321], axis=1)
    return dict(cima=med(500, 150), baixo=med(500, 470), veu=round(float(p.max()), 3))


def neon(gestos, tamanhos):
    def ident(p):
        pk = p.max()
        idx = (p > 0.04) & (p <= 0.5 * pk)
        return dict(pico=round(float(pk), 3), nucleo=int((p > 0.5 * pk).sum()), halo=int((p > 0.04).sum()),
                    patamar=round(float(np.median(p[idx]) / pk), 3) if idx.any() else 0)
    al = lambda f: np.clip((255 - np.asarray(Image.open(f).convert('RGB')).astype(float)[:, :, 0]) / 232, 0, 1)
    g, t = al(gestos), al(tamanhos)
    perfil = lambda a, y0, xs, h: np.median(np.stack([a[y0 - h:y0 + h + 1, x] for x in xs]), axis=0)
    return dict(recto_lento=ident(perfil(g, 90, range(200, 451, 10), 45)),
                recto_200=ident(perfil(t, 440, range(400, 601, 20), 180)))


if __name__ == '__main__':
    if '--neon' in sys.argv:
        i = sys.argv.index('--neon')
        print(neon(sys.argv[i + 1], sys.argv[i + 2]))
    else:
        for f in sys.argv[1:]:
            print(f.split('/')[-1], efeito(f))
