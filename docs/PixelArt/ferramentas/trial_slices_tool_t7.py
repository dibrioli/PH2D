"""P11 T7: operacoes de geometria no Trial, uma sessao por caso; le o dialogo Slice Properties com duplo-clique
no interior previsto de cada slice (zoom 100 %, 2 px de tela por pixel; acha o canvas pela foto).
uso: python3 -I trial_slices_tool_t7.py <work> <dir .ase> <dir fixturas> [caso...]   grava <work>/T7_<caso>_<leitura>.png"""
import os, subprocess, sys, time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trial_slices_tool as t
from PIL import Image

W, ASE, FX = os.path.abspath(sys.argv[1]), sys.argv[2], sys.argv[3]
BG = None
CANCEL = 'click 997 615; wait .25'
SPRITE = 'click 121 10; wait .3'
ROT = SPRITE + '; click 170 164; wait .35'


def do(s):
    t.do(W, s)


def op_sprite(pct):
    return f'{SPRITE}; click 165 120; wait .5; fill 760 505 {pct}; fill 760 543 {pct}; click 765 650; wait .8'


def bbox():
    p = os.path.join(W, '_bb.png')
    subprocess.run(['import', '-window', 'root', p], env=t.env(W), check=True)
    im = Image.open(p).convert('RGB')
    bg = im.getpixel((172, 104))
    x0, y0, x1, y1 = 10**6, 10**6, -1, -1
    for y in range(100, 722):
        for x in range(170, 1540):
            if im.getpixel((x, y)) != bg:
                x0, y0, x1, y1 = min(x0, x), min(y0, y), max(x1, x), max(y1, y)
    return x0 + 1, y0 + 1, x1, y1


def read(case, tag, frame, cx, cy):
    do(f'frame {frame}')
    x0, y0, _, _ = bbox()
    sx, sy = x0 + 2 * int(cx) + 1, y0 + 2 * int(cy) + 1
    do(f'dbl {sx} {sy}; dl3 T7_{case}_{tag}; {CANCEL}')


FULL = [('nine', 0, 16, 16), ('pivot', 0, 48, 16), ('bothf0', 0, 16, 46), ('bothf3', 3, 44, 46)]
ODD = [('odd', 0, 12, 11), ('odd2', 0, 39, 21)]
CASES = {
    'S200': ('full', op_sprite(200), lambda x, y: (x * 2, y * 2), FULL),
    'S50': ('full', op_sprite(50), lambda x, y: (x // 2, y // 2), FULL),
    'S50odd': ('odd', op_sprite(50), lambda x, y: (x // 2, y // 2), ODD),
    'S150odd': ('odd', op_sprite(150), lambda x, y: (x * 3 // 2, y * 3 // 2), ODD),
    'CP10': ('full', None, lambda x, y: (x + 10, y + 10), FULL),
    'CM10': ('full', None, lambda x, y: (x - 10, y - 10), FULL),
    'FH': ('full', f'{ROT}; click 435 238; wait .6', lambda x, y: (63 - x, y), FULL),
    'FV': ('full', f'{ROT}; click 430 260; wait .6', lambda x, y: (x, 63 - y), FULL),
    'R180': ('full', f'{ROT}; click 357 164; wait .6', lambda x, y: (63 - x, 63 - y), FULL),
    'R90': ('full', f'{ROT}; click 370 186; wait .6', lambda x, y: (63 - y, x), FULL),
    'R270': ('full', f'{ROT}; click 377 208; wait .6', lambda x, y: (y, 63 - x), FULL),
    'CelFlip': ('full', 'key shift+h; wait .5', lambda x, y: (x, y), FULL),
    'Crop': ('full', 'wait .1', lambda x, y: (x, y), [('nine', 0, 16, 16), ('pivot', 0, 38, 16)]),
    'Trim': ('trim', f'{SPRITE}; click 133 216; wait .6', lambda x, y: (x - 8, y - 8), [('cross', 0, 16, 16)]),
}


def session(name):
    src, op, tf, reads = CASES[name]
    path = os.path.join(FX if src in ('odd', 'trim') else ASE, src + '.ase')
    os.environ['BOOT'] = '4'
    subprocess.run([sys.executable, '-I', t.__file__, W, 'open', path], check=True)
    if name == 'Crop':
        do(f'key m; setup; pdrag 0 0 40 32; {SPRITE}; click 133 194; wait .8; key shift+c; wait .3; mv 856 400; key 1; wait .4')
    else:
        do('key shift+c; setup; mv 856 400; key 1; wait .4')
    if op is None:
        d = 10 if name == 'CP10' else -10
        do(f'{SPRITE}; click 170 142; wait .6; fill {CANVAS_LEFT} {d}; fill {CANVAS_TOP} {d}; wait .2; shot T7_{name}_dlg keep; click {CANVAS_OK}; wait .8')
    else:
        do(op)
    do(f'shot T7_{name}_tela')
    for tag, fr, x, y in reads:
        cx, cy = tf(x, y)
        read(name, tag, fr, cx, cy)
    do(f'frame 0; click 749 70; wait .3; shot T7_{name}_lista keep 250 55 500 120; key Escape; wait .2')
    if name in ('S200', 'Crop', 'R90'):
        do('key ctrl+z; wait .5; key 3; wait .1; key 1; wait .4')
        read(name, 'undo_nine', 0, 16, 16)
        do(f'click 749 70; wait .3; shot T7_{name}_lista2 keep 250 55 500 120; key Escape; wait .2')


CANVAS_LEFT, CANVAS_TOP, CANVAS_OK = os.environ.get('CANVAS', '0 0|0 0|0 0').split('|')
if __name__ == '__main__':
    for n in sys.argv[4:] or CASES:
        session(n)
