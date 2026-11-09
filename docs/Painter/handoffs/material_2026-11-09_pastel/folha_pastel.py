# Folha do Pastel (09/11): linhas = NOSSO Pastel · CSP · Krita · MyPaint · simulações; colunas = vazio · branco · blocos
# (metade do tamanho) + o detalhe a 200 % da reta na coluna vazia + as réguas. Grava folha_pastel.png e metricas.json.
# Uso: python3 -I folha_pastel.py <scratch: cena/, cena_volta/, krita_cru/, krita_volta/, mp_raw/> <saída>
import json
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from carrega import COL, H, W, base_rgb, compoe_camada, csp, k_de_rgb_sobre_branco, krita, mypaint, ph2d  # noqa: E402
from coerencia_lib import coer  # noqa: E402
from mede import corr_papel, mede  # noqa: E402
from simula import papel_sintetico, sim1, sim2, sim3  # noqa: E402

S, OUT = sys.argv[1], sys.argv[2]
FUNDOS = ('vazio', 'branco', 'blocos')
fonte = ImageFont.truetype('/usr/share/fonts/TTF/DejaVuSans.ttf', 13)
fonte_g = ImageFont.truetype('/usr/share/fonts/TTF/DejaVuSans-Bold.ttf', 15)

base = base_rgb(S + '/cena')
papel = papel_sintetico(H, W)
nosso_rgb, k_nosso = ph2d(S + '/cena')
_, k_nosso_v = ph2d(S + '/cena_volta')
csp_cheia, csp_meia, k_csp_meia = csp()

# (rótulo, subtítulo, rgb 2400x880, k da coluna vazia, escala, nota, coerência ida×volta)
linhas = [('PH2D (nosso)', 'PH2D 8197021 · Pastel (Digital) · 40 px · tela lisa', nosso_rgb, k_nosso, 1.0, '',
           coer(k_nosso, k_nosso_v)),
          ('Clip Studio', 'CSP 1.13.2 PRO aval. · «Pastel» 40 px · foto de 08/11 (só sobrou a 50 %)', csp_cheia,
           k_csp_meia, 0.5, 'meia', None)]
for n, nome in (('Chalk_Grainy', 'h) Chalk Grainy'), ('Chalk_Soft', 'h) Chalk Soft'), ('Chalk_Details', 'h) Chalk Details')):
    im, _ = krita(f'{S}/krita_cru/h__{n}_v3.png', base)
    k_b = k_de_rgb_sobre_branco(im[:, COL:2 * COL])  # a coluna a do Krita perdeu-se: a b é o mesmo traço numa camada
    k3 = np.concatenate([k_b, k_b, k_b], 1)
    vp = f'{S}/krita_volta/h__{n}_volta.png'
    c = coer(k_b, k_de_rgb_sobre_branco(np.asarray(Image.open(vp).convert('RGB'))[:, COL:2 * COL].astype(np.float32))) \
        if os.path.exists(vp) else None
    linhas.append(('Krita', f'Krita 6.0.4.1 · {nome} · 40 px · camada própria', compoe_camada(k3, base), k_b, 1.0, '', c))
for n, nome in (('ramon__Pastel_1', 'ramon/Pastel_1'), ('deevad__chalk', 'deevad/chalk')):
    im, k_a = mypaint(f'{S}/mp_raw/{n}_12.npy', base)
    _, k_v = mypaint(f'{S}/mp_raw/{n}_12_volta.npy', base)
    linhas.append(('MyPaint', f'libmypaint 1.6.1 · {nome}.myb · size 12 (≈40 px) · camada própria', im, k_a, 1.0, '',
                   coer(k_a, k_v)))
for suf, rot, sub in (('__sem_spray', 'PH2D variante A', 'PH2D 8197021 · Pastel SEM o spray (motor real, '
                                                       'PH2D_V3_VARIANTES) · grão do preset 0,6'),
                      ('__sem_spray_grao1', 'PH2D variante B', 'PH2D 8197021 · Pastel SEM o spray, grain_depth 1,0 '
                                                               '(motor real) · tela lisa')):
    rgb, k = ph2d(S + '/cena', suf)
    _, kv = ph2d(S + '/cena_volta', suf)
    linhas.append((rot, sub, rgb, k, 1.0, 'var', coer(k, kv)))
SIMS = (('Simulação 1', 'o papel decide ONDE (a MESMA tinta do nosso, só nos picos)', sim1),
        ('Simulação 2', 'corpo contínuo + grão do papel (mais tinta, à Chalk Soft)', sim2),
        ('Simulação 3', 'CONTROLO: o mesmo spray com grão de 1,2 px, sem papel', sim3))
