"""Resumo da régua do prompt 51: por preset, as métricas da cena (antes × fábrica × Rebelle) e a diferença grande."""
import re, sys

src = open('/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor/crates/ph2d-tool-painter/tests/it/midias_de_fabrica.rs').read()
barras = {}
for m in re.finditer(r'Midia \{ preset: "([^"]+)", foto: "[^"]+", raio: [\d.]+, metricas: &\[(.*?)\] \}', src):
    barras[m.group(1)] = [(a, float(b)) for a, b in re.findall(r'\((\w+), ([\d.]+)\)', m.group(2))]

campo = {'Cobertura': 'cobertura', 'Textura': 'textura', 'Borda': 'borda', 'Orla': 'orla', 'Cv': 'cv',
         'Tinta': 'tinta', 'Cruzamento': 'cruz', 'Ponta': 'ponta', 'RampaLargura': 'rampa',
         'RampaCobertura': 'rampa_cob', 'Pontas': 'pontas', 'Largura': 'largura'}

def nums(s):
    return {k: float(v) for k, v in re.findall(r'(\w+): (-?[\d.]+(?:e-?\d+)?)', s)}

cena, grande, escada, largura = {}, {}, {}, {}
for l in open(sys.argv[1]):
    m = re.match(r'\[regua\] (.+?)@(\w+) (cena|grande|escada|largura)(.*)', l)
    if not m:
        continue
    p, v, k, rest = m.groups()
    if k == 'cena':
        cena[(p, v)] = nums(rest)
    elif k == 'grande':
        grande[(p, v)] = rest.split(': ', 1)[1].strip()
    elif k == 'escada':
        r = rest.split(':')[0].strip()
        escada.setdefault((p, v), {})[r] = [float(x) for x in re.findall(r'escada: ([\d.]+)', rest)] + \
            [float(re.findall(r'contraste: ([\d.]+)', rest)[3])]
    else:
        largura[(p, v)] = rest.split(': ', 1)[1].strip()

vs = sys.argv[2].split(',') if len(sys.argv) > 2 else ['antes', 'fabrica']
for p in dict.fromkeys(k[0] for k in cena):
    print(f'## {p}')
    for met, r in barras.get(p, []):
        if met == 'Chato':
            vals = [cena[(p, v)]['de_pe'] / cena[(p, v)]['deitado'] for v in vs if (p, v) in cena]
        else:
            vals = [cena[(p, v)][campo[met]] for v in vs if (p, v) in cena]
        print(f'  {met:<15} R {r:<8} ' + ' · '.join(f'{v} {x:.3f}' for v, x in zip(vs, vals)))
    for k in ['leve', 'forte', 'carril', 'periodo_fracao', 'cv', 'textura']:
        vals = [cena[(p, v)].get(k, float('nan')) for v in vs if (p, v) in cena]
        print(f'  ({k:<13})            ' + ' · '.join(f'{v} {x:.3f}' for v, x in zip(vs, vals)))
    for v in vs:
        if (p, v) in escada:
            print(f'  escada {v}: ' + ' | '.join(f'r {r}: ' + ' '.join(f'{x:.2f}' for x in e) for r, e in escada[(p, v)].items()))
        if (p, v) in largura:
            print(f'  largura {v}: {largura[(p, v)]}')
        if (p, v) in grande:
            print(f'  grande {v}: {grande[(p, v)]}')
