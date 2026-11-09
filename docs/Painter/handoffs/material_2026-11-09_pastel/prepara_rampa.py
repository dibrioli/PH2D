# Bateria do Krita da rodada 2 (09/11): a mesma cena v3 com a pressão em RAMPA 0,1→1,0 ao longo de cada traço
# (a sonda `PH2D_V3_PRESSAO=rampa`), e a mesma rampa com a cena repetida 2× por cima (`PH2D_V3_PASSADAS=2`).
# Uso: python3 -I prepara_rampa.py <scratch>   (lê <scratch>/bat/bateria_pastel.json, a de prepara.py)
import copy
import json
import sys

S = sys.argv[1]
b = json.load(open(S + '/bat/bateria_pastel.json'))
ops = b['cenas'][0]['ops']
tr = [o for o in ops if o['op'] == 'traco']
for o in tr:
    n = max(len(o['pts']) - 1, 1)
    for i, p in enumerate(o['pts']):
        p[2] = 0.1 + 0.9 * i / n if len(o['pts']) > 3 else 0.1 + 0.9 * 0  # o toque fica a 0,1
for passadas in (1, 2):
    bb = copy.deepcopy(b)
    cab = [o for o in ops if o['op'] not in ('traco', 'foto')]
    corpo = [dict(o, nome=f"{o['nome']}_{k}") for k in range(passadas) for o in tr]
    bb['cenas'][0] = {'nome': 'rampa' + ('' if passadas == 1 else f'_x{passadas}'), 'ops': cab + corpo + [{'op': 'foto', 'nome': 'A'}]}
    json.dump(bb, open(f"{S}/bat/bateria_pastel_rampa{'' if passadas == 1 else '_x2'}.json", 'w'))
print('ok')
