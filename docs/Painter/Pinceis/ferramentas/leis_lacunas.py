#!/usr/bin/env python3
"""As RÉGUAS das leis das lacunas (06_LEIS_DAS_LACUNAS.md): a mesma conta para o oráculo e para o nosso.

Cada sub-comando lê fotos da BATERIA (`bateria.json`, as cenas `gestos`/`tamanhos`/`sobre`) e imprime a medição:

  lamina   <png gestos>…   L3: largura por direcção (rectos 0°, verticais 90°, S ±40°) e o ajuste da lâmina FIXA
                           w(θ) = A|sin(θ−φ)| + B|cos(θ−φ)| × a largura constante; o arranque da vertical (memória)
  modos    <png sobre> <MODO> …   L7: a cor sobre a faixa vermelha e no papel × cada fórmula (sRGB e linear)
  dabs     <png gestos>…   L4: componentes conexas por traço: ⊥ ÷ D, passo, diâmetro, opacidade, n/100 px
  cor      <png gestos>…   L11: ΔH/S/V por traço, ao longo, através, e por decil de pressão na rampa_0_1
  fim      <png gestos> <png tamanhos>   L5+L12: rápido÷lento (verticais), fall-off no recto_40, cauda e cabeça ÷ D
  tecto    <png gestos>…   L8: k = ln T(sobreposição da espiral) ÷ ln T(recto); 1 = tecto por traço, 2 = acumula
  textura  <png tamanhos>…  L6: cobertura, resíduo 5×5 e escala (½-autocorrelação) no recto_40 e no recto_200
  velhas   <dir com log.csv>  fotos consecutivas IGUAIS (máx Δ ≤ 16) — a foto velha de uma cena que não repintou

uso: python3 -I leis_lacunas.py <sub-comando> args…
"""
import csv
import json
import os
import sys
from collections import deque

import numpy as np
from PIL import Image

AQUI = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, AQUI)
import exame_regua as R  # noqa: E402

BAT = json.load(open(R.BATERIA))
AZUL = np.array([23, 101, 179]) / 255.0
H_AZUL = 210.4


def cobertura(png):
    a = R.le(png)
    p = R.papel_de(a)
    return a, p, np.abs(a - p).max(2) / 255.0


def tracos(cena, nomes=None):
    return [t for t in R.tracos(BAT, cena) if t['quem'] == 'preset' and (not nomes or t['nome'] in nomes)]


# ---------------------------------------------------------------- L3 — a lâmina
def larguras(c, a, tr, H=None):
    """Largura por amostra a ¼ do NÍVEL do traço (limiar fixo), tangente não-orientada (graus mod 180)."""
    q, pr, s, t = R.densifica(tr['pts'])
    h = H or R.meia_janela(q, tr['size'])
    S, _, o = R.faixa(c, a, q, t, h, 0)
    nivel = np.percentile(S.max(1), 90)
    w = np.array([(lambda ys: ys[-1] - ys[0] + 1 if len(ys) else 0)(np.where(row >= 0.25 * nivel)[0]) for row in S], float)
    th = (np.degrees(np.arctan2(t[:, 1], t[:, 0])) + 360) % 180
    return w, th, s, pr


