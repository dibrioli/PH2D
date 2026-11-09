# Prepara as entradas NOSSAS da comparação do Pastel (09/11) a partir da cena v3 de 08/11.
# Uso: python3 -I prepara.py <scratch>   (antes: python3 -I ../material_2026-11-08_revisao/cena_v3.py <scratch>/cena)
# Grava: <scratch>/cena_volta/ (os 12 traços invertidos: R→L — a régua de coerência ida×volta) e <scratch>/bat/
# bateria_pastel{,_12}{,_volta}.json (a bateria do Krita/MyPaint: tela 2400x880, verde 00A03C, pressão 1; size 40 e 12).
import json
import os
import shutil
import sys

S = sys.argv[1]
os.makedirs(S + '/cena_volta', exist_ok=True)
os.makedirs(S + '/bat', exist_ok=True)
for f in ('base.rgba', 'base.png', 'pinceis.tsv'):
    shutil.copy(f'{S}/cena/{f}', f'{S}/cena_volta/{f}')
tr = [ln.split() for ln in open(S + '/cena/tracos.txt') if ln.strip()]
assert len(tr) == 12, len(tr)
open(S + '/cena_volta/tracos.txt', 'w').write('\n'.join(' '.join(t[::-1]) for t in tr) + '\n')
for volta in (False, True):
    for size in (40, 12):
        ops = [{"op": "pincel", "quem": "preset"}, {"op": "cor", "rgb": [0, 160, 60]}]
        for i, t in enumerate(tr):
            pts = [[float(x), float(y), 1.0] for x, y in (p.split(',') for p in t)]
            pts = pts * 3 if len(pts) == 1 else pts  # toque: 3 eventos parados
            ops.append({"op": "traco", "nome": f"t{i}", "size_px": size, "passo": 3.0, "pts": pts[::-1] if volta else pts})
        ops.append({"op": "foto", "nome": "A"})
        nome = 'bateria_pastel' + ('' if size == 40 else f'_{size}') + ('_volta' if volta else '')
        cena = 'volta' if volta else 'v3'
        json.dump({"tela": [2400, 880], "papel": [255, 255, 255], "cenas": [{"nome": cena, "ops": ops}]},
                  open(f'{S}/bat/{nome}.json', 'w'))
print('ok', S)
