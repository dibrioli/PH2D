"""Driver da P6 (selecao e transformacao) no Aseprite Trial (Xvfb :78, 1600x1000, screen_scale 2).
uso: python3 -I trial_selecao.py <dir> [prefixo ...]   -> <dir>/lido.json {caso: leitura}
Os .ase vem de trial_ase_selecao.py (o mesmo <dir>). Sem prefixos corre TODOS os casos (ordem de CASES);
com prefixos so os casos cujo nome comeca por um deles. lido.json e acumulativo.

Leitura: zoom 800 % (tecla 4) = 16 px de tela por celula, o centro de cada celula; a origem do canvas
e a caixa do que difere do fundo do editor, refeita a cada abertura e depois de toda mudanca de tamanho.
Mascara: Edit > Fill (F) com o fg MAGENTA opaco (255,0,255): magenta = seleccionado; depois ctrl+z, Esc.
Imagem: o RGB de tela de cada celula (o transparente mostra o xadrez: 128 ou 192 cinza, quadrados de
16 celulas a 800 %).

QUIRKS medidos (26/10, Trial 1.3.18.6):
  * O Trial PERDE o ultimo movimento do rato antes do mouseup/click (um salto A->B dava 1x1 em A; com 6
    passos valia o 5.o). Cura: tres movimentos DENTRO da celula de destino (-2, +1, 0 px) antes de soltar.
  * Laco poligonal (Shift+Q): cada aresta e um ARRASTO do vertice anterior ao seguinte; um clique parado
    no ultimo vertice FECHA. Cliques soltos nao fazem nada.
  * Deselect e Esc (o menu Select diz Esc; Ctrl+D nao aparece no menu).
  * Clicar uma linha de camada no timeline selecciona a CAMADA como intervalo: o Delete apaga a camada
    inteira. Alt+Up/Down NAO muda de camada. (Por isso a leitura usa o Fill, nunca outra camada.)
  * O popup de opcoes da varinha (botao em (990,72)) engole as teclas ate se clicar fora (1300,60).
  * O fg muda-se com DUPLO clique no campo #FFFFFF da barra de cores (64,915) -> popup; duplo clique no
    hex (316,748), ctrl+a, 'ff00ff', Enter; fecha no X (588,740). Fica na sessao.
  * Campos numericos da transformacao (P: x y w h, R: angulo skew) so aparecem com os pixels LEVANTADOS:
    ctrl+a, ctrl+t (a caixa encolhe aos limites do cel), Right, Left. Enter no campo do angulo aplica e
    larga. Angulo positivo = anti-horario na tela.
  * O algoritmo de rotacao e o combo da barra (fora da transformacao: seta em (658,72); itens Fast
    Rotation (571,104), RotSprite (556,122)). Sprite Size tem o seu: Nearest-neighbor, Bilinear, RotSprite.
  * Modificadores da marquee: Shift = somar; Alt sozinho = CONTA-GOTAS (o fg vira a cor clicada e a
    selecao fica); Alt+Shift (premidos antes do arrasto) = subtrair; Ctrl+Shift = intersectar; Ctrl =
    ferramenta Mover (arrasta os pixels). Os botoes da barra (178/206/234/262, 72) sao a referencia.
  * aseprite.ini so e lido ao arrancar; tudo aqui e pela UI.
"""
import json, math, os, subprocess, sys, time
import numpy as np
from PIL import Image

D = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else '.')
ENV = dict(os.environ, DISPLAY=':78'); ENV.pop('WAYLAND_DISPLAY', None)
CELL = 16
X0, Y0 = 656, 402
W, H = 24, 16
MAG = [255, 0, 255]


def xdo(*a): subprocess.run(['xdotool', *map(str, a)], env=ENV, check=True)
def key(*k, t=0.25):
    for x in k:
        xdo('key', x); time.sleep(t)
def keyn(k, n, t=0.4):
    if n > 0:
        xdo('key', '--repeat', n, '--delay', 15, k); time.sleep(t)
