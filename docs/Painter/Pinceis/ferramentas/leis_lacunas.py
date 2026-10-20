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

As fotos NOVAS (cenas de `bateria_lacunas.json`, 06 §10):
  memoria  <png memoria>…   L3: largura e arranque de cada traço (v1/h2 SEM SET_BRUSH × v2/h3 controlo)
  canto    <png canto>…     L3 atraso: largura depois de cada vértice ÷ a do meio da perna
  efeito   <png efeito>…    L7: o α por passagem e o erro de cada fórmula sobre cinza e pastel, 1× e 10×
  velocidade <png>…         L12: largura e cobertura de 3 a 50 px/evento ÷ as do recto de 3 px
  carimbos · carimbos_3d <png>…  L19/L4: por toque — área, manchas, desvio, cor e FORMA (quantas distintas)
  fundo    <png fundo>…     L11: fracção da tinta que é a 2.ª cor (Krita, fundo amarelo)

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


# ================================================================ as fotos NOVAS (bateria_lacunas.json, 06 §10)
_BATL = []


def batl():
    if not _BATL:
        _BATL.append(json.load(open(os.path.join(AQUI, 'bateria_lacunas.json'))))
    return _BATL[0]


def tracos_l(cena, nomes=None):
    return [t for t in R.tracos(batl(), cena) if t['quem'] == 'preset' and (not nomes or t['nome'] in nomes)]


