"""Dynamics do Pencil no Aseprite Trial 1.3.18.6 (R-V, R-S do P7_plano §5). Caixa preta, so corrido (02/11).

uso: PH2D_TRIAL_DISPLAY=:92 python3 -I trial_dinamica.py <dir> {size|zoom|grad|stab} <tag>  -> <dir>/<tag>.json
     o popup configura-se A MAO antes de cada modo (xdotool): ver coordenadas abaixo.

Reusa trial_elipse_grelha.py (abre/fecha/shot). Canvas BRANCO 160 x 100 (write_white) a 400 % (8 px/celula);
'zoom' corre a 200 % (4 px/celula). Arrasto = UM processo xdotool com mousemove+sleep encadeados (stroke).
Coordenadas (1600x1000, screen_scale 2): campo do tamanho do pincel (230,72) duplo-clique + ctrl+a + numero;
botao Dynamics (320,72); no popup: Stabilizer checkbox (322,109), matriz coluna Velocity x=496,
linhas Size y=170 · Angle y=194 · Gradient y=218; o resto do popup move-se com o que esta ligado
(fotos 55_/56_). Tecla x troca FG/BG (o HOME limpo comeca FG branco/BG preto).
Lancador: Xvfb :92 -screen 0 1600x1000x24 -nolisten tcp & ; env -u WAYLAND_DISPLAY DISPLAY=:92
SDL_VIDEODRIVER=x11 HOME=<scratch>/home ~/.local/opt/aseprite-trial/Aseprite-trial.AppImage &
"""
import json, math, os, struct, subprocess, sys, time, zlib
import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trial_elipse_grelha as drv   # noqa: E402

D = drv.D
xdo, key, shot = drv.xdo, drv.key, drv.shot


def write_white(path, w, h):
    raw = bytes([255, 255, 255, 255]) * (w * h)
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


# ── o arrasto controlado: UM processo xdotool com mousemove+sleep encadeados (sem custo de spawn) ──────
def stroke(row, i0, i1, step_px, dt, settle=0.0):
    """arrasto recto horizontal na linha de celulas `row`, de i0 a i1, passo step_px de tela, dt s entre
    movimentos; devolve (px_tela, segundos de relogio do arrasto)"""
    a = drv.cc(i0, row); b = drv.cc(i1, row)
    xdo('mousemove', *a); time.sleep(0.3); xdo('mousemove', a[0] + 1, a[1]); time.sleep(0.2)
    xdo('mousemove', *a); time.sleep(0.2)
    cmd = ['mousedown', '1']
    x = a[0]
    while x < b[0]:
        x = min(x + step_px, b[0])
        cmd += ['sleep', f'{dt:.4f}', 'mousemove', str(x), str(a[1])]
    if settle:
        cmd += ['sleep', f'{settle:.3f}']
    cmd += ['mouseup', '1']
    t = time.time(); xdo(*cmd); el = time.time() - t
    time.sleep(0.4)
    return b[0] - a[0], el


def heights(a, row, half=10):
    """altura (celulas nao-brancas) de cada coluna na faixa row-half..row+half; e o 1.o/ultimo i pintado"""
    out = []
    for i in range(drv.W):
        h = 0
        for j in range(max(0, row - half), min(drv.H, row + half + 1)):
            p = a[int(drv.Y0 + (j + 0.5) * drv.CELL), int(drv.X0 + (i + 0.5) * drv.CELL)]
            h += int(p.sum() < 3 * 250)
        out.append(h)
    return out


def fresh():
    """documento branco novo (fecha sem gravar e reabre white.ase), Pencil activo"""
    drv.fecha(); drv.abre('white.ase'); key('b')


DT = 0.002
SIZE_STEPS = [8, 16, 24, 32, 48, 64]


def page_run(specs, tag, read=heights):
    """ate 5 arrastos por documento (linhas 9, 29, 49, 69, 89); specs = [(step_px, dt)]"""
    out = []
    for p in range(0, len(specs), 5):
        fresh()
        meta = []
        for k, (st, dt) in enumerate(specs[p:p + 5]):
            r = 9 + k * 20
            px, el = stroke(r, 8, 150, st, dt)
            meta.append((r, st, dt, px, el))
        a = shot(f'{tag}_{p // 5}.png')
        for r, st, dt, px, el in meta:
            out.append({'step_px': st, 'dt_s': dt, 'screen_px': px, 'wall_s': round(el, 4),
                        'px_per_s': round(px / el), 'cells_per_s': round(px / el / drv.CELL, 1),
                        'row': r, 'read': read(a, r)})
    return out


def size_case(tag):
    specs = [(st, DT) for st in SIZE_STEPS for _ in range(3)]
    return page_run(specs, tag)