def shot(p='z.png', park=True):
    if park:
        xdo('mousemove', 1590, 990); time.sleep(0.3)
    p = os.path.join(D, p)
    subprocess.run(['import', '-window', 'root', p], env=ENV, check=True)
    return np.asarray(Image.open(p).convert('RGB')).astype(int)

def bbox(a):
    reg = a[100:948, 168:1545]
    bg = a[120, 200]
    m = np.any(reg != bg, axis=2)
    ys, xs = np.nonzero(m)
    return xs.min() + 168, ys.min() + 100, xs.max() + 168, ys.max() + 100

def rebbox(name='bb.png'):
    """origem e tamanho do canvas: borda escura 2 px; em baixo +2 px de sombra"""
    global X0, Y0, W, H
    x0, y0, x1, y1 = bbox(shot(name))
    X0, Y0 = x0 + 2, y0 + 2
    wpx, hpx = x1 - 2 - X0 + 1, y1 - 4 - Y0 + 1
    W, H = round(wpx / CELL), round(hpx / CELL)
    assert abs(wpx - W * CELL) <= 1 and abs(hpx - H * CELL) <= 2, (x0, y0, x1, y1)
    return W, H

def cc(i, j, fx=0.5, fy=0.5):
    return int(X0 + (i + fx) * CELL), int(Y0 + (j + fy) * CELL)

def _settle(b):
    for d in (-2, 1, 0):
        time.sleep(0.1); xdo('mousemove', b[0] + d, b[1] + d)
    time.sleep(0.3)

def drag(a, b, mods=(), steps=6):
    path([a, b], mods, steps)