def memoria(png):
    """L3: largura mediana e ARRANQUE (1.os 15 px ÷ mediana) de cada traço; v1/h2 vieram SEM SET_BRUSH."""
    a, _, c = cobertura(png)
    out = {}
    for tr in tracos_l('memoria'):
        w = larguras(c, a, tr)[0]
        m = len(w)
        w50 = float(np.median(w[m // 10: m - m // 10]))
        out[tr['nome']] = {'w50': w50, 'arranque': float(np.median(w[:15]) / w50) if w50 > 0 else float('nan'),
                           'fim': float(np.median(w[-15:]) / w50) if w50 > 0 else float('nan')}
    return out


def canto(png):
    """L3 atraso: largura por distância depois de cada vértice ÷ a do meio da perna (d = 30 px no zigue-zague)."""
    a, _, c = cobertura(png)
    out = {}
    for tr in tracos_l('canto'):
        w, th, s, pr = larguras(c, a, tr, H=20.0)
        xy = tr['pts'][:, :2]
        d = np.hypot(*np.diff(xy, axis=0).T)
        vira = np.where(np.abs(np.diff(np.degrees(np.arctan2(*np.diff(xy, axis=0).T[::-1])) % 360)) > 30)[0]
        s_v = np.concatenate([[0.0], np.cumsum(d)])[vira + 1]
        ext = np.concatenate([[0.0], s_v, [s[-1]]])
        pernas = []
        for i in range(1, len(ext) - 1):
            ini, fim = ext[i], ext[i + 1]
            meio = np.median(w[(s > ini + 0.4 * (fim - ini)) & (s < ini + 0.6 * (fim - ini))])
            prof = [float(np.median(w[(s >= ini + k - 1) & (s <= ini + k + 1)]) / meio) for k in (2, 5, 10, 15, 20, 25)
                    if ini + k + 1 < fim]
            pernas.append({'w_antes': float(np.median(w[(s > ext[i - 1] + 0.4 * (ini - ext[i - 1])) & (s < ini - 0.4 * (ini - ext[i - 1]))])),
                           'w_meio': float(meio), 'perfil_2_5_10_15_20_25': [round(x, 2) for x in prof]})
        out[tr['nome']] = pernas
    return out


def _passa(fn, b, s, al, n):
    for _ in range(n):
        b = b + al * (fn(b, s) - b)
    return b


def efeito(png):
    """L7: base CINZA (y 150) e PASTEL (y 470) a x 375; o efeito 1× (x 250) e 10× (x 500). Um α por passagem,
    ajustado em conjunto aos 4 pontos (2 bases × 1×/10×) pela fórmula iterada; papel (y 310) × NORMAL."""
    a = np.asarray(Image.open(png).convert('RGB'), float) / 255.0
    pts = []
    for y in (150, 470):
        base = _caixa(a, 375, y, 30, 8)
        for x, n in ((250, 1), (500, 10)):
            pts.append((base, _caixa(a, x, y, 5, 8), n))
    papel = _caixa(a, 375, 310, 30, 8)
    p1, p10 = _caixa(a, 250, 310, 5, 20), _caixa(a, 500, 310, 5, 20)
    grade = np.concatenate([np.arange(0.0, 0.1, 0.0005), np.arange(0.1, 1.0001, 0.005)])
    res = []
    for esp, f, g in (('sRGB', lambda x: x, lambda x: x), ('lin', _lin, _enc)):
        for m, fn in MODOS.items():
            h = (lambda b, s, fn=fn: g(fn(f(b), f(s))))
            best = min((max(float(np.abs(_passa(h, b, AZUL, al, n) - C).max()) for b, C, n in pts), al) for al in grade)
            res.append((best[0] * 255, esp, m, best[1]))
    res.sort()
    dE = [float(np.abs(C - b).max() * 255) for b, C, n in pts]
    alfa_p = lambda C, n: 1 - (1 - _alfa(papel, C, AZUL)) ** (1 / n) if _alfa(papel, C, AZUL) < 1 else 1.0
    return {'base_cinza': (pts[0][0] * 255).round().tolist(), 'base_pastel': (pts[2][0] * 255).round().tolist(),
            'cinza_1x_10x': [(pts[0][1] * 255).round().tolist(), (pts[1][1] * 255).round().tolist()],
            'pastel_1x_10x': [(pts[2][1] * 255).round().tolist(), (pts[3][1] * 255).round().tolist()],
            'dE_cinza_1x_10x_pastel_1x_10x': [round(x, 1) for x in dE],
            'alfa_papel_por_passagem_1x_10x': [round(alfa_p(p1, 1), 4), round(alfa_p(p10, 10), 4)],
            'melhores': [(round(e, 2), esp, m, round(al, 4)) for e, esp, m, al in res[:6]],
            'por_modo_sRGB': {m: round(e, 2) for e, esp, m, al in res if esp == 'sRGB'}}


def velocidade(png):
    """L12: largura mediana e cobertura da banda central de cada recto (3…50 px/evento) ÷ a do recto de 3 px."""
    _, _, c = cobertura(png)
    out = []
    for tr in tracos_l('velocidade'):
        w, cov, n = _serie(c, tr, 0)
        core = slice(60, n - 60)
        out.append((tr['nome'], float(np.median(w[core])), float(np.mean(cov[core]))))
    w0, c0 = out[0][1], out[0][2]
    return {nm: {'w': w, 'w_rel': round(w / w0, 3) if w0 else None, 'cob_rel': round(cv / c0, 3) if c0 else None}
            for nm, w, cv in out}


def carimbos(png, cena='carimbos'):
    """L4/L19: cada TOQUE é um carimbo (ou uma nuvem de emissão). Por toque: área, nº de manchas, desvio do centro
    ÷ D, a cor (ΔH, S, V) e a FORMA (máscara 24×24 centrada); formas agrupadas por correlação ≥ 0,9."""
    a, _, c = cobertura(png)
    ts = tracos_l(cena)
    xs = sorted({float(t['pts'][0][0]) for t in ts})
    meia = 0.5 * (xs[1] - xs[0]) if len(xs) > 1 else 30.0
    linhas = {}
    for t in ts:
        x, y, p = t['pts'][0]
        D = t['size']
        x0, x1, y0, y1 = int(x - meia), int(x + meia), int(max(0, y - 2.4 * D)), int(min(c.shape[0], y + 2.4 * D))
        z = c[y0:y1, x0:x1]
        m = z > 0.12
        if m.sum() < 3:
            linhas.setdefault(p, []).append(None)
            continue
        lab, n = _rotula(m)
        area = np.bincount(lab.ravel())[1:]
        yy, xx = np.indices(z.shape)
        cx, cy = (xx * z).sum() / z.sum() + x0, (yy * z).sum() / z.sum() + y0
        ys_, xs_ = np.nonzero(m)
        caixa = z[ys_.min():ys_.max() + 1, xs_.min():xs_.max() + 1]
        forma = np.asarray(Image.fromarray((caixa * 255).astype(np.uint8)).resize((24, 24), Image.BILINEAR), float)
        h, sa, v = _hsv(a[y0:y1, x0:x1][m] * 1.0)
        ok = sa > 0.2
        dh = float(np.median((h[ok] - H_AZUL + 180) % 360 - 180)) if ok.sum() > 3 else float('nan')
        linhas.setdefault(p, []).append({'area': int(m.sum()), 'manchas': int((area >= 3).sum()),
                                         'dx_D': float((cx - x) / D), 'dy_D': float((cy - y) / D),
                                         'larg': int(np.ptp(xs_) + 1), 'alt': int(np.ptp(ys_) + 1),
                                         'dH': dh, 'S': float(np.median(sa)), 'V': float(np.median(v)),
                                         'cobertura_media': float(z[m].mean()), '_f': forma})
    grupos, out = [], {}
    for p, L in sorted(linhas.items(), reverse=True):
        seq = []
        for e in L:
            if e is None:
                seq.append('-')
                continue
            f = e.pop('_f')
            f = (f - f.mean()) / (f.std() + 1e-9)
            k = next((i for i, g in enumerate(grupos) if float((f * g).mean()) >= 0.9), None)
            if k is None:
                grupos.append(f)
                k = len(grupos) - 1
            seq.append(chr(65 + k) if k < 26 else '+')
        v = [e for e in L if e]
        q = lambda key: np.percentile([e[key] for e in v], [10, 50, 90]).round(2).tolist() if v else None
        out[f'pressao_{p}'] = {'toques': len(L), 'com_tinta': len(v), 'formas_seq': ''.join(seq),
                               'area_p10_50_90': q('area'), 'manchas_p10_50_90': q('manchas'),
                               'larg_p10_50_90': q('larg'), 'alt_p10_50_90': q('alt'),
                               'dx_D_sd': float(np.std([e['dx_D'] for e in v])) if v else None,
                               'dy_D_sd': float(np.std([e['dy_D'] for e in v])) if v else None,
                               'dH_p10_50_90': q('dH'), 'V_p10_50_90': q('V'), 'cob_p10_50_90': q('cobertura_media')}
    out['formas_distintas'] = len(grupos)
    return out


def _nao_azul(rgb):
    """Fracção dos px SATURADOS (S > 0,25) cuja matiz cai fora do azul (30°–170°: amarelo, laranja, verde = a
    mistura frente×fundo em RGB). O pálido da borda fica azul na matiz — a distância RGB enganava-se nele."""
    h, sa, v = _hsv(rgb)
    ok = sa > 0.25
    return float(np.mean((h[ok] > 30) & (h[ok] < 170))) if ok.sum() > 5 else float('nan'), int(ok.sum())


def fundo(png):
    """L11 (Krita, frente AZUL, fundo AMARELO): fracção da tinta saturada com matiz de AMARELO/VERDE (não-azul), na
    cena e por decil da pressão na rampa_0_1. > 0 ⇒ o preset usa a 2.ª cor (ou sorteia a matiz: ver o ΔH)."""
    a, _, c = cobertura(png)
    m = c > 0.1
    if m.sum() < 20:
        return {'tinta_px': int(m.sum())}
    fr, n = _nao_azul(a[m])
    out = {'tinta_px': int(m.sum()), 'saturados': n, 'frac_nao_azul': round(fr, 4)}
    tr = tracos_l('fundo', ['rampa_0_1'])[0]
    q, pr, s, t = R.densifica(tr['pts'])
    Sx, RGB, o = R.faixa(c, a, q, t, 0.4 * tr['size'], 0)
    dec = []
    for k in range(10):
        sel = (pr >= k / 10) & (pr < (k + 1) / 10 + (k == 9))
        mm = Sx[sel] > 0.1
        if mm.sum() > 5:
            f, n = _nao_azul(RGB[sel][mm])
            if n > 5:
                dec.append((k / 10, round(f, 3)))
    out['rampa_nao_azul_por_decil'] = dec
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
        fn = {'lamina': lamina, 'dabs': dabs, 'cor': cor, 'tecto': tecto, 'textura': textura, 'memoria': memoria,
              'canto': canto, 'efeito': efeito, 'velocidade': velocidade, 'carimbos': carimbos,
              'carimbos_3d': lambda f: carimbos(f, 'carimbos_3d'), 'fundo': fundo}[sub]
        for png in args:
            p({'png': png, **fn(png)})


if __name__ == '__main__':
    main()
