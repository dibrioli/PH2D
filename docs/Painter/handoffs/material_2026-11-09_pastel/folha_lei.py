# Rodada 2 do Pastel (09/11): simulação do ENCHIMENTO do dente + folha_variantes.png. Uso: python3 -I folha_lei.py <scratch> <saída>
# Enchimento: na 2.ª passada a altura efectiva sobe com a tinta já pousada, h_ef = h + k·alfa_1; a lei de cada passada é
# a do Height medido na 1.ª (pinta E onde h_ef ≥ limiar; E = o máximo local da 1.ª, limiar por bloco de 10 px).
import json
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from carrega import COL, base_rgb, compoe_camada, krita, ph2d  # noqa: E402
from krita_rampa import k_krita  # noqa: E402
from lei import NUCLEO, PS, altura, curva  # noqa: E402
from simula import gauss  # noqa: E402

S, OUT = sys.argv[1], sys.argv[2]
MELHOR = 'H_d08_j07'
Y0, Y1, X0, X1 = 60, 160, 80, 720


def enche(k1, h, kc, viz=0.0):
    """`viz` > 0: a tinta pousada enche o dente À VOLTA (alfa desfocado `viz` px), não só no próprio píxel."""
    a, hh = k1[Y0:Y1, X0:X1], h[Y0:Y1, X0:X1]
    al = np.clip(gauss(a, viz), 0, 1) if viz > 0 else a
    E = np.lib.stride_tricks.sliding_window_view(np.pad(a, 3, mode='edge'), (7, 7)).max((2, 3))
    k2 = np.zeros_like(a)
    for x in range(0, a.shape[1], 10):
        nuc = a[NUCLEO.start - Y0:NUCLEO.stop - Y0, x:x + 10]
        fr = float((nuc >= 0.05).mean())
        if fr <= 0:
            continue
        thr = np.quantile(hh[NUCLEO.start - Y0:NUCLEO.stop - Y0, x:x + 10], 1 - fr)
        k2[:, x:x + 10] = E[:, x:x + 10] * (hh[:, x:x + 10] + kc * al[:, x:x + 10] >= thr)
    out = k1.copy()
    out[Y0:Y1, X0:X1] = 1 - (1 - a) * (1 - k2)
    return out


h, _ = altura(S)
_, kr = ph2d(S + '/cena', f'__{MELHOR}__rampa')
_, kr2 = ph2d(S + '/cena', f'__{MELHOR}__rampa__x2')
sims = {kc: enche(kr, h, kc) for kc in (0.0, 0.5, 1.0)}
simv = {kc: enche(kr, h, kc, 1.5) for kc in (0.5, 1.0)}
tab = {'motor 1 passada': curva(kr, h), 'motor 2 passadas': curva(kr2, h)}
for kc, s in sims.items():
    tab[f'simulação k={kc}' + (' (controlo)' if kc == 0 else '')] = curva(s, h)
for kc, s in simv.items():
    tab[f'sim. vizinhança k={kc}'] = curva(s, h)
print('enchimento —', MELHOR, '(reta, núcleo ±5 px; p =', PS, ')')
for n, c in tab.items():
    print(f"{n:28s} cob {[x['cob'] for x in c]}\n{'':28s} pint {[x['pintada'] for x in c]}  vales {[x['vales'] for x in c]}")
json.dump(tab, open(S + '/enchimento.json', 'w'), indent=1)

# ── FOLHA ──
base = base_rgb(S + '/cena')
fonte = ImageFont.truetype('/usr/share/fonts/TTF/DejaVuSans.ttf', 13)
fonte_g = ImageFont.truetype('/usr/share/fonts/TTF/DejaVuSans-Bold.ttf', 15)


def cel(rgb):
    return Image.fromarray(rgb.clip(0, 255).astype(np.uint8)).resize((400, 440), Image.BICUBIC)


def sobre_branco(k):
    return compoe_camada(k, np.full(k.shape + (3,), 255, np.float32))


