"""Grava a fixtura oracle_trial_frames.json a partir de entradas.json + lido.json + onion_*.json
(trial_quadros.py). uso: python3 -I trial_fixture_quadros.py <dir> <saida.json> [N]
Guarda as N primeiras celulas (por omissao 768 = 8 linhas de 96) dos casos de tela, em hex RGB(A), e a
sequencia de quadros tocada (o 1.o passo e o quadro de partida, mostrado antes do Enter)."""
import json, os, sys
D, OUT = sys.argv[1], sys.argv[2]
N = int(sys.argv[3]) if len(sys.argv) > 3 else 768
E = json.load(open(os.path.join(D, 'entradas.json')))
L = json.load(open(os.path.join(D, 'lido.json')))
hx = lambda px: ''.join(f'{v:02x}' for p in px[:N] for v in p)
SOL = [tuple(c) for c in E['solidas']]
DIRS = {'fwd': 0, 'rev': 1, 'pp': 2, 'ppr': 3}

cel = [{'layer': l, 'cel': c, 'screen_rgb': hx(L[f'co_l{l:03d}_c{c:03d}.ase'])} for l, c in E['co']]
z = [{'name': n, 'z': zs, 'screen_rgb': hx(L[f'z_{n}.ase'])} for n, zs in E['z'].items()]
onion = {}
for v, f, ini in (('defpos', 'on_faixa.ase', {}), ('pos1', 'on_faixa.ase', {'position': 1}),
                  ('tint', 'on_vazio.ase', {'type': 1, 'prev_frames': 2, 'next_frames': 2}),
                  ('clamp', 'on_vazio.ase', {'opacity_base': 50, 'opacity_step': 60, 'prev_frames': 2, 'next_frames': 2})):
    o = json.load(open(os.path.join(D, f'onion_{v}.json')))[f]
    onion[v] = {'file': f, 'ini': ini, 'off_rgb': hx(o['off']), 'on_rgb': hx(o['on'])}
play = []
for n in sorted(k for k in L if k.startswith('tg_')):
    runs = L[n]['runs']
    seq = [SOL.index(tuple(c)) for c, _ in runs]
    ms = [round(k * 1000 / L[n]['fps']) for _, k in runs]
    case = {'file': n, 'frames': seq[:-1], 'ms': ms[1:-1]}
    if n == 'tg_dur.ase':
        case.update(durations=[100, 300, 600, 200], tag=None, start=0)
    else:
        d, r = n[3:-4].split('_r')
        case.update(durations=[250] * 6, tag={'from': 1, 'to': 4, 'direction': DIRS[d], 'repeat': int(r)}, start=1)
    play.append(case)
json.dump({
    'source': 'Aseprite Trial 1.3.18.6 (AppImage) under Xvfb :78, 400 % zoom, centre of each 8x8 screen block; '
              'inputs written by docs/PixelArt/ferramentas/trial_ase_quadros.py (public .ase spec); playback = '
              'ffmpeg x11grab 50 fps of the canvas centre after Enter',
    'note': 'cel opacity = mul(layer, cel); cels ordered by (layer index + z, z); onion skin = Normal with '
            'max(0, base - step*(d-1)), behind by default, red/blue tint = Rec.709 luma v -> ((255+v)/2, v/2, v/2) '
            'and its mirror; the onion off/on reads are of frame 2 (the checker shows where nothing is drawn)',
    'n': N, 'w': E['w'],
    'back': hx(E['back']), 'src': {k: hx(v) for k, v in E['src'].items()},
    'onion_colors': E['onion'],
    'cel': cel, 'z': z, 'onion': onion, 'play': play,
}, open(OUT, 'w'))
print(len(cel), len(z), len(onion), len(play), os.path.getsize(OUT))