def main_size(tag):
    res = size_case(tag)
    p = os.path.join(D, f'{tag}.json'); json.dump(res, open(p, 'w'))
    for r in res:
        h = [x for x in r['read'] if x]
        print(tag, r['step_px'], r['px_per_s'], 'len', len(h), 'max', max(h), 'first2', h.index(2) if 2 in h else None)




def main_zoom(tag):
    """o mesmo passo de TELA a 200 % (4 px/celula): a unidade da velocidade e tela ou celula?"""
    drv.CELL = 4
    _abre = drv.abre
    drv.abre = lambda nome, zoom='2': _abre(nome, zoom)
    res = page_run([(st, DT) for st in (16, 32) for _ in range(3)], tag)
    json.dump(res, open(os.path.join(D, f'{tag}.json'), 'w'))
    for r in res:
        h = [x for x in r['read'] if x]
        print(tag, r['step_px'], r['px_per_s'], 'len', len(h), 'max', max(h), h)


def colors(a, row, half=2):
    """cor (hex) de cada celula nas linhas row-half..row+half"""
    return [''.join('%02x%02x%02x' % tuple(a[int(drv.Y0 + (j + 0.5) * drv.CELL),
                                             int(drv.X0 + (i + 0.5) * drv.CELL)]) + ' '
                    for i in range(drv.W)).split() for j in range(row - half, row + half + 1)]


def main_grad(tag):
    """Gradient <- Velocity (Bayer 4x4 escolhido no popup antes), pincel 5 px sem Size dinamico"""
    res = page_run([(st, DT) for st in (16, 32, 64)], tag, read=colors)
    json.dump(res, open(os.path.join(D, f'{tag}.json'), 'w'))
    for r in res:
        print(tag, r['step_px'], r['px_per_s'])
        for line in r['read']:
            print(' '.join({'000000': '#', 'ffffff': '.'}.get(c, '?') for c in line[:150]))


def main_stab(tag):
    """Stabilizer ligado (popup antes): arrasto lento i 8 -> 80 (passo 8 px, 20 ms) e largar com/sem
    pausa de 1 s parado; le o ultimo i pintado vs o i do cursor (80)"""
    out = []
    fresh()
    for k, settle in enumerate((0.0, 1.0, 0.0, 1.0)):
        r = 9 + k * 20
        px, el = stroke(r, 8, 80, 8, 0.02, settle=settle)
        out.append({'settle_s': settle, 'row': r, 'release_i': 80, 'px_per_s': round(px / el)})
    a = shot(f'{tag}.png')
    for o in out:
        h = heights(a, o['row'])
        on = [i for i, x in enumerate(h) if x]
        o.update(first_i=on[0], last_i=on[-1], lag_cells=80 - on[-1], lag_px=(80 - on[-1]) * drv.CELL, read=h)
        print(tag, o['settle_s'], o['first_i'], o['last_i'], o['lag_cells'], max(h))
    json.dump(out, open(os.path.join(D, f'{tag}.json'), 'w'))


# ── rodada 2 (02/11): XTest por ctypes no MESMO processo — cada evento com time.monotonic ──────────────
import ctypes  # noqa: E402
_X = ctypes.CDLL('libX11.so.6'); _T = ctypes.CDLL('libXtst.so.6')
_X.XOpenDisplay.restype = ctypes.c_void_p
_DPY = None


def _dpy():
    global _DPY
    if _DPY is None:
        _DPY = _X.XOpenDisplay(drv.ENV['DISPLAY'].encode())
    return ctypes.c_void_p(_DPY)


def xt_move(x, y):
    _T.XTestFakeMotionEvent(_dpy(), -1, int(x), int(y), 0); _X.XFlush(_dpy())


def xt_button(down):
    _T.XTestFakeButtonEvent(_dpy(), 1, 1 if down else 0, 0); _X.XFlush(_dpy())


def serp_rows(n_rows, i0=20, i1=620, j0=20, dj=40):
    """serpentina em celulas: n_rows linhas horizontais j0, j0+dj, ... alternando o sentido"""
    pts = []
    for r in range(n_rows):
        j = j0 + r * dj
        a, b = (i0, i1) if r % 2 == 0 else (i1, i0)
        pts += [(a, j), (b, j)]
    return pts


def path_events(pts_cells, segs):
    """pts_cells = vertices em celulas; segs = [(dist_px_ate, step_px, v_px_s)] por troco de
    distancia acumulada de TELA; devolve [(x, y, d, dt_antes)]"""
    P = [drv.cc(i, j) for i, j in pts_cells]
    out, d, k = [], 0.0, 0
    x, y = P[0]
    for (ax, ay), (bx, by) in zip(P, P[1:]):
        L = math.hypot(bx - ax, by - ay); s = 0.0
        while s < L:
            while k < len(segs) - 1 and d >= segs[k][0]:
                k += 1
            _end, st, v = segs[k]
            step = min(st, L - s); s += step; d += step
            out.append((ax + (bx - ax) * s / L, ay + (by - ay) * s / L, d, step / v))
    return P[0], out


