"""P8 R-G/R-S/R-O/R-B no Aseprite Trial 1.3.18.6 (caixa preta, so corrido; 02/11).

uso: PH2D_TRIAL_DISPLAY=:93 python3 -I trial_efeitos_ferramentas.py <dir> <passo> [args]
     passos: probe <w> <h> <keys...>  (abre branco wxh, carrega teclas, fotografa)
             grad | fillstroke | outline | blur   -> <dir>/<passo>.json
Reusa trial_elipse_grelha.py (abre/fecha/shot/cc) e trial_dinamica.py (write_white).
Lancador: Xvfb :93 -screen 0 1600x1000x24 -nolisten tcp & ;
  env -u WAYLAND_DISPLAY DISPLAY=:93 SDL_VIDEODRIVER=x11 HOME=<scratch>/home Aseprite-trial.AppImage &
  (aseprite.ini: Frame 0 0 1600 1000, screen_scale 2, fg_color rgb{0,0,0,255}, bg_color rgb{255,0,0,255})
"""
import json, os, struct, subprocess, sys, time, zlib
import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trial_elipse_grelha as drv   # noqa: E402
import trial_dinamica as din        # noqa: E402

D = drv.D
xdo, key, shot = drv.xdo, drv.key, drv.shot


def write_rgba(path, w, h, px):
    """px: lista de (r,g,b,a) linha a linha"""
    raw = b''.join(bytes(p) for p in px)
    _s, _c = drv._string, drv._chunk
    chunks = [_c(0x2007, struct.pack('<HHI8x', 1, 0, 0)),
              _c(0x2004, struct.pack('<HHHHHHB3x', 3, 0, 0, 0, 0, 0, 255) + _s('src')),
              _c(0x2005, struct.pack('<HhhBHh5x', 0, 0, 0, 255, 2, 0) + struct.pack('<HH', w, h)
                 + zlib.compress(raw))]
    fr = b''.join(chunks)
    body = struct.pack('<IHHH2xI', 16 + len(fr), 0xF1FA, len(chunks), 100, len(chunks)) + fr
    hdr = struct.pack('<IHHHHHIHII', 128 + len(body), 0xA5E0, 1, w, h, 32, 1 | 2, 100, 0, 0)
    hdr += struct.pack('<B3xHBBhhHH', 0, 0, 1, 1, 0, 0, 16, 16)
    hdr += b'\0' * (128 - len(hdr))
    open(path, 'wb').write(hdr + body)


def opendoc(name, w, h, zoom='3'):
    drv.CW, drv.CH = w, h
    return drv.abre(name, zoom)


def grid(a):
    return [['%02x%02x%02x' % tuple(a[int(drv.Y0 + (j + 0.5) * drv.CELL), int(drv.X0 + (i + 0.5) * drv.CELL)])
             for i in range(drv.W)] for j in range(drv.H)]


def probe(w, h, *keys):
    din.write_white(os.path.join(D, f'white{w}x{h}.ase'), w, h)
    opendoc(f'white{w}x{h}.ase', w, h)
    for k in keys:
        key(k, t=0.5)
    a = shot('probe.png', park=False)
    print(drv.X0, drv.Y0, drv.W, drv.H)


# ── R-G: o Gradient (Shift+G). Barra: Linear (510,72) · Radial (538,72) · lista do dither (822,72) ──
DITHER_Y = {'solid': 105, 'bayer2': 163, 'bayer4': 221, 'bayer8': 279}


def px_of(c):
    """celula fracionaria -> pixel de tela (centro da celula = c inteiro)"""
    return int(drv.X0 + (c[0] + 0.5) * drv.CELL), int(drv.Y0 + (c[1] + 0.5) * drv.CELL)


def set_dither(name):
    xdo('mousemove', 822, 72, 'click', 1); time.sleep(0.6)
    xdo('mousemove', 640, DITHER_Y[name], 'click', 1); time.sleep(0.4)


def grad_case(w, h, shape, a, b, tag):
    opendoc(f'white{w}x{h}.ase', w, h)
    key('shift+g', t=0.4)
    xdo('mousemove', 510 if shape == 'linear' else 538, 72, 'click', 1); time.sleep(0.3)
    drv.drag(px_of(a), px_of(b))
    time.sleep(0.3)
    g = grid(shot(tag + '.png'))
    drv.fecha()
    return g


