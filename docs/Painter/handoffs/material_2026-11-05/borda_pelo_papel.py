#!/usr/bin/env python3
"""SIMULAÇÃO (em imagem, não é o motor) de «variar a borda pelo papel». borda_pelo_papel.py <pasta das fotos> <saida.png>
Da foto de fábrica sai a cobertura a = tinta ÷ miolo, e o papel h (a luz do fundo, sem tinta, estendida por baixo do traço).
Só onde 0 < a < 1 (a orla) a cobertura desloca-se pelo vale/crista do papel: a' = clip(a + k·(h−0,5)·4a(1−a)); o miolo não muda.
Coluna do motor: PaperGate 0,25 e 0,6 pelos knobs que já existem (mexem no miolo também)."""
import sys
import numpy as np
from PIL import Image, ImageDraw, ImageFont
import glob
sys.path.insert(0, __file__.rsplit('/', 1)[0])
from manchas import caixa, distancia

def fonte(n):
    p = glob.glob("/usr/share/fonts/**/DejaVuSans.ttf", recursive=True)
    return ImageFont.truetype(p[0], n) if p else ImageFont.load_default()

def le(arq):
    return np.asarray(Image.open(arq).convert("RGB"), float)

def papel_h(ref):
    """O relevo: reconstruído da imagem do papel sem tinta (`cena sem tinta` = o fundo; o que a tinta tapa não importa fora da orla)."""
    lum = ref.mean(2)
    return np.clip((lum - np.percentile(lum, 2)) / (np.percentile(lum, 98) - np.percentile(lum, 2)), 0, 1)

def sim(fab, k):
    fundo = np.median(fab[:12, :12], axis=(0, 1))
    d = np.abs(fab - fundo).sum(2)
    top = np.percentile(d, 99)
    a = np.clip(d / top, 0, 1)
    tinta = np.median(fab[a > 0.95], axis=0)
    # O papel debaixo da orla: a luz local do fundo, com a tinta tirada por a.
    base = (fab - a[..., None] * tinta) / np.maximum(1 - a[..., None], 0.15)
    h = papel_h(np.clip(base, 0, 255))
    # Só a faixa da orla (±9 px do raio 33 da curva): fora dela a imagem é a de fábrica, ao byte.
    dist = distancia(*fab.shape[:2])
    faixa = np.clip(1 - np.abs(dist - 33) / 9, 0, 1)[..., None]
    # O papel da orla: fora do traço o relevo é o próprio fundo; lê-se dali, à mesma posição do lado de fora.
    h = papel_h(caixa(fab.mean(2), 3)[..., None].repeat(3, 2))
    a2 = np.clip(a + k * (h - 0.5), 0, 1)
    fundo_img = np.clip(base, 0, 255)
    novo = np.clip(fundo_img * (1 - a2[..., None]) + tinta * a2[..., None], 0, 255)
    return fab * (1 - faixa) + novo * faixa

BOX = (180, 110, 340, 230)
if __name__ == "__main__":
    d, saida = sys.argv[1], sys.argv[2]
    linhas = []
    for pincel in ("Wet_Liner", "Wet_Wash"):
        p = f"{d}/{pincel}_papel_sem_vermelho_cheia140_"
        fab = le(p + "fabrica_produto.ppm")
        linhas.append((pincel, [("fábrica (hoje)", fab), ("sim. orla k=0,6", sim(fab, 0.6)), ("sim. orla k=1,2", sim(fab, 1.2)),
                                ("motor PaperGate 0,25", le(p + "portao25_produto.ppm")), ("motor PaperGate 0,6", le(p + "portao60_produto.ppm"))]))
    W, H = (BOX[2] - BOX[0]) * 2, (BOX[3] - BOX[1]) * 2
    o = Image.new("RGB", (5 * (W + 6) + 6, 2 * (H + 24)), (245, 245, 245))
    D = ImageDraw.Draw(o)
    for l, (pincel, cols) in enumerate(linhas):
        for c, (rot, im) in enumerate(cols):
            x, y = 6 + c * (W + 6), l * (H + 24)
            D.text((x, y + 3), f"{pincel.replace('_', ' ')}: {rot}", fill=(20, 20, 120), font=fonte(13))
            r = Image.fromarray(im.astype("uint8")).crop(BOX).resize((W, H), Image.LANCZOS)
            o.paste(r, (x, y + 22))
    o.save(saida)
    print(o.size)
