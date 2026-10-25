"""Grava a fixtura oracle_trial_blend.json a partir de entradas.json + lido.json (trial_camadas.py).
uso: python3 -I trial_fixture_camadas.py <dir> <saida.json> [N]
Guarda as N primeiras celulas (por omissao 768 = 8 linhas de 96) de cada ficheiro, em hex RGB(A)."""
import json, os, sys
D, OUT = sys.argv[1], sys.argv[2]
N = int(sys.argv[3]) if len(sys.argv) > 3 else 768
E = json.load(open(os.path.join(D, 'entradas.json')))
L = json.load(open(os.path.join(D, 'lido.json')))
hx = lambda px: ''.join(f'{v:02x}' for p in px[:N] for v in p)
casos = []
for f in E['files']:
    if f not in L:
        raise SystemExit(f'falta {f}')
    casos.append({'file': f, 'screen_rgb': hx(L[f])})
json.dump({
    'source': 'Aseprite Trial 1.3.18.6 (AppImage) under Xvfb :78, 400 % zoom, centre of each 8x8 screen '
              'block; inputs written by docs/PixelArt/ferramentas/trial_ase_camadas.py (public .ase spec)',
    'control': 'm00_o255 read against exact source-over: 0 cells with |d|>1 (6144)',
    'files': 'm{MM}_o{OOO}: back opaque + src mode MM opacity OOO · g{MM}: group[mid Normal, src MM] · '
             'gm{MM}_o{OOO}: group mode MM opacity OOO [src] · cel_o{OOO}: cel opacity · ix_o{OOO}: Indexed',
    'n': N, 'w': E['w'],
    'back': hx(E['back']), 'src': hx(E['src']), 'mid': hx(E['mid']),
    'pal': hx(E['pal']), 'ib': ''.join(f'{v:02x}' for v in E['ib'][:N]), 'is': ''.join(f'{v:02x}' for v in E['is'][:N]),
    'cases': casos,
}, open(OUT, 'w'))
print(len(casos), os.path.getsize(OUT))