def stroke_t(start, ev, pauses=()):
    """arrasto com relogio proprio: espera activa ate o instante de cada evento; pauses = {d_px: s}"""
    xt_move(*start); time.sleep(0.3); xt_move(start[0] + 1, start[1]); time.sleep(0.15)
    xt_move(*start); time.sleep(0.2)
    xt_button(True); time.sleep(0.05)
    t0 = time.monotonic(); tn = t0; log = []
    pz = sorted(pauses)
    for x, y, d, dt in ev:
        tn += dt
        if pz and d >= pz[0]:
            tn += pauses[pz.pop(0)]
        while time.monotonic() < tn:
            pass
        t = time.monotonic(); xt_move(round(x), round(y)); log.append((round(t - t0, 5), round(x), round(y), round(d, 1)))
    time.sleep(0.15); xt_button(False); time.sleep(0.5)
    return log


def row_sizes(a, j, half=14):
    """altura (celulas nao-brancas) por coluna em torno da linha j"""
    lo, hi = max(0, j - half), min(drv.H, j + half + 1)
    ys = [int(drv.Y0 + (q + 0.5) * drv.CELL) for q in range(lo, hi)]
    xs = [int(drv.X0 + (i + 0.5) * drv.CELL) for i in range(drv.W)]
    sub = a[np.ix_(ys, xs)]
    return (sub.sum(axis=2) < 3 * 250).sum(axis=0).tolist()


def big_setup(cw=640, ch=400, zoom='1', cell=2):
    drv.CW, drv.CH, drv.CELL = cw, ch, cell
    write_white(os.path.join(D, 'wbig.ase'), cw, ch)
    _abre = drv.abre
    drv.abre = lambda nome, zoom_=zoom: _abre('wbig.ase', zoom_)


def run_serp(tag, segs, n_rows=10, pauses=()):
    fresh()
    start, ev = path_events(serp_rows(n_rows), segs)
    log = stroke_t(start, ev, dict(pauses))
    a = shot(f'{tag}.png')
    rows = {20 + 40 * r: row_sizes(a, 20 + 40 * r) for r in range(n_rows)}
    return {'tag': tag, 'segs': segs, 'pauses': list(pauses), 'n_rows': n_rows, 'cell_px': drv.CELL,
            'log': log, 'rows': rows}


def main_r2(tag, spec_json):
    """spec_json = [[nome, segs, n_rows, pauses], ...]; um documento por caso, tudo em <tag>.json"""
    big_setup()
    out = []
    for nome, segs, n_rows, pauses in json.loads(open(spec_json).read()):
        r = run_serp(f'{tag}_{nome}', [tuple(s) for s in segs], n_rows, [tuple(p) for p in pauses])
        out.append(r)
        json.dump(out, open(os.path.join(D, f'{tag}.json'), 'w'))
        mx = max(max(v) for v in r['rows'].values())
        print(nome, 'events', len(r['log']), 'wall', r['log'][-1][0], 'max', mx, flush=True)


def run_rows(tag, specs):
    """ate 10 arrastos RECTOS por documento (linhas j = 20 + 40 k, i 20 -> 620): a serpentina nao serve
    porque a inversao de sentido zera o filtro (medido: cada volta recomeca em 1). spec = {'segs':
    [(d_ate, step, v)], 'pauses': [(d, s)]}"""
    fresh()
    out = []
    for k, sp in enumerate(specs):
        j = 20 + 40 * k
        start, ev = path_events([(20, j), (620, j)], [tuple(x) for x in sp['segs']])
        log = stroke_t(start, ev, {d: z for d, z in sp.get('pauses', [])})
        out.append(dict(sp, row=j, log=log))
    a = shot(f'{tag}.png')
    for o in out:
        o['read'] = row_sizes(a, o['row'])
    return out


def main_r2rows(tag, spec_json):
    """spec_json = {nome_doc: [spec, ...]}; tudo em <tag>.json"""
    big_setup()
    res = {}
    for nome, specs in json.loads(open(spec_json).read()).items():
        res[nome] = run_rows(f'{tag}_{nome}', specs)
        json.dump(res, open(os.path.join(D, f'{tag}.json'), 'w'))
        for o in res[nome]:
            print(nome, o.get('name'), 'wall', o['log'][-1][0], o['read'][20:621:20], flush=True)


if __name__ == '__main__' and len(sys.argv) > 2:
    globals()['main_' + sys.argv[2]](*sys.argv[3:])
