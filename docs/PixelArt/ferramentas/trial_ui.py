"""Acha widgets do diálogo do Aseprite Trial por casamento EXATO de modelo.
uso: trial_ui.py <tpldir> shot.png nome [nome...] -> 'nome x y' (centro) por ocorrência, ordenado por y."""
import sys
import numpy as np
from PIL import Image
tdir, shot = sys.argv[1], sys.argv[2]
S = np.asarray(Image.open(shot).convert('RGB')).astype(np.int32)
Sk = (S[..., 0] << 16) | (S[..., 1] << 8) | S[..., 2]
for name in sys.argv[3:]:
    T = np.asarray(Image.open(f'{tdir}/{name}.png').convert('RGB')).astype(np.int32)
    Tk = (T[..., 0] << 16) | (T[..., 1] << 8) | T[..., 2]
    th, tw = Tk.shape
    # pixel mais raro do modelo no ecrã
    vals, cnt = np.unique(Tk, return_counts=True)
    best = min(vals, key=lambda v: (Sk == v).sum() or 10**9)
    ty, tx = np.argwhere(Tk == best)[0]
    hits = []
    for y, x in np.argwhere(Sk == best):
        y0, x0 = y - ty, x - tx
        if y0 < 0 or x0 < 0 or y0 + th > Sk.shape[0] or x0 + tw > Sk.shape[1]:
            continue
        if np.array_equal(Sk[y0:y0 + th, x0:x0 + tw], Tk):
            hits.append((y0 + th // 2, x0 + tw // 2))
    for y, x in sorted(hits):
        print(name, x, y)
