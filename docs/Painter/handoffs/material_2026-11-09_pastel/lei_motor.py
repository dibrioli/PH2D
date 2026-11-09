# Rodadas 3–4 do Pastel (09/11): réguas das peças de MOTOR (softness · tip_depth) e do grão sobre as saídas de
# corre_lei.sh. A ALTURA de cada variante é a do SEU papel (a variante papel_<tex> com a mesma textura).
# Uso: TSV=<var_pastel_motor*.tsv> python3 -I lei_motor.py <scratch>  → tabela + <scratch>/lei_motor<N>.json
import json
import os
import sys

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from carrega import ph2d  # noqa: E402
from coerencia_lib import coer  # noqa: E402
from krita_rampa import k_krita  # noqa: E402
from lei import NUCLEO, PS, corr_miolo, curva, janela, x_de_p  # noqa: E402
from mede import mede  # noqa: E402

AQUI = os.path.dirname(os.path.abspath(__file__))
TSV = os.environ.get('TSV', os.path.join(AQUI, 'var_pastel_motor.tsv'))
LINHAS = [l.rstrip('\n').split('\t') for l in open(TSV) if l.strip() and not l.startswith('#')]


def chave_tex(campos):
    t = next((c[len('texture='):] for c in campos if c.startswith('texture=')), None)
    if t is None:
        return None
    j = json.loads(t)
    return (j['kind'], tuple(j['size']), tuple(j['params']))


PAPEIS = {chave_tex(c): c[1] for c in LINHAS if c[1].startswith('papel_')}
VARS = [(c[1], PAPEIS.get(chave_tex(c))) for c in LINHAS if not c[1].startswith('papel_')]


def altura(s, papel):
    _, k = ph2d(s + '/cena', f'__{papel}')
    j = k[60:160, 80:720].astype(np.float64)
    r = np.empty(j.size)
    r[np.argsort(j, axis=None, kind='stable')] = np.linspace(0, 1, j.size)
    h = np.full(k.shape, np.nan)
    h[60:160, 80:720] = r.reshape(j.shape)
    return h


def parciais(k, p):
    """Dos píxeis com tinta no núcleo à pressão p, a fracção PARCIAL: abaixo de 85 % do alfa máximo da janela (o
    percentil 95) — o grão de borda macia. A opacidade pela pressão escala tudo e não conta."""
    kk = janela(k, p).ravel()
    t = kk[kk >= 0.05]
    if len(t) == 0:
        return None
    return round(float((t < 0.85 * np.quantile(t, 0.95)).mean()), 3)


def enche(k1, k2, p):
    """A 2.ª passada: dos píxeis VAZIOS na 1.ª (k < 0,05), a fracção que ganha tinta."""
    a, b = janela(k1, p).ravel(), janela(k2, p).ravel()
    return round(float(((a < 0.05) & (b >= 0.05)).sum() / max((a < 0.05).sum(), 1)), 3)


def grao_meio(k, p):
    """O tamanho do grão à pressão p: o passo em x (px) em que a autocorrelação do resíduo cai a 1/e, no núcleo."""
    x = x_de_p(p)
    j = k[NUCLEO, x - 40:x + 40].astype(np.float64)
    r = j - j.mean()
    if r.std() < 1e-6:
        return None
    ac = np.fft.irfft(np.abs(np.fft.rfft(r, axis=1)) ** 2, axis=1).mean(0)
    ac = ac / ac[0]
    return next((i for i in range(1, len(ac) // 2) if ac[i] < 1 / np.e), len(ac) // 2)


def borda_quebrada(k):
    """À pressão 1, na reta (x 150–650): o desvio-padrão (px) da posição da borda de cima e de baixo (o primeiro píxel
    ≥ 50 % do pico, vindo de fora), sem a tendência linear — 0 = borda lisa."""
    j = k[65:155, 150:650].astype(np.float64)
    lim = 0.5 * np.quantile(j, 0.99)
    out = []
    for lado in (j, j[::-1]):
        pos = np.array([np.argmax(c >= lim) if (c >= lim).any() else np.nan for c in lado.T])
        x = np.arange(len(pos))
        ok = ~np.isnan(pos)
        res = pos[ok] - np.polyval(np.polyfit(x[ok], pos[ok], 1), x[ok])
        out.append(res.std())
    return round(float(np.mean(out)), 2)


def extra(k1, kr, kr2):
    return dict(parciais={p: parciais(kr, p) for p in (0.35, 0.5, 0.65)},
                enche={p: enche(kr, kr2, p) for p in (0.35, 0.5, 0.65)},
                grao={p: grao_meio(kr, p) for p in (0.5, 0.65)}, quebra=borda_quebrada(k1))


def tudo(s):
    hs = {pp: altura(s, pp) for pp in set(PAPEIS.values())}
    res = {}
    for v, pp in VARS:
        _, k1 = ph2d(s + '/cena', f'__{v}')
        _, kv = ph2d(s + '/cena_volta', f'__{v}')
        _, kr = ph2d(s + '/cena', f'__{v}__rampa')
        _, kr2 = ph2d(s + '/cena', f'__{v}__rampa__x2')
        h = hs.get(pp)
        p1 = dict(**mede(k1), coer=coer(k1, kv))
        if h is not None:
            p1['corr_altura'] = corr_miolo(k1, h)
        res[v] = dict(p1=p1, **extra(k1, kr, kr2),
                      rampa=curva(kr, h) if h is not None else [dict(p=p, cob=round(float(janela(kr, p).mean()), 3))
                                                                for p in PS],
                      rampa_x2=[dict(p=p, cob=round(float(janela(kr2, p).mean()), 3)) for p in PS])
    k1 = k_krita(f'{s}/krita_rampa/h__Chalk_Soft_rampa.png')
    k2 = k_krita(f'{s}/krita_rampa_x2/h__Chalk_Soft_rampa_x2.png')
    kp = k_krita(f'{s}/krita_cru/h__Chalk_Soft_v3.png')
    res['krita'] = dict(p1=mede(kp), **extra(kp, k1, k2),
                        rampa=[dict(p=p, cob=round(float(janela(k1, p).mean()), 3)) for p in PS],
                        rampa_x2=[dict(p=p, cob=round(float(janela(k2, p).mean()), 3)) for p in PS])
    return res


if __name__ == '__main__':
    S = sys.argv[1]
    res = tudo(S)
    json.dump(res, open(S + '/' + os.path.basename(TSV).replace('var_pastel_', 'lei_').replace('.tsv', '.json'), 'w'),
              indent=1, default=str)
    print('variante      larg borda quebra  cob   cv  buracos coer corrH | parciais .35/.5/.65 | grão .5/.65 | enche .35/.5/.65')
    for v, r in res.items():
        m = r['p1']
        print(f"{v:13s} {m['larg50']:5} {m['borda']:5} {r['quebra']:5} {m['cob_miolo']:5} {m['cv_miolo']:5} "
              f"{m['vazio_miolo']:6} {m.get('coer', '—')} {m.get('corr_altura', '—')} | {list(r['parciais'].values())} | "
              f"{list(r['grao'].values())} | {list(r['enche'].values())}")
        print(f"   rampa    cob {[x['cob'] for x in r['rampa']]}")
        print(f"   rampa_x2 cob {[x['cob'] for x in r['rampa_x2']]}")
