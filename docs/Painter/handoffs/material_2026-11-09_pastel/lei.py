# Rodada 2 do Pastel (09/11): réguas da LEI (o papel decide a cobertura pela pressão) sobre as saídas de corre_lei.sh.
# Uso: python3 -I lei.py <scratch>  → imprime a tabela e grava <scratch>/lei.json. Reta da coluna vazia (y 110,
# x 100→700); na rampa a pressão do ponto x é 0,1 + 0,9·(x − 100)/600.
import json
import os
import sys

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from carrega import ph2d  # noqa: E402
from coerencia_lib import coer  # noqa: E402
from mede import mede  # noqa: E402

S = sys.argv[1] if len(sys.argv) > 1 else None
VARS = ['hoje', 'LH', 'H', 'SUB', 'LH_d05', 'LH_d09', 'LH_h06', 'LH_o08', 'LH_j05', 'LH_po03', 'LH_d09_po03', 'LH_g24',
        'H_d05', 'H_d09', 'H_h06', 'H_o08', 'H_j07', 'H_d09_o08', 'H_o08_h06',
        'H_d09_j07', 'H_d08_j07']
PS = (0.2, 0.35, 0.5, 0.65, 0.8, 0.95)
NUCLEO = slice(105, 116)  # o miolo da reta: ±5 px do eixo (cabe no traço mais fino da rampa)


def x_de_p(p):
    return int(round(100 + (p - 0.1) / 0.9 * 600))


def altura(s):
    """A ALTURA do papel do motor: a variante `papel` (LinearHeight a d = 1, sem mãos) pinta cobertura = f(t);
    devolvida em ranking 0..1 no rectângulo da reta (onde é monotónica)."""
    _, k = ph2d(s + '/cena', '__papel')
    j = k[60:160, 80:720].astype(np.float64)
    r = np.empty(j.size)
    r[np.argsort(j, axis=None, kind='stable')] = np.linspace(0, 1, j.size)
    h = np.full(k.shape, np.nan)
    h[60:160, 80:720] = r.reshape(j.shape)
    return h, k


def janela(a, p, meia=25):
    x = x_de_p(p)
    return a[NUCLEO, x - meia:x + meia]


def curva(k, h):
    """Por pressão: cobertura média, fracção pintada (k ≥ 0,05), corr. com a altura, e cobertura dos VALES
    (o quartil mais baixo do papel) e dos PICOS (o quartil mais alto)."""
    out = []
    for p in PS:
        kk, hh = janela(k, p).ravel(), janela(h, p).ravel()
        c = np.corrcoef(kk, hh)[0, 1] if kk.std() > 1e-6 else float('nan')
        out.append(dict(p=p, cob=round(float(kk.mean()), 3), pintada=round(float((kk >= 0.05).mean()), 3),
                        corr_altura=round(float(c), 2), vales=round(float(kk[hh <= np.quantile(hh, 0.25)].mean()), 3),
                        picos=round(float(kk[hh >= np.quantile(hh, 0.75)].mean()), 3)))
    return out


def corr_miolo(k, h):
    kk, hh = k[NUCLEO, 150:650].ravel(), h[NUCLEO, 150:650].ravel()
    return round(float(np.corrcoef(kk, hh)[0, 1]), 2) if kk.std() > 1e-6 else None


def tudo(s):
    h, _ = altura(s)
    res = {}
    for v in VARS:
        _, k1 = ph2d(s + '/cena', f'__{v}')
        _, kv = ph2d(s + '/cena_volta', f'__{v}')
        _, kr = ph2d(s + '/cena', f'__{v}__rampa')
        _, kr2 = ph2d(s + '/cena', f'__{v}__rampa__x2')
        res[v] = dict(p1=dict(**mede(k1), coer=coer(k1, kv), corr_altura=corr_miolo(k1, h)),
                      rampa=curva(kr, h), rampa_x2=curva(kr2, h))
    return res, h


if __name__ == '__main__':
    res, h = tudo(S)
    json.dump(res, open(S + '/lei.json', 'w'), indent=1)
    print('variante | larg borda cob cv buracos soltos grão coer corrH | cobertura×pressão (p .2 .35 .5 .65 .8 .95)')
    for v, r in res.items():
        m = r['p1']
        print(f"{v:12s} {m['larg50']:5} {m['borda']:5} {m['cob_miolo']:5} {m['cv_miolo']:5} {m['vazio_miolo']:5} "
              f"{m['quase_soltos']:5} {m['grao_x']}x{m['grao_y']} {m['coer']} {m['corr_altura']}")
        for nome in ('rampa', 'rampa_x2'):
            c = r[nome]
            print(f"   {nome:8s} cob {[x['cob'] for x in c]} pint {[x['pintada'] for x in c]}")
            print(f"            corrH {[x['corr_altura'] for x in c]} vales {[x['vales'] for x in c]} picos {[x['picos'] for x in c]}")