def linha_ph2d(suf):
    rgb, _ = ph2d(S + '/cena', suf)
    _, a = ph2d(S + '/cena', suf + '__rampa')
    _, b = ph2d(S + '/cena', suf + '__rampa__x2')
    return [rgb[:, :COL], rgb[:, COL:2 * COL], rgb[:, 2 * COL:], sobre_branco(a), sobre_branco(b), rgb[Y0:Y1, 250:550]]


kb = k_krita(f'{S}/krita_cru/h__Chalk_Soft_v3.png')
krgb = compoe_camada(np.concatenate([kb] * 3, 1), base)
L = [('PH2D hoje', 'Pastel de fábrica (spray 20, grão 2 px)', linha_ph2d('__hoje')),
     ('Krita Chalk Soft', 'Krita 6.0.4.1 · camada própria (col. b)', [krgb[:, :COL], krgb[:, COL:2 * COL], krgb[:, 2 * COL:],
      sobre_branco(k_krita(f'{S}/krita_rampa/h__Chalk_Soft_rampa.png')),
      sobre_branco(k_krita(f'{S}/krita_rampa_x2/h__Chalk_Soft_rampa_x2.png')), krgb[Y0:Y1, 250:550]])]
for v, sub in (('H_d08_j07', 'Height · d 0,8 · jitter 0,7'), ('H_d09_j07', 'Height · d 0,9 · jitter 0,7'),
               ('H_j07', 'Height · d 0,7 · jitter 0,7'), ('LH_d09', 'LinearHeight · d 0,9 · jitter 1')):
    L.append((f'variante {v}', sub + ' · dureza 0,85 Smooth · opac. 1 · sem spray', linha_ph2d(f'__{v}')))
s05, s1 = sobre_branco(simv[0.5]), sobre_branco(simv[1.0])
L.append(('Simulação enchimento', f'sobre {MELHOR}, alfa da vizinhança 1,5 px: col. 1–3 = motor 2 passadas · sim k 0,5 · sim k 1 (rampa); '
          '«rampa» = motor 1 passada; «2 passadas» = sim k 1',
          [sobre_branco(kr2)[:, :COL], s05[:, :COL], s1[:, :COL], sobre_branco(kr), s1, s1[Y0:Y1, 250:550]]))
COLS = ('vazio · p 1', 'branco · p 1', 'blocos · p 1', 'rampa 0,1→1 (vazio)', '2 passadas na rampa', 'detalhe 200 % (reta, p 1)')
LW, CW, DW, TOPO = 230, 400, 600, 22
lin_h = 440 + TOPO + 14
fol = Image.new('RGB', (LW + 5 * (CW + 8) + DW + 10, 60 + lin_h * len(L)), (36, 36, 40))
d = ImageDraw.Draw(fol)
d.text((10, 10), 'Pastel — rodada 2 (09/11): o papel decide a cobertura pela pressão · nosso hoje × Krita Chalk Soft × '
       'variantes (só receita) × simulação do enchimento', font=fonte_g, fill=(255, 255, 255))
d.text((10, 34), 'rampa: a pressão sobe 0,1→1,0 ao longo de cada traço (esquerda→direita); 2 passadas: a cena repetida por '
       'cima · traço v3, verde 00A03C, 40 px · números: README.md', font=fonte, fill=(200, 200, 200))
for i, (rot, sub, cels) in enumerate(L):
    y = 60 + i * lin_h
    d.text((10, y + TOPO + 10), rot, font=fonte_g, fill=(255, 210, 120) if 'Sim' in rot else (255, 255, 255))
    yy = y + TOPO + 34
    for j in range(0, len(sub), 28):
        d.text((10, yy), sub[j:j + 28], font=fonte, fill=(200, 200, 200))
        yy += 17
    for c, im in enumerate(cels):
        x = LW + c * (CW + 8)
        if c < 5:
            fol.paste(cel(im), (x, y + TOPO))
        else:
            fol.paste(Image.fromarray(im.clip(0, 255).astype(np.uint8)).resize((DW, 200), Image.NEAREST), (x, y + TOPO))
        d.text((x, y + 4), COLS[c], font=fonte, fill=(170, 200, 255))
fol.save(f'{OUT}/folha_variantes.png', optimize=True)
print('folha ok')