GRAD_CASES = [  # (w, h, shape, a, b, nome)
    (32, 8, 'linear', (0, 4), (31, 4), 'h32x8'),
    (32, 32, 'linear', (0, 16), (31, 16), 'h'),
    (32, 32, 'linear', (8, 16), (23, 16), 'h_curto'),
    (32, 32, 'linear', (-8, 16), (39, 16), 'h_fora'),
    (32, 32, 'linear', (16, 0), (16, 31), 'v'),
    (32, 32, 'linear', (0, 0), (31, 31), 'diag'),
    (32, 32, 'radial', (8, 8), (24, 24), 'radial'),
    (32, 32, 'radial', (4, 12), (28, 20), 'radial_elipse'),
]


def main_grad(fills):
    for w, h in {(c[0], c[1]) for c in GRAD_CASES}:
        din.write_white(os.path.join(D, f'white{w}x{h}.ase'), w, h)
    out_p = os.path.join(D, os.environ.get('GRAD_OUT', 'grad.json'))
    out = json.load(open(out_p)) if os.path.exists(out_p) else {}
    opendoc('white32x32.ase', 32, 32); key('shift+g', t=0.4)
    for f in fills:
        set_dither(f)
        drv.fecha()
        for w, h, shape, a, b, nome in GRAD_CASES:
            k = f'{f}/{nome}'
            out[k] = {'w': w, 'h': h, 'shape': shape, 'from_cell': a, 'to_cell': b, 'fill': f,
                      'grid': grad_case(w, h, shape, a, b, f'g_{f}_{nome}')}
            json.dump(out, open(out_p, 'w'))
            print(k, out[k]['grid'][h // 2][:32:2], flush=True)
        opendoc('white32x32.ase', 32, 32); key('shift+g', t=0.4)
    drv.fecha()


def set_size(n):
    """campo do tamanho do pincel na barra de contexto (230,72)"""
    xdo('mousemove', 230, 72, 'click', '--repeat', 2, '--delay', 80, 1); time.sleep(0.3)
    key('ctrl+a'); xdo('type', str(n)); key('Return', t=0.4)


# ── R-S: Edit > Fill (F) e Edit > Stroke (S) sobre a selecao; canvas branco 32x16, FG preto ──────────
RECT = ((2, 2), (11, 9))       # 10 x 8
ELL = ((16, 2), (27, 13))      # 12 x 12


def fs_case(op, size, tag):
    opendoc('white32x16.ase', 32, 16)
    key('b', t=0.3); set_size(size)
    key('m', t=0.3); drv.drag(px_of(RECT[0]), px_of(RECT[1])); key(op, t=0.6); key('ctrl+d', t=0.4)
    key('shift+m', t=0.3); drv.drag(px_of(ELL[0]), px_of(ELL[1])); key(op, t=0.6); key('ctrl+d', t=0.4)
    g = grid(shot(tag + '.png'))
    drv.fecha()
    return g


def main_fs():
    din.write_white(os.path.join(D, 'white32x16.ase'), 32, 16)
    out = {}
    for op, size in (('f', 1), ('s', 1), ('s', 3), ('s', 5)):
        k = f'{op}{size}'
        out[k] = fs_case(op, size, 'fs_' + k)
        json.dump(out, open(os.path.join(D, os.environ.get('FS_OUT', 'fs.json')), 'w'))
        for row in out[k]:
            print(''.join({'000000': '#', 'ffffff': '.'}.get(c, '?') for c in row))
        print(flush=True)


# ── R-B: Blur (R) e Jumble (Shift+R) sobre um xadrez 16x16 preto/branco por celula ───────────────────
def checker(w, h):
    return [(0, 0, 0, 255) if (i + j) % 2 == 0 else (255, 255, 255, 255) for j in range(h) for i in range(w)]


def blur_case(tool, size, clicks, tag):
    opendoc('checker16.ase', 16, 16)
    want = os.path.join(D, {'blur': 'r1.png', 'jumble': 'r2.png'}[tool])   # barra de estado de referencia
    ref = np.asarray(__import__('PIL.Image').Image.open(want).convert('RGB'))[972:996, 0:200]
    xdo('mousemove', 80, 400); time.sleep(0.3)
    for _ in range(4):
        key('r', t=0.6)
        cur = shot('st.png', park=False)[972:996, 0:200]
        dist = {t: (cur != np.asarray(__import__('PIL.Image').Image.open(os.path.join(D, f)).convert('RGB'))[972:996, 0:200]).any(axis=2).sum()
                for t, f in (('blur', 'r1.png'), ('jumble', 'r2.png'))}
        if min(dist, key=dist.get) == tool:
            break
    else:
        raise SystemExit('ferramenta ' + tool + ' nao seleccionada')
    set_size(size)
    shot(tag + '_bar.png', park=False)
    for a, b in clicks:
        if a == b:
            xdo('mousemove', *px_of(a)); time.sleep(0.3); xdo('click', 1); time.sleep(0.4)
        else:
            drv.drag(px_of(a), px_of(b))
    g = grid(shot(tag + '.png'))
    drv.fecha()
    return g


def main_blur():
    write_rgba(os.path.join(D, 'checker16.ase'), 16, 16, checker(16, 16))
    out_p = os.path.join(D, os.environ.get('BLUR_OUT', 'blur.json'))
    out = json.load(open(out_p)) if os.path.exists(out_p) else {}
    for tool, size, clicks, nome in (
            ('blur', 1, [((8, 8), (8, 8))], 'blur_1'),
            ('blur', 3, [((8, 8), (8, 8))], 'blur_3'),
            ('blur', 1, [((8, 8), (8, 8))] * 2, 'blur_1_x2'),
            ('jumble', 1, [((2, 8), (13, 8))] * 3, 'jumble_1'),
            ('jumble', 3, [((2, 8), (13, 8))] * 3, 'jumble_3')):
        if nome in out:
            continue
        out[nome] = {'tool': tool, 'size': size, 'clicks': clicks, 'grid': blur_case(tool, size, clicks, 'b_' + nome)}
        json.dump(out, open(os.path.join(D, os.environ.get('BLUR_OUT', 'blur.json')), 'w'))
        for row in out[nome]['grid']:
            print(' '.join(c[:2] if c[0:2] == c[2:4] == c[4:6] else '??' for c in row))
        print(flush=True)


# ── R-O: Edit > FX > Outline (Shift+O); forma assimetrica de 6 celulas pretas ───────────────────────
SHAPE = [(6, 6), (7, 6), (8, 6), (6, 7), (6, 8), (7, 8)]


def shape_px(bg):
    px = [bg] * (16 * 16)
    for i, j in SHAPE:
        px[j * 16 + i] = (0, 0, 255, 255)   # azul: o contorno e o FG preto
    return px


def write_outline_inputs():
    write_rgba(os.path.join(D, 'shape_t.ase'), 16, 16, shape_px((0, 0, 0, 0)))
    write_rgba(os.path.join(D, 'shape_w.ase'), 16, 16, shape_px((255, 255, 255, 255)))


# dialogo (foto 70_): presets da matriz circulo (618,474) · quadrado (654,474) · horizontal (618,510) ·
# vertical (654,510); Outside (738,467) · Inside (738,493); OK (950,395); matriz 3x3 editavel em
# x 617/649/681, y 558/590/622; Outline Color = FG, Background Color = Mask
PRESET = {'circle': (618, 474), 'square': (654, 474), 'horizontal': (618, 510), 'vertical': (654, 510)}
PLACE = {'outside': (738, 467), 'inside': (738, 493)}


def outline_case(doc, preset, place, tag):
    opendoc(doc, 16, 16)
    key('shift+o', t=1.0)
    xdo('mousemove', *PRESET[preset], 'click', 1); time.sleep(0.4)
    xdo('mousemove', *PLACE[place], 'click', 1); time.sleep(0.4)
    a = shot(tag + '_dlg.png', park=False)
    mat = [[('on' if a[y, x].sum() < 200 else 'off') for x in (617, 649, 681)] for y in (558, 590, 622)]
    xdo('mousemove', 950, 395, 'click', 1); time.sleep(0.8)
    g = grid(shot(tag + '.png'))
    drv.fecha()
    return mat, g


def main_outline():
    write_outline_inputs()
    out_p = os.path.join(D, os.environ.get('OUTLINE_OUT', 'outline.json'))
    out = json.load(open(out_p)) if os.path.exists(out_p) else {}
    for doc in ('shape_t.ase', 'shape_w.ase'):
        for preset in PRESET:
            for place in PLACE:
                k = f'{doc[:-4]}/{preset}/{place}'
                if k in out:
                    continue
                mat, g = outline_case(doc, preset, place, 'o_' + k.replace('/', '_'))
                out[k] = {'matrix_read': mat, 'grid': g}
                json.dump(out, open(out_p, 'w'))
                print(k, mat, flush=True)
                for row in g[3:12]:
                    print(' '.join({'000000': '#', '0000ff': 'B', 'ffffff': '.'}.get(c, c[:2]) for c in row[3:12]))


if __name__ == '__main__':
    step = sys.argv[2]
    if step == 'probe':
        probe(int(sys.argv[3]), int(sys.argv[4]), *sys.argv[5:])
    elif step == 'grad':
        main_grad(sys.argv[3:])
    elif step == 'fillstroke':
        main_fs()
    elif step == 'outline':
        main_outline()
    elif step == 'blur':
        main_blur()