def path(pts, mods=(), steps=6):
    """arrasto por uma polilinha; cada segmento em passos de ~4 px (minimo steps)"""
    for m in mods: xdo('keydown', m)
    xdo('mousemove', *pts[0]); time.sleep(0.15); xdo('mousedown', 1); time.sleep(0.1)
    for a, b in zip(pts, pts[1:]):
        n = max(steps, int(math.hypot(b[0] - a[0], b[1] - a[1]) / 4))
        for s in range(1, n + 1):
            xdo('mousemove', a[0] + (b[0] - a[0]) * s // n, a[1] + (b[1] - a[1]) * s // n); time.sleep(0.02)
    _settle(pts[-1])
    xdo('mouseup', 1); time.sleep(0.15)
    for m in mods: xdo('keyup', m)
    time.sleep(0.2)

def click(p, mods=()):
    for m in mods: xdo('keydown', m)
    xdo('mousemove', *p); _settle(p)
    xdo('click', 1); time.sleep(0.15)
    for m in mods: xdo('keyup', m)
    time.sleep(0.15)

def cells(a, w=None, h=None):
    w = w or W; h = h or H
    return [[a[int(Y0 + (j + 0.5) * CELL), int(X0 + (i + 0.5) * CELL)].tolist() for i in range(w)] for j in range(h)]

def mask_now(name='m'):
    ensure_fg()   # Alt+clique com a marquee e o conta-gotas: o fg pode ter mudado
    key('f', t=0.4)
    a = shot(name + '.png')
    m = [''.join('1' if p == MAG else '0' for p in row) for row in cells(a)]
    key('ctrl+z', t=0.35); key('Escape', t=0.3)
    return m

def rgb_now(name='i'):
    a = shot(name + '.png')
    return [[f'{p[0]:02x}{p[1]:02x}{p[2]:02x}' for p in row] for row in cells(a)]

def floating(a=None):
    """os pixels estao levantados? (a barra mostra os campos P:, brancos por dentro)"""
    a = shot('fl.png', park=False) if a is None else a
    return bool((a[66:78, 470:512].sum(2) >= 750).mean() > 0.5)

def commit():
    if floating():
        xdo('mousemove', 178, 72, 'click', 1); time.sleep(0.5)


def abre(nome, zoom='4'):
    xdo('key', 'ctrl+o'); time.sleep(1.2)
    xdo('type', '--delay', 12, os.path.join(D, nome)); time.sleep(0.3)
    xdo('key', 'Return'); time.sleep(1.8)
    if tuple(shot('z0.png')[723, 1300]) == (124, 145, 157):
        xdo('mousemove', 1537, 723, 'click', 1); time.sleep(0.5)   # fecha a janela Preview
    xdo('mousemove', 856, 523); xdo('key', '1'); time.sleep(0.6)
    x0, y0, x1, y1 = bbox(shot('z1.png'))
    xdo('mousemove', (x0 + x1) // 2, (y0 + y1) // 2); xdo('key', zoom); time.sleep(0.8)
    return rebbox('z1b.png')

def fecha():
    xdo('key', 'ctrl+w'); time.sleep(0.7)
    c = shot('z2.png')
    if tuple(c[437, 640]) == (124, 144, 159) and tuple(c[500, 700]) == (211, 203, 190):
        xdo('mousemove', 799, 546, 'click', 1); time.sleep(0.6)   # «Don't Save»

def menu(*pts):
    """o 1.o ponto e o titulo (clique), os do meio so passam o rato, o ultimo e clicado"""
    for k, p in enumerate(pts):
        xdo('mousemove', *p); time.sleep(0.5)
        if k == 0 or k == len(pts) - 1:
            xdo('click', 1); time.sleep(0.6)

EDIT, SEL, SPRITE = (65, 10), (318, 10), (120, 10)
MODES = {'replace': (178, 72), 'add': (206, 72), 'subtract': (234, 72), 'intersect': (262, 72)}

def tool(k):
    key('Escape'); key(k); xdo('mousemove', *MODES['replace'], 'click', 1); time.sleep(0.3)

def ensure_fg():
    a = shot('fg.png')
    if a[915, 20].tolist() != MAG:
        xdo('mousemove', 64, 915, 'click', '--repeat', 2, '--delay', 80, 1); time.sleep(0.8)
        xdo('mousemove', 316, 748, 'click', '--repeat', 2, '--delay', 80, 1); time.sleep(0.3)
        key('ctrl+a'); xdo('type', 'ff00ff'); key('Return')
        xdo('mousemove', 588, 740, 'click', 1); time.sleep(0.5)
        assert shot('fg.png')[915, 20].tolist() == MAG


# ── familias ──────────────────────────────────────────────────────────────────────────────────────
def c_rect(a, b, mods=()):
    def f():
        tool('m'); drag(cc(*a), cc(*b), mods); return mask_now()
    return f

def c_ellipse(a, b):
    def f():
        tool('shift+m'); drag(cc(*a), cc(*b)); return mask_now()
    return f

def c_lasso(pts, poly):
    def f():
        if poly:
            tool('shift+q')
            for a, b in zip(pts, pts[1:]):
                drag(cc(*a), cc(*b)); time.sleep(0.4)
            click(cc(*pts[-1])); time.sleep(0.4)
        else:
            tool('q'); path([cc(*p) for p in pts])
        return mask_now()
    return f

R1, R2 = ((3, 3), (12, 10)), ((8, 6), (18, 13))
def c_mode(how):
    def f():
        tool('m'); drag(cc(*R1[0]), cc(*R1[1]))
        if how in MODES:
            xdo('mousemove', *MODES[how], 'click', 1); time.sleep(0.3)
            drag(cc(*R2[0]), cc(*R2[1]))
            xdo('mousemove', *MODES['replace'], 'click', 1); time.sleep(0.3)
        else:
            drag(cc(*R2[0]), cc(*R2[1]), mods=how)
        return mask_now()
    return f

def c_ellipse_mode(mods):
    def f():
        tool('m'); drag(cc(*R1[0]), cc(*R1[1])); key('shift+m'); drag(cc(*R2[0]), cc(*R2[1]), mods=mods)
        return mask_now()
    return f

def c_select(what):
    def f():
        tool('m')
        if what == 'all':
            key('ctrl+a')
        elif what == 'reselect':
            drag(cc(*R1[0]), cc(*R1[1])); key('Escape'); key('ctrl+shift+d')
        elif what == 'inverse':
            drag(cc(*R1[0]), cc(*R1[1])); key('ctrl+shift+i')
        elif what == 'inverse_none':
            key('ctrl+shift+i')
        elif what == 'ctrl_d':
            drag(cc(*R1[0]), cc(*R1[1])); key('ctrl+d')
        elif what == 'click_deselects':
            drag(cc(*R1[0]), cc(*R1[1])); click(cc(20, 13))
        return mask_now()
    return f

MSHAPES = {
    'rect': lambda: drag(cc(5, 4), cc(11, 10)),
    'px': lambda: drag(cc(12, 8, .3, .3), cc(12, 8, .7, .7)),
    'edge': lambda: drag(cc(0, 0), cc(4, 3)),
    'ell': lambda: (key('shift+m'), drag(cc(13, 4), cc(21, 10)), key('m')),
    'L': lambda: (drag(cc(2, 9), cc(4, 14)), drag(cc(2, 12), cc(9, 14), mods=('shift',))),
}
MODIFY = {'border': (558, 156), 'expand': (562, 178), 'contract': (566, 200)}
def c_morph(shape, kind, n, brush):
    def f():
        tool('m'); MSHAPES[shape]()
        menu(SEL, (337, 156), MODIFY[kind]); time.sleep(0.5)
        xdo('mousemove', 853, 451, 'click', 1); time.sleep(0.2); key('ctrl+a'); xdo('type', str(n))
        xdo('mousemove', 792, 486 if brush == 'circle' else 518, 'click', 1); time.sleep(0.3)
        xdo('mousemove', 734, 568, 'click', 1); time.sleep(0.6)
        return mask_now()
    return f

def wand_opts(tol, contiguous, conn):
    a = shot('wo.png')
    on = (a[64:81, 846:866].sum(2) < 200).sum() > 20
    if on != contiguous:
        xdo('mousemove', 856, 72, 'click', 1); time.sleep(0.3)
    xdo('mousemove', 990, 72, 'click', 1); time.sleep(0.6)
    xdo('mousemove', 1030 if conn == 4 else 1130, 207, 'click', 1); time.sleep(0.3)
    xdo('mousemove', 1300, 60, 'click', 1); time.sleep(0.4)
    xdo('mousemove', 808, 72, 'click', 1); time.sleep(0.3); key('ctrl+a'); xdo('type', str(tol)); key('Return')

def c_wand(at, tol, contiguous=False, conn=8, then=None):
    def f():
        tool('w'); wand_opts(tol, contiguous, conn); click(cc(*at))
        if then:
            click(cc(*then[0]), mods=then[1])
        return mask_now()
    return f

def c_range(tol, hexc='000000'):
    def f():
        tool('m'); menu(SEL, (360, 134)); time.sleep(0.8)
        xdo('mousemove', 829, 473, 'click', 1); time.sleep(0.8)
        xdo('mousemove', 1128, 514, 'click', '--repeat', 2, '--delay', 80, 1); time.sleep(0.3)
        key('ctrl+a'); xdo('type', hexc); key('Return')
        xdo('mousemove', 755, 401, 'click', 1); time.sleep(0.5)
        if tol <= 128:
            xdo('mousemove', 813, 513, 'click', 1); time.sleep(0.3); keyn('Left', 20); keyn('Right', tol)
        else:
            xdo('mousemove', 887, 513, 'click', 1); time.sleep(0.3); keyn('Right', 20); keyn('Left', 255 - tol)
        a = shot('cr.png', park=False)
        Image.fromarray(a[500:528, 805:895].astype('uint8')).save(os.path.join(D, f'cr_tol{tol}.png'))
        xdo('mousemove', 752, 584, 'click', 1); time.sleep(0.6)
        return mask_now()
    return f

# ── transformacoes (reabrem o ficheiro) ─────────────────────────────────────────────────────────────
SELR = ((8, 4), (13, 8))
ROT = {'180': (440, 304), '90cw': (453, 326), '90ccw': (458, 348)}
SHIFT = {'left': (443, 392), 'right': (447, 414), 'up': (437, 436), 'down': (447, 458)}
CANVAS = {'180': (358, 164), '90cw': (371, 186), '90ccw': (377, 208), 'flip_h': (437, 238), 'flip_v': (428, 260)}

def finish_read():
    commit(); key('Escape', t=0.4); rebbox(); return {'w': W, 'h': H, 'rgb': rgb_now()}

def c_edit(op, sel):
    def f():
        tool('m')
        if sel == 'rect':
            drag(cc(*SELR[0]), cc(*SELR[1]))
        elif sel == 'all':
            key('ctrl+a')
        if op in ('flip_h', 'flip_v'):
            key('shift+h' if op == 'flip_h' else 'shift+v', t=0.5)
        elif op.startswith('rot'):
            menu(EDIT, (92, 304), ROT[op[3:]])
        elif op.startswith('shift_'):
            menu(EDIT, (85, 392), SHIFT[op[6:]])
        elif op.startswith('canvas_'):
            menu(SPRITE, (172, 164), CANVAS[op[7:]])
        return finish_read()
    return f

ALGS = {'fast': (571, 104), 'rotsprite': (556, 122)}
def set_alg(alg):
    xdo('mousemove', 658, 72, 'click', 1); time.sleep(0.5)
    xdo('mousemove', *ALGS[alg], 'click', 1); time.sleep(0.4)

FIELDS = {'x': 490, 'y': 547, 'w': 603, 'h': 659, 'angle': 733, 'skew': 789}
def c_xform(alg, **vals):
    def f():
        tool('m'); set_alg(alg)
        key('ctrl+a'); key('ctrl+t', t=0.5); xdo('mousemove', *cc(12, 7)); time.sleep(0.2)
        key('Right', 'Left', t=0.4)
        assert floating(), 'os campos da transformacao nao apareceram'
        for k, v in vals.items():
            xdo('mousemove', FIELDS[k], 72, 'click', 1); time.sleep(0.3); key('ctrl+a'); xdo('type', '--', str(v))
            key('Tab' if k != list(vals)[-1] else 'Return', t=0.6)
        out = finish_read()
        set_alg('fast')
        return out
    return f

METHODS = {'nearest': (787, 644), 'bilinear': (757, 662), 'rotsprite': (764, 680)}
def c_size(w, h, method):
    def f():
        tool('m'); key('ctrl+alt+i', t=1.0)
        xdo('mousemove', 767, 399, 'click', 1); time.sleep(0.2); key('ctrl+a'); xdo('type', str(w)); key('Tab')
        xdo('mousemove', 767, 437, 'click', 1); time.sleep(0.2); key('ctrl+a'); xdo('type', str(h)); key('Tab')
        xdo('mousemove', 938, 611, 'click', 1); time.sleep(0.5); xdo('mousemove', *METHODS[method], 'click', 1)
        time.sleep(0.4); xdo('mousemove', 766, 650, 'click', 1); time.sleep(1.0)
        return finish_read()
    return f

ANCHOR = {'tl': (1397, 175), 'c': (1430, 208), 'br': (1463, 241), 't': (1430, 175), 'l': (1397, 208)}
def c_canvas(w=None, h=None, anchor='c', borders=None, trim=False):
    def f():
        tool('m'); key('c', t=1.0)
        if borders:
            for (x, y), v in zip(((1305, 317), (1497, 317), (1305, 355), (1497, 355)), borders):
                xdo('mousemove', x, y, 'click', 1); time.sleep(0.2); key('ctrl+a'); xdo('type', str(v)); key('Tab')
        else:
            xdo('mousemove', *ANCHOR[anchor], 'click', 1); time.sleep(0.3)
            xdo('mousemove', 1315, 171, 'click', 1); time.sleep(0.2); key('ctrl+a'); xdo('type', str(w)); key('Tab')
            xdo('mousemove', 1315, 209, 'click', 1); time.sleep(0.2); key('ctrl+a'); xdo('type', str(h)); key('Tab')
        if trim:
            xdo('mousemove', 1205, 406, 'click', 1); time.sleep(0.3)
        xdo('mousemove', 1363, 456, 'click', 1); time.sleep(1.0)
        return finish_read()
    return f

def c_crop(shape):
    def f():
        tool('m')
        if shape == 'ell':
            key('shift+m')
        drag(cc(5, 4), cc(14, 10)); menu(SPRITE, (132, 194)); time.sleep(0.6)
        return finish_read()
    return f

def c_trim():
    def f():
        tool('m'); menu(SPRITE, (132, 216)); time.sleep(0.6)
        return finish_read()
    return f

def c_paste(kind):
    def f():
        tool('m')
        if kind in ('nudge', 'merged', 'plain'):
            src = ((2, 2), (6, 5)) if kind == 'nudge' else ((8, 4), (15, 9))
            drag(cc(*src[0]), cc(*src[1])); key('ctrl+shift+c' if kind == 'merged' else 'ctrl+c')
            key('Escape'); key('ctrl+v', t=0.6)
            fl = floating()
            keyn('Right', 10 if kind == 'nudge' else 8)
            out = finish_read(); out['floating_after_paste'] = fl
            return out
        drag(cc(2, 2), cc(6, 5)); key('ctrl+c'); drag(cc(14, 8), cc(20, 13)); key('ctrl+v', t=0.6)
        return finish_read()
    return f


# ── a lista (planeada de uma vez: CLAUDE §0.10) ─────────────────────────────────────────────────────
CASES = []
PARAMS = {}   # o que cada caso fez, em dados (lido por trial_fixture_selecao.py)
def add(name, file, fn, reopen=False, **params):
    CASES.append((name, file, fn, reopen))
    PARAMS[name] = dict(file=file, **params)

for a, b in [((2, 3), (6, 8)), ((6, 8), (2, 3)), ((2, 8), (6, 3)), ((6, 3), (2, 8)), ((5, 5), (6, 6)),
             ((10, 4), (10, 9)), ((-1, -1), (3, 2)), ((20, 12), (26, 18)), ((7, 7), (7, 7)), ((0, 0), (23, 15))]:
    add(f'rect_{a[0]}_{a[1]}__{b[0]}_{b[1]}', 'red.ase', c_rect(a, b), op='rect_marquee', press=a, release=b)
for w in range(1, 13):
    for h in range(1, 10):
        add(f'ell_{w}x{h}', 'red.ase', c_ellipse((1, 1), (w, h)), op='ellipse_marquee', press=(1, 1), release=(w, h))
for a, b in [((12, 9), (1, 1)), ((8, 1), (1, 7)), ((14, 2), (22, 14)), ((0, 0), (23, 15))]:
    add(f'ell_from_{a[0]}_{a[1]}__{b[0]}_{b[1]}', 'red.ase', c_ellipse(a, b), op='ellipse_marquee', press=a, release=b)
POLYS = {'tri': [(3, 3), (15, 3), (9, 12)], 'quad': [(3, 3), (15, 3), (15, 12), (3, 12)],
         'star': [(11, 0), (15, 13), (4, 5), (18, 5), (7, 13)], 'bowtie': [(3, 3), (15, 12), (15, 3), (3, 12)],
         'arrow': [(2, 2), (20, 2), (11, 7), (20, 13), (2, 13)], 'thin': [(2, 2), (20, 12), (21, 12)]}
for k, pts in POLYS.items():
    add(f'lasso_{k}', 'red.ase', c_lasso(pts, False), op='lasso', vertices=pts)
    add(f'plasso_{k}', 'red.ase', c_lasso(pts, True), op='polygonal_lasso', vertices=pts)
for k, how in [('shift', ('shift',)), ('alt_shift', ('alt', 'shift')), ('ctrl_shift', ('ctrl', 'shift')),
               ('ctrl', ('ctrl',)), ('btn_add', 'add'), ('btn_subtract', 'subtract'),
               ('btn_intersect', 'intersect')]:
    add(f'mode_{k}', 'red.ase', c_mode(how), True, op='rect_then_rect', first=R1, second=R2,
        modifiers=list(how) if isinstance(how, tuple) else [], button=how if isinstance(how, str) else None)
add('mode_ell_alt_shift', 'red.ase', c_ellipse_mode(('alt', 'shift')), True, op='rect_then_ellipse', first=R1,
    second=R2, modifiers=['alt', 'shift'])
for w in ['all', 'reselect', 'inverse', 'inverse_none', 'ctrl_d', 'click_deselects']:
    add(f'select_{w}', 'red.ase', c_select(w), op='select_menu', what=w, rect=R1)
MSHAPE_DESC = {'rect': [['rect', (5, 4), (11, 10)]], 'px': [['rect', (12, 8), (12, 8)]],
               'edge': [['rect', (0, 0), (4, 3)]], 'ell': [['ellipse', (13, 4), (21, 10)]],
               'L': [['rect', (2, 9), (4, 14)], ['add_rect', (2, 12), (9, 14)]]}
for shape in MSHAPES:
    for kind in MODIFY:
        for n in (1, 2, 3):
            for brush in ('circle', 'square'):
                add(f'morph_{shape}_{kind}_{n}_{brush}', 'red.ase', c_morph(shape, kind, n, brush),
                    op='modify', kind=kind, n=n, brush=brush, shape=MSHAPE_DESC[shape])
def wand(name, file, at, tol, contiguous=False, conn=8, then=None):
    add(name, file, c_wand(at, tol, contiguous, conn, then), op='magic_wand', at=at, tolerance=tol,
        contiguous=contiguous, connectivity=conn,
        then=None if not then else {'at': then[0], 'modifiers': list(then[1])})
for tol in [0, 1, 2, 7, 8, 9, 15, 16, 17, 31, 32, 33, 63, 64, 65, 100, 127, 128, 129, 200, 254, 255]:
    wand(f'wand_ref0_tol{tol}', 'wand.ase', (0, 0), tol)
for tol in [0, 1, 8, 64, 128, 254]:
    wand(f'wand_transp_tol{tol}', 'wand.ase', (0, 8), tol)
for tol in [0, 1, 2]:
    wand(f'wand_a1_tol{tol}', 'wand.ase', (22, 3), tol)
for tol in [27, 28, 30]:
    wand(f'wand_r100_tol{tol}', 'wand.ase', (17, 0), tol)
wand('wand_contig_ref0_tol8_c4', 'wand.ase', (0, 0), 8, True, 4)
for conn in (4, 8):
    wand(f'wandc_red_c{conn}', 'conn.ase', (0, 15), 0, True, conn)
    wand(f'wandc_diag_c{conn}', 'conn.ase', (12, 0), 0, True, conn)
    wand(f'wandc_island_c{conn}', 'conn.ase', (5, 5), 0, True, conn)
wand('wandc_noncontig_red', 'conn.ase', (0, 15), 0, False, 4)
wand('wandc_add_shift', 'conn.ase', (0, 15), 0, True, 4, then=((12, 0), ('shift',)))
wand('wandc_sub_alt', 'conn.ase', (0, 15), 0, False, 4, then=((5, 5), ('alt',)))
wand('wandc_sub_alt_shift', 'conn.ase', (0, 15), 0, False, 4, then=((5, 5), ('alt', 'shift')))
for tol in [0, 1, 8, 64, 128, 254, 255]:
    add(f'range_black_tol{tol}', 'wand.ase', c_range(tol), op='color_range', color='000000ff', tolerance=tol)
for op in ['flip_h', 'flip_v']:
    for sel in ['none', 'rect']:
        add(f'edit_{op}_{sel}', 'xf.ase', c_edit(op, sel), True, op=op, selection=SELR if sel == 'rect' else None)
for r in ROT:
    for sel in ['none', 'rect']:
        add(f'edit_rot{r}_{sel}', 'xf.ase', c_edit('rot' + r, sel), True, op='edit_rotate_' + r,
            selection=SELR if sel == 'rect' else None)
for s_ in SHIFT:
    for sel in ['rect', 'all']:
        add(f'edit_shift_{s_}_{sel}', 'xf.ase', c_edit('shift_' + s_, sel), True, op='edit_shift_' + s_,
            selection=SELR if sel == 'rect' else 'all')
for c in CANVAS:
    add(f'edit_canvas_{c}', 'xf.ase', c_edit('canvas_' + c, 'none'), True, op='rotate_canvas_' + c)
for alg in ALGS:
    for ang in [15, 22, 30, 33, 37, 40, 45, 90, 135, -30]:
        add(f'xform_{alg}_rot{ang}', 'xf.ase', c_xform(alg, angle=ang), True, op='transform', algorithm=alg,
            fields={'angle': ang}, selection='all (ctrl+a, ctrl+t: box = cel bounds 7x5 at (9,5))')
    for w, h in [(14, 10), (10, 7), (5, 3)]:
        add(f'xform_{alg}_scale{w}x{h}', 'xf.ase', c_xform(alg, w=w, h=h), True, op='transform', algorithm=alg,
            fields={'w': w, 'h': h}, selection='all (box 7x5 at (9,5))')
    add(f'xform_{alg}_skew20', 'xf.ase', c_xform(alg, skew=20), True, op='transform', algorithm=alg,
        fields={'skew': 20}, selection='all (box 7x5 at (9,5))')
for w, h in [(36, 24), (12, 8), (25, 13), (48, 32)]:
    for m in METHODS:
        add(f'size_{m}_{w}x{h}', 'sz.ase', c_size(w, h, m), True, op='sprite_size', w=w, h=h, method=m)
for k, kw in [('30x20_c', dict(w=30, h=20)), ('30x20_tl', dict(w=30, h=20, anchor='tl')),
              ('30x20_br', dict(w=30, h=20, anchor='br')), ('31x19_c', dict(w=31, h=19)),
              ('20x12_c', dict(w=20, h=12)), ('21x13_c', dict(w=21, h=13)),
              ('borders_3_2_0_1', dict(borders=(3, 2, 0, 1)))]:
    add(f'canvas_{k}', 'sz.ase', c_canvas(**kw), True, op='canvas_size', **kw)
add('canvas_bg_30x20_c', 'szb.ase', c_canvas(w=30, h=20), True, op='canvas_size', w=30, h=20, anchor='c')
add('crop_rect', 'sz.ase', c_crop('rect'), True, op='crop', selection=['rect', (5, 4), (14, 10)])
add('crop_ell', 'sz.ase', c_crop('ell'), True, op='crop', selection=['ellipse', (5, 4), (14, 10)])
for f_ in ['trim.ase', 'trimc.ase', 'trimcb.ase']:
    add(f'trim_{f_[:-4]}', f_, c_trim(), True, op='trim')
add('paste_nudge', 'sz.ase', c_paste('nudge'), True, op='copy_paste', copy=[(2, 2), (6, 5)],
    then='Esc, ctrl+v, Right x10, commit')
add('paste_into_sel', 'sz.ase', c_paste('into_sel'), True, op='copy_paste', copy=[(2, 2), (6, 5)],
    then='select (14,8)-(20,13), ctrl+v, commit')
add('paste_merged', 'pm.ase', c_paste('merged'), True, op='copy_merged_paste', copy=[(8, 4), (15, 9)],
    then='Esc, ctrl+v, Right x8, commit')
add('paste_plain', 'pm.ase', c_paste('plain'), True, op='copy_paste', copy=[(8, 4), (15, 9)],
    then='Esc, ctrl+v, Right x8, commit')

def main():
    pref = sys.argv[2:]
    todo = [c for c in CASES if not pref or any(c[0].startswith(p) for p in pref)]
    out_p = os.path.join(D, 'lido.json')
    out = json.load(open(out_p)) if os.path.exists(out_p) else {}
    cur = None
    for name, file, fn, reopen in todo:
        if reopen or cur != file:
            if cur is not None:
                key('Escape'); fecha()
            abre(file); cur = file
            ensure_fg()   # a barra de cores so existe com um documento aberto
        t = time.time()
        try:
            out[name] = fn()
        except Exception as e:   # um caso que falha nao para o lote; fica registado
            out[name] = {'error': repr(e)}
            key('Escape'); key('Escape')
        if reopen:
            key('Escape'); fecha(); cur = None
        print(f'{name} {time.time() - t:.1f}s', flush=True)
        json.dump(out, open(out_p, 'w'))
    if cur is not None:
        key('Escape'); fecha()
    print('FIM', len(todo))


if __name__ == '__main__':
    main()
