"""A régua do BORRÃO do cruzamento (cena `sobre` da bateria, foto A ou B).

Para cada vertical azul (x = 250 · 500 lentos, 750 rápido) e cada faixa vermelha (molhada y≈150, seca y≈470):
- mistura = píxel de tinta longe do azul limpo E do vermelho limpo (a cor que nem um nem outro tem);
- acima / abaixo = quantos px a mistura passa da borda da faixa (linhas com ≥ 3 píxeis de mistura, contíguas);
- lado = quantos px a mistura passa da borda do vertical, dentro da faixa;
- area_fora = píxeis de mistura FORA do rectângulo faixa ∩ vertical;
- L = a luminância mediana da mistura dentro do rectângulo (o escurecimento).
uso: python3 -I borrao.py <png>... [--json]
"""
import json
import sys

import numpy as np
from PIL import Image

T = 55.0


def lum(c):
    return 0.2126 * c[..., 0] + 0.7152 * c[..., 1] + 0.0722 * c[..., 2]


def bordas(mask_col):
    ys = np.nonzero(mask_col)[0]
    return (int(ys.min()), int(ys.max())) if len(ys) else (None, None)


def mede(path):
    im = np.asarray(Image.open(path).convert("RGB")).astype(np.float32)
    h, w, _ = im.shape
    sx, sy = w / 1000.0, h / 640.0
    tinta = im.sum(-1) < 690
    out = {}
    for faixa, yc in (("molhada", 150), ("seca", 470)):
        # vermelho limpo e bordas da faixa, a meio entre os verticais
        xs = [int(375 * sx), int(625 * sx)]
        y0, y1 = int((yc - 50) * sy), int((yc + 50) * sy)
        col = im[y0:y1, xs[0] - 3 : xs[0] + 4]
        vm = (col[..., 0] > 110) & (col[..., 2] < 110)
        t, b = bordas(vm.any(1))
        if t is None:
            continue
        top, bot = y0 + t, y0 + b
        red = np.median(col[vm], 0)
        for x0, nome in ((250, "1"), (500, "2"), (750, "3r")):
            x = int(x0 * sx)
            # azul limpo: o vertical sobre o papel entre as faixas, longe do recto (y 310)
            ya, yb = int(240 * sy), int(280 * sy)
            strip = im[ya:yb, x - 40 : x + 41]
            bm = (strip[..., 2] > strip[..., 0] + 40) & tinta[ya:yb, x - 40 : x + 41]
            if bm.sum() < 20:
                continue
            blue = np.median(strip[bm], 0)
            cols = np.nonzero(bm.mean(0) > 0.5)[0]
            if len(cols) == 0:
                continue
            xl, xr = x - 40 + int(cols.min()), x - 40 + int(cols.max())
            # janela
            wy0, wy1 = max(0, top - int(90 * sy)), min(h, bot + int(90 * sy))
            wx0, wx1 = max(0, x - int(60 * sx)), min(w, x + int(61 * sx))
            win = im[wy0:wy1, wx0:wx1]
            dB = np.linalg.norm(win - blue, axis=-1)
            dR = np.linalg.norm(win - red, axis=-1)
            # a mistura vermelho × azul: o verde cai (o vermelho tem G 0) e não é o vermelho
            mix = (win[..., 1] < 0.65 * blue[1]) & (win[..., 0] < 110) & tinta[wy0:wy1, wx0:wx1]
            rows = mix.sum(1) >= 3
            # contíguo à faixa: cresce a partir do meio da faixa
            ri = (top + bot) // 2 - wy0
            a = ri
            while a - 1 >= 0 and rows[a - 1]:
                a -= 1
            z = ri
            while z + 1 < len(rows) and rows[z + 1]:
                z += 1
            if not rows[ri]:
                a = z = ri
            acima = max(0, top - (wy0 + a))
            abaixo = max(0, (wy0 + z) - bot)
            band_rows = slice(top - wy0, bot - wy0 + 1)
            mc = mix[band_rows].sum(0) >= 3
            cs = np.nonzero(mc)[0]
            lado = 0
            if len(cs):
                lado = max(0, xl - (wx0 + int(cs.min())), (wx0 + int(cs.max())) - xr)
            dentro = np.zeros_like(mix)
            dentro[band_rows, xl - wx0 : xr - wx0 + 1] = True
            area_fora = int((mix & ~dentro).sum())
            core = win[mix & dentro]
            L = float(np.median(lum(core))) if len(core) else float("nan")
            out[f"{faixa}_{nome}"] = dict(
                acima=round(acima / sy), abaixo=round(abaixo / sy), lado=round(lado / sx),
                area_fora=round(area_fora / (sx * sy)), L=round(L, 1),
                larg_azul=round((xr - xl + 1) / sx), faixa=[round(top / sy), round(bot / sy)],
            )
    return out


if __name__ == "__main__":
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    js = "--json" in sys.argv
    res = {p: mede(p) for p in args}
    if js:
        print(json.dumps(res))
    else:
        for p, r in res.items():
            n = p.rsplit("/", 2)[-2] + "/" + p.rsplit("/", 1)[-1].replace("_sobre_A.png", "").replace("_sobre_B.png", "·B")
            def f(k):
                v = r.get(k)
                return "  --  " if v is None else f"{v['acima']:3d}/{v['abaixo']:3d}/{v['lado']:2d} a{v['area_fora']:5d} L{v['L']:5.1f}"
            print(f"{n:52s} M1 {f('molhada_1')} | M2 {f('molhada_2')} | M3r {f('molhada_3r')} || S1 {f('seca_1')} | S3r {f('seca_3r')}")
