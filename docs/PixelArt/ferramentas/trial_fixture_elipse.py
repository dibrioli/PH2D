"""Grava crates/ph2d-pixel-core/tests/fixtures/oracle_trial_ellipse_grid.json das leituras do
trial_elipse_grelha.py (a marquee eliptica do Aseprite Trial).

uso: python3 -I trial_fixture_elipse.py <grelha.json da grelha 1..24 (tl e br)> <grelha.json extra> ... <saida>
As leituras antigas (sem "at") tem a posicao refeita por trial_elipse_grelha.pages() — o empacotamento
e deterministico. Cada caixa guarda as linhas como [x0, x1] relativos a caixa (null = linha vazia)."""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.argv, args = sys.argv[:2], sys.argv[1:]   # o driver le sys.argv[1] ao importar
import trial_elipse_grelha as drv   # noqa: E402

*ins, out = args
spans = lambda m: [[r.index('1'), len(r) - 1 - r[::-1].index('1')] if '1' in r else None for r in m]

grid = json.load(open(ins[0]))
pos = {}
for p in drv.pages([(w, h) for w in range(1, drv.MAXS + 1) for h in range(1, drv.MAXS + 1)]):
    for w, h, x, y in p:
        pos[f'{w}x{h}'] = [x, y]
def norm(v, k):
    return (v['at'], v['mask']) if isinstance(v, dict) else (pos[k], v)

cases, same = {}, 0
for k, v in grid['tl'].items():
    at, m = norm(v, k)
    cases[k] = {'at': at, 'rows': spans(m)}
    same += norm(grid['br'][k], k)[1] == m
assert same == len(grid['tl']) == 576, same
extra = {}
for f in ins[1:]:
    for k, v in json.load(open(f))['tl'].items():
        extra[k] = {'at': v['at'], 'rows': spans(v['mask'])}
assert not set(extra) & set(cases)
cases.update(extra)

fx = {
    'source': 'Aseprite Trial 1.3.18.6 (AppImage, observed black box) under its own Xvfb 1600x1000, isolated HOME, '
              'screen_scale 2, 400 % zoom (8 screen px per cell), centre of each cell. Driven by '
              'docs/PixelArt/ferramentas/trial_elipse_grelha.py (xdotool), written by trial_fixture_elipse.py. '
              'Aseprite source never read. Measured 2026-10-27.',
    'method': 'Elliptical Marquee (Shift+M) in Add mode (bar button, never Shift) on a 160x100 opaque red sprite: '
              'many boxes per page, shelf-packed with 4 free cells between them (the transform handles grab presses '
              'closer than that), press at the CENTRE of the box corner cell "at" and release at the centre of the '
              'opposite corner (w-1, h-1 further); mask read by Edit>Fill #FF00FF + Esc + screenshot; nothing outside '
              'the boxes may be selected (checked per page). "rows"[j] = [x0, x1] selected in row j, relative to the '
              'box (null = empty row).',
    'command': 'PH2D_TRIAL_DISPLAY=:84 python3 -I trial_elipse_grelha.py <dir> tl br ; ... <dir2> tl --caixas '
               '<lista.json> (sides 26..64 x 3 odd heights each; even widths 66..150 and even heights 26..96 with '
               'one small odd other side) ; python3 -I trial_fixture_elipse.py <dir>/grelha.json <dir2>/grelha.json '
               '<dir3>/grelha.json <this file>',
    'checks': 'drag direction: the 1..24 grid was measured pressing top-left AND bottom-right, 576/576 identical '
              '(and tl/br/tr/bl on 16 boxes); position: 5 boxes at 4 different (x, y) each, identical; zoom: the '
              '112 boxes of oracle_trial_selection.json (800 %) are reproduced exactly here.',
    'law_observed': 'odd side, or even side in {4, 8, 12, 22}: Zingl plotEllipseRect on that axis; any other even '
                    'side n: Zingl with n+1 on that axis, then the two lines next to the middle one dropped and the '
                    'middle one doubled (rows and columns independently). 715/715.',
    'cases': cases,
}
head = json.dumps({k: v for k, v in fx.items() if k != 'cases'}, indent=1)[:-2]
body = ',\n'.join(f'  {json.dumps(k)}: {json.dumps(v, separators=(",", ":"))}'
                  for k, v in sorted(cases.items(), key=lambda kv: tuple(map(int, kv[0].split('x')))))
open(out, 'w').write(head + ',\n "cases": {\n' + body + '\n }\n}\n')
assert json.load(open(out))['cases'] == cases
print(len(cases), 'casos ->', out)
