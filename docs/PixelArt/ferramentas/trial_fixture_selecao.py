"""Grava a fixtura oracle_trial_selection.json a partir de entradas.json (trial_ase_selecao.py) e lido.json
(trial_selecao.py). uso: python3 -I trial_fixture_selecao.py <dir> <saida.json>
Mascaras = linhas de '0'/'1'. Imagens = grelha de ids da ENTRADA: para xf.ase o id 1..19 da forma (0 =
transparente, o xadrez 808080/c0c0c0 da tela); para sz/szb/pm o indice y*24+x da celula de sz.ase (-2 =
transparente); -1 = cor que nao e de nenhuma entrada -> o caso leva tambem 'rgb' (hex de tela, opaco)."""
import json, os, sys

D, OUT = sys.argv[1], sys.argv[2]
sys.argv = [sys.argv[0], D]
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trial_selecao as T   # so os PARAMS (nao corre nada)

E = json.load(open(os.path.join(D, 'entradas.json')))
L = json.load(open(os.path.join(D, 'lido.json')))
hx = lambda p: '%02x%02x%02x' % tuple(p[:3])
hxa = lambda p: '%02x%02x%02x%02x' % tuple(p[:4])
XF = {hx(p): i for p, i in zip(E['xf'], E['xf_ids']) if i}
SZ = {hx(p): k for k, p in enumerate(E['sz'])}
CHECKER = {'808080', 'c0c0c0'}


def ids_of(rgb, table):
    out, unk = [], False
    for row in rgb:
        r = []
        for h in row:
            if h in table:
                r.append(table[h])
            elif h in CHECKER:
                r.append(0 if table is XF else -2)
            else:
                r.append(-1); unk = True
        out.append(r)
    return out, unk


cases = {}
for name, prm in T.PARAMS.items():
    if name not in L:
        continue
    v = L[name]
    c = {k: (list(x) if isinstance(x, tuple) else x) for k, x in prm.items()}
    if isinstance(v, list):
        c['mask'] = v
    elif 'error' in v:
        c['error'] = v['error']
    else:
        c['w'], c['h'] = v['w'], v['h']
        table = XF if prm['file'].startswith('xf') else SZ
        ids, unk = ids_of(v['rgb'], table)
        c['ids'] = ids
        if unk:
            c['rgb'] = [''.join(row) for row in v['rgb']]
        if 'floating_after_paste' in v:
            c['floating_after_paste'] = v['floating_after_paste']
    cases[name] = c