def lamina(png):
    a, _, c = cobertura(png)
    L = {t['nome']: larguras(c, a, t) for t in tracos('gestos', ['recto_lento', 'recto_rapido', 'curva_s', 'vertical', 'vertical_rapido'])}

    def med(k, sel=None):
        w = L[k][0]
        m = len(w)
        v = w[sel] if sel is not None else w[m // 10: m - m // 10]
        return float(np.median(v)) if len(v) else float('nan')
    w, th, s, pr = L['curva_s']
    ths = (np.degrees(np.unwrap(np.radians(2 * th)) / 2) + 90) % 180 - 90
    m = len(w)
    core = np.zeros(m, bool)
    core[m // 10: m - m // 10] = True
    th_all, w_all = [], []
    for k in ('recto_lento', 'curva_s', 'vertical'):
        ww, tt = L[k][0], L[k][1]
        mm = len(ww)
        th_all.append(tt[mm // 10: mm - mm // 10])
        w_all.append(ww[mm // 10: mm - mm // 10])
    th_all, w_all = np.concatenate(th_all), np.concatenate(w_all)
    ok = w_all > 0
    best = None
    for phi in range(0, 180, 2):
        r = np.radians(th_all[ok] - phi)
        A = np.stack([np.abs(np.sin(r)), np.abs(np.cos(r))], 1)
        sol, *_ = np.linalg.lstsq(A, w_all[ok], rcond=None)
        e = float(np.sqrt(np.mean((A @ sol - w_all[ok]) ** 2)))
        if best is None or e < best[0]:
            best = (e, phi, sol)
    e, phi, (La, Lb) = best
    eixo = (phi if La >= Lb else phi + 90) % 180
    V = L['vertical'][0]
    w90 = med('vertical')
    return {'w0': med('recto_lento'), 'w0_rapido': med('recto_rapido'), 'w90': w90, 'w90_rapido': med('vertical_rapido'),
            'S_+40': med('curva_s', core & (ths > 30)), 'S_-40': med('curva_s', core & (ths < -30)),
            'arranque_vertical': float(np.median(V[:15]) / w90) if w90 > 0 else float('nan'),
            'fixa_eixo_longo': eixo, 'fixa_longo': float(max(La, Lb)), 'fixa_fino': float(min(La, Lb)),
            'fixa_rms': e, 'constante_rms': float(np.std(w_all[ok]))}


# ---------------------------------------------------------------- L7 — modos
def _lum(c):
    return 0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]


def _clip(c):
    l, n, x = _lum(c), c.min(), c.max()
    if n < 0:
        c = l + (c - l) * l / (l - n)
    if x > 1:
        c = l + (c - l) * (1 - l) / (x - l)
    return c


def _setlum(c, l):
    return _clip(c + (l - _lum(c)))


def _sat(c):
    return c.max() - c.min()


def _setsat(c, s):
    c = c.copy()
    mn, md, mx = np.argsort(c)
    if c[mx] > c[mn]:
        c[md] = (c[md] - c[mn]) * s / (c[mx] - c[mn])
        c[mx] = s
    else:
        c[md] = c[mx] = 0
    c[mn] = 0
    return c


MODOS = {
    'NORMAL': lambda b, s: s, 'MULTIPLY': lambda b, s: b * s, 'SCREEN': lambda b, s: 1 - (1 - b) * (1 - s),
    'OVERLAY': lambda b, s: np.where(b <= 0.5, 2 * b * s, 1 - 2 * (1 - b) * (1 - s)),
    'HARD_LIGHT': lambda b, s: np.where(s <= 0.5, 2 * b * s, 1 - 2 * (1 - b) * (1 - s)),
    'LINEAR_BURN': lambda b, s: np.clip(b + s - 1, 0, 1), 'ADDITIVE': lambda b, s: np.clip(b + s, 0, 1),
    'COLOR_DODGE': lambda b, s: np.clip(b / np.maximum(1 - s, 1e-6), 0, 1),
    'COLOR_BURN': lambda b, s: 1 - np.clip((1 - b) / np.maximum(s, 1e-6), 0, 1),
    'LIGHTEN': np.maximum, 'DARKEN': np.minimum,
    'COLOR': lambda b, s: _setlum(s, _lum(b)), 'HUE': lambda b, s: _setlum(_setsat(s, _sat(b)), _lum(b)),
    'SATURATION': lambda b, s: _setlum(_setsat(b, _sat(s)), _lum(b)), 'LUMINOSITY': lambda b, s: _setlum(b, _lum(s)),
}


def _lin(c):
    return np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)


def _enc(c):
    return np.where(c <= 0.0031308, c * 12.92, 1.055 * np.clip(c, 0, None) ** (1 / 2.4) - 0.055)


def _alfa(base, res, alvo):
    d = alvo - base
    return float(np.dot(res - base, d) / max(np.dot(d, d), 1e-9))


def _caixa(a, x, y, rx, ry):
    return np.median(a[y - ry:y + ry + 1, x - rx:x + rx + 1].reshape(-1, 3), 0)


def modos(png, declarado):
    """Base = a faixa SECA (y 470) longe dos cruzamentos; resultado = os cruzamentos x 250/500; papel = os mesmos x."""
    a = np.asarray(Image.open(png).convert('RGB'), float) / 255.0
    P = _caixa(a, 375, 230, 20, 4)
    Cp = np.mean([_caixa(a, 250, 390, 5, 15), _caixa(a, 500, 390, 5, 15), _caixa(a, 620, 310, 30, 3)], 0)
    Rb = _caixa(a, 375, 470, 30, 8)
    C = np.mean([_caixa(a, x, 470, 5, 8) for x in (250, 500)], 0)
    res = []
    for esp, f, g in (('sRGB', lambda x: x, lambda x: x), ('lin', _lin, _enc)):
        for m, fn in MODOS.items():
            alvo = g(fn(f(Rb), f(AZUL)))
            al = _alfa(Rb, C, alvo)
            if al > 0:
                res.append((float(np.abs(Rb + al * (alvo - Rb) - C).max() * 255), esp, m, al))
    res.sort()
    dec = next((x for x in res if x[2] == declarado and x[1] == 'sRGB'), None)
    return {'base': (Rb * 255).round().tolist(), 'em_cima': (C * 255).round().tolist(), 'dE': float(np.abs(C - Rb).max() * 255),
            'alfa_papel': _alfa(P, Cp, AZUL), 'declarado': dec, 'melhores': res[:3]}


# ---------------------------------------------------------------- L4 — dabs
def _rotula(m):
    h, w = m.shape
    lab = np.zeros((h, w), np.int32)
    n = 0
    for y0, x0 in zip(*np.nonzero(m)):
        if lab[y0, x0]:
            continue
        n += 1
        lab[y0, x0] = n
        dq = deque([(y0, x0)])
        while dq:
            y, x = dq.popleft()
            for yy in (y - 1, y, y + 1):
                for xx in (x - 1, x, x + 1):
                    if 0 <= yy < h and 0 <= xx < w and m[yy, xx] and not lab[yy, xx]:
                        lab[yy, xx] = n
                        dq.append((yy, xx))
    return lab, n


def dabs(png, limiar=0.12):
    _, _, c = cobertura(png)
    lab, n = _rotula(c > limiar)
    idx, cv = lab.ravel(), c.ravel()
    area = np.bincount(idx, minlength=n + 1)
    soma = np.bincount(idx, cv, minlength=n + 1)
    yy, xx = np.indices(c.shape)
    cx = np.bincount(idx, (xx * c).ravel(), n + 1) / np.maximum(soma, 1e-9)
    cy = np.bincount(idx, (yy * c).ravel(), n + 1) / np.maximum(soma, 1e-9)
    comps = np.arange(1, n + 1)[area[1:] >= 3]
    P = np.stack([cx[comps], cy[comps]], 1)
    ts = tracos('gestos', ['recto_lento', 'recto_rapido', 'rampa_0_1', 'rampa_afunila', 'vertical', 'vertical_rapido', 'curva_s'])
    dmin = np.full(len(comps), np.inf)
    dono = np.full(len(comps), -1)
    off = np.zeros(len(comps))
    sat = np.zeros(len(comps))
    for j, tr in enumerate(ts):
        q, pr, s, t = R.densifica(tr['pts'])
        d2 = ((P[:, None, :] - q[None, :, :]) ** 2).sum(2)
        i = d2.argmin(1)
        d = np.sqrt(d2[np.arange(len(P)), i])
        o = ((P - q[i]) * np.stack([-t[i, 1], t[i, 0]], 1)).sum(1)
        u = d < dmin
        dmin[u], dono[u], off[u], sat[u] = d[u], j, o[u], s[i][u]
    out = {}
    for j, tr in enumerate(ts):
        sel = (dono == j) & (dmin < 2 * tr['size'])
        if sel.sum() < 4:
            continue
        cc = comps[sel]
        dia = 2 * np.sqrt(area[cc] / np.pi)
        op = soma[cc] / area[cc]
        gaps = np.diff(np.sort(sat[sel]))
        comp = R.densifica(tr['pts'])[2][-1]
        out[tr['nome']] = {'n': int(sel.sum()), 'por_100px': float(100 * sel.sum() / comp),
                           'perp_sd_D': float(off[sel].std() / tr['size']),
                           'diam_p10_50_90': np.percentile(dia, [10, 50, 90]).round(1).tolist(), 'diam_cv': float(dia.std() / dia.mean()),
                           'opac_p10_50_90': np.percentile(op, [10, 50, 90]).round(3).tolist(), 'opac_cv': float(op.std() / op.mean()),
                           'passo_media': float(gaps.mean()) if len(gaps) > 3 else None,
                           'passo_cv': float(gaps.std() / gaps.mean()) if len(gaps) > 3 else None}
    return out


# ---------------------------------------------------------------- L11 — a cor
def _hsv(rgb):
    r, g, b = (rgb / 255.0).T
    mx = np.maximum(np.maximum(r, g), b)
    d = mx - np.minimum(np.minimum(r, g), b)
    h = np.zeros_like(mx)
    m = d > 1e-6
    rr = m & (mx == r)
    gg = m & (mx == g) & ~rr
    bb = m & ~rr & ~gg
    h[rr] = (60 * ((g[rr] - b[rr]) / d[rr])) % 360
    h[gg] = 60 * ((b[gg] - r[gg]) / d[gg]) + 120
    h[bb] = 60 * ((r[bb] - g[bb]) / d[bb]) + 240
    return h, np.where(mx > 0, d / np.maximum(mx, 1e-6), 0), mx


def cor(png):
    a, _, c = cobertura(png)
    out = {}
    for tr in tracos('gestos'):
        q, pr, s, t = R.densifica(tr['pts'])
        S, RGB, o = R.faixa(c, a, q, t, min(R.meia_janela(q, tr['size']), 0.4 * tr['size']), 0)
        nivel = np.percentile(S.max(1), 90)
        if nivel < 0.1:
            continue
        h, sa, v = (x.reshape(S.shape) for x in _hsv(RGB.reshape(-1, 3)))
        ok = (S > 0.5 * nivel) & (sa > 0.2)
        z = np.array([(pr[i], np.median((h[i][ok[i]] - H_AZUL + 180) % 360 - 180), np.median(sa[i][ok[i]]),
                       np.median(v[i][ok[i]]), np.std((h[i][ok[i]] - H_AZUL + 180) % 360 - 180))
                      for i in range(len(q)) if ok[i].sum() >= 3])
        if len(z) < 20:
            continue
        r = {'dH_p10_50_90': np.percentile(z[:, 1], [10, 50, 90]).round(1).tolist(), 'S_p50': float(np.median(z[:, 2])),
             'V_p50': float(np.median(z[:, 3])), 'atraves_sd': float(np.median(z[:, 4]))}
        if np.ptp(pr) > 0.3:
            r['por_decil'] = [(k / 10, round(float(np.median(z[sel, 1])), 1), round(float(np.median(z[sel, 2])), 3),
                               round(float(np.median(z[sel, 3])), 3))
                              for k in range(10) for sel in [(z[:, 0] >= k / 10) & (z[:, 0] < (k + 1) / 10 + (k == 9))] if sel.sum() > 3]
        out[tr['nome']] = r
    return out


# ---------------------------------------------------------------- L5 + L12 — o fim e a velocidade
def _serie(c, tr, ext):
    q, pr, s, t = R.densifica(tr['pts'])
    S, _, o = R.faixa(c, c[..., None].repeat(3, 2), q, t, max(6.0, float(tr['size'])), ext)
    nivel = np.percentile(S.max(1)[ext:ext + len(q)], 90)
    w = np.array([(lambda ys: ys[-1] - ys[0] + 1 if len(ys) else 0)(np.where(row >= 0.25 * nivel)[0]) for row in S], float)
    banda = np.abs(o) <= 0.35 * max(np.median(w[ext:ext + len(q)]), 2)
    return w, S[:, banda].mean(1), len(q)


def fim(gestos, tamanhos):
    out = {}
    _, _, c = cobertura(gestos)
    ts = {t['nome']: t for t in tracos('gestos')}
    for k in ('vertical', 'vertical_rapido'):
        w, cov, n = _serie(c, ts[k], 0)
        out[k] = (float(np.median(w[15:n - 15])), float(np.mean(cov[15:n - 15])))
    out['rapido_lento_largura'] = out['vertical_rapido'][0] / max(out['vertical'][0], 1e-9)
    out['rapido_lento_cobertura'] = out['vertical_rapido'][1] / max(out['vertical'][1], 1e-9)
    _, _, c = cobertura(tamanhos)
    tr = tracos('tamanhos', ['recto_40'])[0]
    ext = 60
    w, cov, n = _serie(c, tr, ext)
    core = cov[ext:ext + n]
    b = [core[i:i + 40].mean() for i in range(0, n - 39, 40)]
    c0 = np.mean(b[1:3])
    out['fall_fim_sobre_inicio'] = float(np.mean(b[-2:]) / c0) if c0 > 0.02 else float('nan')
    abaixo = [i * 40 + 20 for i, v in enumerate(b) if i >= 1 and v < 0.5 * c0]
    out['fall_s50_px'] = float(abaixo[0]) if abaixo else float('inf')
    wm = np.median(w[ext + n // 4: ext + n // 2])
    idx, tinta = np.where(w >= 0.8 * wm)[0], np.where(w > 0)[0]
    if len(idx) and len(tinta) and wm > 2:
        out['cauda_D'] = float((tinta[-1] - idx[-1]) / 40.0)
        out['cabeca_D'] = float((idx[0] - tinta[0]) / 40.0)
    return out


# ---------------------------------------------------------------- L8 — o tecto por traço
def tecto(png):
    _, _, c = cobertura(png)
    Ts = float(np.median(1 - c[85:96, 150:450]))
    Tm = float(np.percentile(1 - c[170:390, 690:910], 2))
    k = np.log(max(Tm, 1e-3)) / np.log(max(Ts, 1e-3)) if Ts < 0.97 else float('nan')
    return {'cobertura_recto': 1 - Ts, 'max_espiral': 1 - Tm, 'k': float(k)}


# ---------------------------------------------------------------- L6 — a textura
def _ac_raio(z, eixo):
    z = z - z.mean()
    n = z.shape[eixo]
    v = (z * z).mean()
    for k in range(1, n // 2):
        if (np.take(z, range(0, n - k), eixo) * np.take(z, range(k, n), eixo)).mean() / max(v, 1e-12) < 0.5:
            return k
    return n // 2


def textura(png):
    a, _, c = cobertura(png)
    out = {}
    for nome, H in (('recto_40', 40.0), ('recto_200', 200.0)):
        tr = tracos('tamanhos', [nome])[0]
        q, pr, s, t = R.densifica(tr['pts'])
        S, _, o = R.faixa(c, a, q, t, H, 0)
        nivel = np.percentile(S.max(1), 90)
        w = np.median([(lambda ys: ys[-1] - ys[0] + 1 if len(ys) else 0)(np.where(r >= 0.25 * nivel)[0]) for r in S])
        M = S[100:-100][:, np.abs(o) <= 0.3 * w]
        res = M - R.box(M, 2)
        out[nome] = {'largura': float(w), 'cobertura': float(M.mean()), 'buracos': float(np.mean(M < 0.3 * nivel)),
                     'residuo_pct': float(100 * res.std() / max(M.mean(), 1e-6)),
                     'escala_ao_longo_px': _ac_raio(M, 0), 'escala_atraves_px': _ac_raio(M, 1)}
    return out


# ---------------------------------------------------------------- fotos velhas
def velhas(d):
    rows = list(csv.DictReader(open(os.path.join(d, 'log.csv'))))
    seq = []
    for r in rows:
        f = os.path.join(d, r['TOOL'], f"{r['nome']}_{r['cena']}_A.png")
        if os.path.exists(f) and (not seq or seq[-1] != f):
            seq.append(f)
    out, prev, pa = [], None, None
    for f in seq:
        a = np.asarray(Image.open(f).convert('RGB'), np.int16)
        if pa is not None:
            m = int(np.abs(a - pa).max())
            if m <= 16 and 'CLONE' not in f:
                out.append((m, os.path.relpath(prev, d), os.path.relpath(f, d)))
        prev, pa = f, a
    return out


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        sys.exit(2)
    sub, args = sys.argv[1], sys.argv[2:]
    def p(x):
        print(json.dumps(x, ensure_ascii=False, default=float))
    if sub == 'modos':
        for png, m in zip(args[::2], args[1::2]):
            p({'png': png, **modos(png, m)})
    elif sub == 'fim':
        p(fim(args[0], args[1]))
    elif sub == 'velhas':
        for m, a, b in velhas(args[0]):
            print(f"máxΔ {m:3d}  {a} → {b}")
    else:
        fn = {'lamina': lamina, 'dabs': dabs, 'cor': cor, 'tecto': tecto, 'textura': textura}[sub]
        for png in args:
            p({'png': png, **fn(png)})


if __name__ == '__main__':
    main()