for rot, sub, f in SIMS:
    k3 = np.concatenate([f(k_nosso, papel[:, c * COL:(c + 1) * COL]) for c in range(3)], 1)  # o papel é da tela
    s_a = k3[:, :COL]
    s_v = f(k_nosso_v, papel[:, :COL])
    linhas.append((rot, f'{sub} · papel sintético Cold Press', compoe_camada(k3, base), s_a, 1.0, 'sim', coer(s_a, s_v)))

# ── RÉGUAS ──
med = []
for rot, sub, rgb, k, e, nota, c in linhas:
    m = dict(linha=rot, quem=sub, **mede(k, e), coerencia_ida_volta=c)
    if e == 1.0:  # a mesma régua à escala do CSP (meia, bicúbica) para comparar com ele
        km = np.asarray(Image.fromarray((k[:, :COL] * 255).astype(np.uint8)).resize((400, 440), Image.BICUBIC)) / 255
        m['a_50pc'] = mede(km, 0.5)
        m['corr_papel_sintetico'] = corr_papel(k[:, :COL], papel[:, :COL])
    med.append(m)
json.dump(med, open(f'{OUT}/metricas.json', 'w'), ensure_ascii=False, indent=1)

# ── FOLHA ──
CW, CH, LW, DW, TOPO = 400, 440, 250, 600, 22
lin_h = CH + TOPO + 14
fol = Image.new('RGB', (LW + 3 * (CW + 8) + DW + 10, 60 + lin_h * len(linhas)), (36, 36, 40))
d = ImageDraw.Draw(fol)
d.text((10, 10), 'Pastel — o NOSSO × Clip Studio · Krita · MyPaint × simulações do conserto   (09/11; traço v3: '
       'reta · S · zigue-zague · toque, verde 00A03C, pressão 1; células a 50 %, detalhe a 200 %)', font=fonte_g,
       fill=(255, 255, 255))
d.text((10, 34), 'colunas: (a) camada VAZIA sobre o papel branco · (b) branco opaco · (c) blocos opacos · '
       'detalhe: a reta da coluna (a), x 250–550 · réguas: README.md', font=fonte, fill=(200, 200, 200))
for i, (rot, sub, rgb, k, e, nota, c) in enumerate(linhas):
    y = 60 + i * lin_h
    d.text((10, y + TOPO + 10), rot, font=fonte_g, fill=(255, 210, 120) if nota == 'sim' else (255, 255, 255))
    yy = y + TOPO + 34
    for pedaco in [sub[j:j + 30] for j in range(0, len(sub), 30)]:
        d.text((10, yy), pedaco, font=fonte, fill=(200, 200, 200))
        yy += 17
    for col in range(3):
        if nota == 'meia':
            cel = csp_meia[:, col * 400:(col + 1) * 400]
        else:
            cel = np.asarray(Image.fromarray(rgb[:, col * COL:(col + 1) * COL].clip(0, 255).astype(np.uint8))
                             .resize((CW, CH), Image.BICUBIC))
        x = LW + col * (CW + 8)
        fol.paste(Image.fromarray(cel.clip(0, 255).astype(np.uint8)), (x, y + TOPO))
        cab = rot if nota == 'sim' else ' · '.join(sub.split(' · ')[:2])
        d.text((x, y + 4), f'{cab[:44]} · {FUNDOS[col]}', font=fonte, fill=(170, 200, 255))
    det = (csp_cheia if nota == 'meia' else rgb)[60:160, 250:550]
    x = LW + 3 * (CW + 8)
    fol.paste(Image.fromarray(det.clip(0, 255).astype(np.uint8)).resize((DW, 200), Image.NEAREST), (x, y + TOPO))
    d.text((x, y + 4), 'detalhe 200 % (vazio)' + (' — foto a 50 % ampliada: grão borrado' if nota == 'meia' else ''),
           font=fonte, fill=(170, 200, 255))
    mm = med[i]
    txt = (f"largura 50 % {mm['larg50']} px · borda 10–90 % {mm['borda']} px\n"
           f"cobertura do miolo {mm['cob_miolo']} · irregularidade (cv) {mm['cv_miolo']}\n"
           f"miolo a zero {mm['vazio_miolo']} · pontos quase soltos {mm['quase_soltos']}\n"
           f"grão {mm['grao_x']}×{mm['grao_y']} px · coerência ida×volta {mm['coerencia_ida_volta']}"
           + (f"\ncorr. com o papel sintético {mm['corr_papel_sintetico']}" if nota == 'sim' else ''))
    d.multiline_text((x, y + TOPO + 210), txt, font=fonte, fill=(220, 220, 160), spacing=4)
fol.save(f'{OUT}/folha_pastel.png', optimize=True)
for m in med:
    print(m['linha'], m['quem'].split(' · ')[1], {k: v for k, v in m.items() if k not in ('a_50pc', 'linha', 'quem')})
