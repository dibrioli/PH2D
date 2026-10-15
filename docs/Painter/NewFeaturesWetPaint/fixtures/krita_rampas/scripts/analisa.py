# Análise das saídas do oráculo Krita (BGRA cru de Node.pixelData, 600x200, não-pré-multiplicado).
# Uso: python3 -I analisa.py <dir_bgra> <dir_png_saida> <pattern_stock.png>
import sys, json, glob, os
import numpy as np
from PIL import Image
D, O, STOCK = sys.argv[1], sys.argv[2], sys.argv[3]
W, H = 600, 200
STROKES = [dict(x0=40, x1=540, y=60), dict(x0=63, x1=563, y=140)]   # 2.º = 1.º + (23, 80)
DY = np.arange(-26, 27)                     # linhas dentro do disco (r=30), longe da borda
XR = np.arange(40, 460)                     # x relativo ao início do traço, longe das pontas
ramp = np.arange(64) / 63.0                 # o padrão rampa_horizontal_64: valor = (x mod 64)/63
stock = np.asarray(Image.open(STOCK).convert('L'), np.float64) / 255.0
def load(n):
    a = np.fromfile(n, np.uint8).reshape(H, W, 4).astype(np.float64)
    return a[..., 2], a[..., 1], a[..., 0], a[..., 3]          # R G B A
def region(arr, s):
    return arr[(s['y'] + DY)[:, None], (s['x0'] + XR)[None, :]]
def corr(a, b):
    a = a - a.mean(); b = b - b.mean(); d = np.sqrt((a * a).sum() * (b * b).sum())
    return float((a * b).sum() / d) if d > 1e-12 else float('nan')
res = {}
for f in sorted(glob.glob(os.path.join(D, '*.bgra'))):
    name = os.path.basename(f)[:-5]
    R, G, B, A = load(f)
    Image.fromarray(np.stack([R, G, B, A], -1).astype(np.uint8), 'RGBA').save(os.path.join(O, name + '.png'))
    out = {}
    tprof, tcols, acols = [], [], []
    for k, s in enumerate(STROKES):
        a = region(A, s) / 255.0; r = region(R, s); g = region(G, s); b = region(B, s)
        m = a > 16 / 255.0
        t = np.where(m & (r + b > 8), b / np.maximum(r + b, 1e-9), np.nan)   # 0 = vermelho, 1 = azul
        cx = s['x0'] + XR                                                   # x de CANVAS de cada coluna
        st = dict(mean_alpha=round(float(a.mean()), 4), holes_frac=round(float((a < 0.5).mean()), 4),
                  green_frac_of_rgb=round(float(np.nansum(g * m) / max(np.nansum((r + g + b) * m), 1)), 4))
        if np.isfinite(t).sum() > 100:
            st['t_mean'] = round(float(np.nanmean(t)), 4)
            st['t_std_along_stroke'] = round(float(np.nanmean(np.nanstd(t, axis=1))), 4)   # varia com x?
            st['t_std_across_stroke'] = round(float(np.nanstd(np.nanmean(t, axis=1))), 4)  # varia com dy?
            tc = np.nanmean(t, axis=0); tcols.append((cx, tc))
            st['corr_t_vs_canvasRamp'] = round(corr(tc, ramp[cx % 64]), 4)
            st['corr_t_vs_strokeRelRamp'] = round(corr(tc, ramp[XR % 64]), 4)
            tprof.append(np.nanmean(t, axis=1))
        ac = a.mean(axis=0); acols.append((cx, ac))
        st['alpha_col_std'] = round(float(ac.std()), 4)
        st['corr_alpha_vs_canvasRamp'] = round(corr(ac, ramp[cx % 64]), 4)
        st['corr_alpha_vs_strokeRelRamp'] = round(corr(ac, ramp[XR % 64]), 4)
        # padrão stock 2D: melhor deslocamento (ox, oy) do ladrilho contra o mapa de alfa
        ph, pw = stock.shape
        yy = (s['y'] + DY)[:, None]; xx = cx[None, :]
        best = max(((corr(a, stock[(yy + oy) % ph, (xx + ox) % pw]), ox, oy) for oy in range(0, ph, 1) for ox in range(0, pw, 4)) if 'stock' in name.lower() else [(float('nan'), 0, 0)])
        if 'stock' in name.lower():
            st['stock_corr_at_canvas_origin'] = round(corr(a, stock[yy % ph, xx % pw]), 4)
            st['stock_best_offset'] = [best[1], best[2], round(best[0], 4)]
        out[f'stroke{k + 1}'] = st
    if len(tprof) == 2:
        out['t_profile_across_maxdiff_between_strokes'] = round(float(np.nanmax(np.abs(tprof[0] - tprof[1]))), 4)
        out['t_profile_stroke1_dy_-24_0_+24'] = [round(float(tprof[0][i]), 3) for i in (2, 26, 50)]
    if len(tcols) == 2:
        out['t_cols_same_RELATIVE_x_maxdiff'] = round(float(np.nanmax(np.abs(tcols[0][1] - tcols[1][1]))), 4)
        c0, t0 = tcols[0]; c1, t1 = tcols[1]; com = np.intersect1d(c0, c1)
        out['t_cols_same_CANVAS_x_maxdiff'] = round(float(np.nanmax(np.abs(t0[np.isin(c0, com)] - t1[np.isin(c1, com)]))), 4)
    c0, a0 = acols[0]; c1, a1 = acols[1]; com = np.intersect1d(c0, c1)
    out['alpha_cols_same_RELATIVE_x_maxdiff'] = round(float(np.abs(a0 - a1).max()), 4)
    out['alpha_cols_same_CANVAS_x_maxdiff'] = round(float(np.abs(a0[np.isin(c0, com)] - a1[np.isin(c1, com)]).max()), 4)
    res[name] = out
print(json.dumps(res, indent=1))
