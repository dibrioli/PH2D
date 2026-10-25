"""Junta os .jsonl observados no fixture oracle_trial_dither.json."""
import sys, json
sys.path.insert(0, '/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/c142c8eb-76c4-4200-a92d-b3bc73e244a0/scratchpad/oraculo/tools')
from trial_fit_common import *
out_path = sys.argv[1]; srcs = sys.argv[2:]
DN = ['none', 'ordered2', 'ordered4', 'ordered8', 'old2', 'old4', 'old8', 'floyd']
MN = ['default_octree', 'table_rgb5_a3', 'octree']
FN = ['default_euclidean', 'rgb', 'linearized_rgb', 'ciexyz', 'cielab']
cases = {}; used = set()
for p in srcs:
    for r in records(p):
        if r['undo_mismatch'] != 0 or -1 in r['indices']:
            print('skip', r['image'], r['palette'], r['dither'], file=sys.stderr); continue
        k = (r['image'], r['palette'], r['dither'], r['mapping'], r['best_fit'])
        cases[k] = dict(image=r['image'], palette=r['palette'], dither=DN[r['dither']], mapping=MN[r['mapping']],
                        best_fit=FN[r['best_fit']], indices=r['indices'])
        used.add(r['image'])
doc = dict(
    source='Aseprite Trial 1.3.18.6 (AppImage), observed by screenshot at 800% zoom under Xvfb; '
           'Sprite > Color Mode > More Options > -> Indexed, Advanced Options checked. Inputs are RGB PNG without alpha '
           '(Background layer, index 0 usable). Palette loaded from GIMP .gpl before converting. '
           'Driver docs/PixelArt/ferramentas/trial_dither.py; never read Aseprite source.',
    settings=dict(amount_percent=100, merge_layers=False,
                  note='mapping/best_fit per case. Default(Octree)+RGB no-dither == exact nearest (sq. Euclid RGB, ties lowest index); Default(Euclidean) is NOT. Floyd == serpentine, int error, each share trunc(e*w/16), value clamped. Ordered/Old laws NOT yet fitted exactly (see docs).'),
    palettes=PALS,
    inputs={n: dict(w=SIZES[n][0], h=SIZES[n][1], rgb=IMGS[n].reshape(-1, 3).tolist()) for n in sorted(used)},
    cases=[cases[k] for k in sorted(cases)])
json.dump(doc, open(out_path, 'w'), separators=(',', ':'))
print(len(doc['cases']), 'cases', file=sys.stderr)
