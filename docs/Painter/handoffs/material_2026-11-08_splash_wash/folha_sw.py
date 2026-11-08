# Folha (08/11): por pincel, as variantes lado a lado (a 1.ª = antes), uma linha por fundo (vazio · branco · blocos).
# Uso: python3 -I folha_sw.py <out_ph2d> <saida.png> <Nome_slug> <sufixo>... (sufixo '-' = sem sufixo)
import sys, pathlib
import numpy as np
from PIL import Image, ImageDraw
W, H, COL, ESC = 2400, 880, 800, 0.5
out, saida, slug, sufs = pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3], sys.argv[4:]
xadrez = ((np.indices((H, COL)).sum(0) // 16) % 2 * 40 + 200).astype(np.float32)[..., None].repeat(3, -1)
cw, ch = int(COL * ESC), int(H * ESC)
folha = Image.new('RGB', (cw * len(sufs) + 8 * len(sufs), ch * 3 + 30 + 16), 'white')
d = ImageDraw.Draw(folha)
for j, s in enumerate(sufs):
    nome = slug + ('' if s == '-' else '__' + s)
    a = np.frombuffer((out / f'{nome}.rgba').read_bytes(), np.uint8).reshape(H, W, 4).astype(np.float32) / 255
    d.text((j * (cw + 8) + 4, 6), nome, fill='black')
    for k in range(3):
        f = a[:, k * COL:(k + 1) * COL]
        fundo = xadrez / 255 if k == 0 else 1.0
        rgb = f[..., :3] * f[..., 3:4] + fundo * (1 - f[..., 3:4])
        im = Image.fromarray((rgb * 255).clip(0, 255).astype(np.uint8)).resize((cw, ch), Image.LANCZOS)
        folha.paste(im, (j * (cw + 8), 30 + k * (ch + 8)))
folha.save(saida)