W, H = E['w'], E['h']
rows = lambda px: [''.join(hxa(px[y * W + x]) for x in range(W)) for y in range(H)]
fx = {
    'source': 'Aseprite Trial 1.3.18.6 (AppImage, observed black box) under Xvfb :78 1600x1000, screen_scale 2, '
              '800 % zoom (16 screen px per cell), centre of each cell; inputs written by '
              'docs/PixelArt/ferramentas/trial_ase_selecao.py (public .ase spec), driven by trial_selecao.py '
              '(xdotool; quirks in its docstring), this file by trial_fixture_selecao.py. Aseprite source never read.',
    'method': 'cell coords (x, y) are sprite pixels; a drag "press A release B" means press at the CENTRE of cell A '
              'and release at the centre of cell B (sub-cell sweeps: start/end cell = floor of the pointer). '
              'mask = selection read by Edit>Fill with opaque #FF00FF (1 = selected). Images: see ids legend.',
    'ids_legend': 'xf*.ase: 1..19 = the shape pixel ids (inputs.xf_ids), 0 = transparent; sz/szb/pm: y*24+x of '
                  'sz.ase, -2 = transparent; -1 = colour of no input (case then carries rgb rows, screen hex, '
                  'composited on the checker if not opaque).',
    'laws_observed': {
        'rect_marquee': 'cells from floor(press) to floor(release) INCLUSIVE on both axes, any drag direction, '
                        'clipped to the sprite; a click without motion deselects (10/10)',
        'ellipse_marquee': 'box = inclusive press..release cells; the filled shape equals Zingl plotEllipseRect '
                           '(x0,y0,x1,y1) spans in 106/112 boxes; 4x6 5x6 6x5 10x6 10x8 24x16 are 1 px tighter at '
                           'the top/bottom rows (tables here are the truth)',
        'polygonal_lasso': 'even-odd interior sampled at cell centres (vertices at cell centres) UNION the Bresenham '
                           'outline of every edge incl. the closing one (6/6 exact); freehand lasso = same rule over '
                           'the mouse samples',
        'modes': 'Shift add, Alt+Shift subtract, Ctrl+Shift intersect (equal to the bar buttons); Alt alone with the '
                 'marquee = eyedropper; Ctrl = Move tool (drags the selected pixels)',
        'select_menu': 'All = whole sprite; Reselect restores the last mask; Inverse = complement (of nothing = all); '
                       'Deselect = Esc / ctrl+d',
        'modify': 'expand = dilation, contract = erosion (sprite edge counts as OUTSIDE), border = mask - contract '
                  '(INNER band, width n); square brush = (2n+1)^2 square; circle brush = the ellipse-marquee mask of '
                  'a (2n+1)x(2n+1) box (n=1 plus, n=2 5x5 minus corners, n=3 a2+b2<=12) (90/90)',
        'magic_wand': 'selected iff max(|dR|,|dG|,|dB|,|dA|) <= tolerance against the clicked RGBA (raw, no '
                      'premultiply); when the clicked pixel has A=0 every A=0 pixel matches whatever its RGB; '
                      'contiguous = flood fill 4/8 (Pixel Connectivity popup); non-contiguous = global (41+7 cases)',
        'color_range': 'same law as the non-contiguous wand with the dialog colour (7/7)',
        'flip': 'with a selection: mirror inside the selection bounds; without: whole sprite (= Flip Canvas)',
        'edit_rotate': '180 in place; 90 of a w x h selection: new h x w block at (floor(cx - h/2), floor(cy - w/2)) '
                       'with c = box centre, old area cleared; without selection: content rotated as the canvas '
                       'would be (x\'=H-1-y, y\'=x for CW) but canvas size kept (clipped)',
        'rotate_canvas': '90 CW/CCW swap w,h (x\'=H-1-y,y\'=x / x\'=y,y\'=W-1-x); 180 and flips in place',
        'edit_shift': '1 px with wrap inside the selection bounds (needs a selection; ctrl+a = whole sprite)',
        'transform': 'fields only with lifted pixels; positive angle = CCW on screen; 90 exact; scale by w/h fields = '
                     'nearest with src = floor((d+0.5)*S/N), placed at floor(c - N/2); fast rotation is NOT a plain '
                     'inverse-nearest map (best 12..19 cells off); RotSprite drops pixels of 1-px lines',
        'sprite_size': 'nearest: src = floor(d*S/N); bilinear: aligned corners fx = d*(S-1)/(N-1), every channel within '
                       '1 of the float bilinear, truncation exact on 99 %; RotSprite = nearest at integer 2x, own '
                       'pattern otherwise (source colours only)',
        'canvas_size': 'anchor centre: left = trunc(dw/2), top = dh - trunc(dh/2); new area transparent, or the colour '
                       'bar BACKGROUND colour (#000000 here) on a Background layer',
        'crop': 'to the selection BOUNDS, pixels outside an elliptical mask kept',
        'trim': 'removes transparent AND uniform opaque border (blue on a normal layer too)',
        'paste': 'lands at the copied position (floating); Copy = active layer, Copy Merged = flattened visible',
    },
    'inputs': {
        'w': W, 'h': H,
        'red': hxa(E['red']),
        'wand_rows_rgba': rows(E['wand']),
        'wand_note': 'ref (0,0) = 000000ff; column x=1..23 has d = wand_ds[x-1]; row 0 R+d, 1 G+d, 2 B+d, 3 A-d, '
                     '4 R+d G+d, 5 R+d G+d B+d, 6 R+d and A-d, 8 RGB=(d,d,d) A=0 (and (0,8) = 00000000)',
        'wand_ds': E['wand_ds'],
        'conn_rows_rgba': rows(E['conn']),
        'xf_ids': [E['xf_ids'][y * W:(y + 1) * W] for y in range(H)],
        'xf_colors_rgba': {str(i): hxa(p) for p, i in zip(E['xf'], E['xf_ids']) if i},
        'xfo_background': hxa(E['grey']),
        'sz_rows_rgba': rows(E['sz']),
        'trim_rows_rgba': rows(E['trim']),
        'trimc': 'trim with transparent -> 0000ffff (normal layer); trimcb the same on a Background layer',
        'pm': 'bottom layer = sz, top layer (active) = 0000ffff for x < 12, transparent for x >= 12',
    },
    'cases': cases,
}
json.dump(fx, open(OUT, 'w'), indent=None, separators=(',', ':'))
print(len(cases), 'casos', os.path.getsize(OUT), 'bytes')
