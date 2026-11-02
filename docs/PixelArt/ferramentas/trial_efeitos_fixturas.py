"""Monta as 4 fixturas oracle_trial_{gradient,fill_stroke,outline,blur}.json a partir das saidas de
trial_efeitos_ferramentas.py (corrida 1: grad/fs/outline/blur.json; corrida 2: *2.json) e verifica as leis.
uso: python3 -I trial_efeitos_fixturas.py <dir_das_saidas> crates/ph2d-pixel-core/tests/fixtures
"""
import json, math, os, sys
W = sys.argv[1]; F = sys.argv[2]
L = lambda n: json.load(open(os.path.join(W, n))) if os.path.exists(os.path.join(W, n)) else {}
CMD = 'PH2D_TRIAL_DISPLAY=:93 python3 -I docs/PixelArt/ferramentas/trial_efeitos_ferramentas.py <dir> {}'
SRC = 'Aseprite Trial 1.3.18.6 (caixa preta, so corrido; nunca o fonte)'
METH = ('Xvfb :93 1600x1000, screen_scale 2, HOME isolado; sprites de entrada NOSSOS escritos como .ase pela spec '
        'publica e abertos por ctrl+o (o Trial nao grava); zoom 400 % = 8 px de tela por celula; arrastos por xdotool '
        '(trial_elipse_grelha.drag); leitura do centro de cada celula na fotografia (import -window root); '
        'cada grelha = linhas de hex RRGGBB separadas por espaco. Transparente le-se como o xadrez da vista (80/c0..).')
def rows(g): return [' '.join(r) for r in g]
def rep(a, b, keyf):
    same = [k for k in a if k in b and keyf(a[k]) == keyf(b[k])]
    diff = [k for k in a if k in b and keyf(a[k]) != keyf(b[k])]
    return {'cases_compared': len(same) + len(diff), 'identical': len(same), 'different': diff}
def dump(name, d):
    p = os.path.join(F, name); json.dump(d, open(p, 'w'), indent=1, ensure_ascii=False); print(p, os.path.getsize(p))


# ── 2.a rodada (02/11, Xvfb :95): `... trial_efeitos_fixturas.py <dir95> <fixtures> r2 <leis.json>` acrescenta
# chaves NOVAS as fixturas existentes (nada se apaga); <leis.json> = {fixtura: {chave: valor}} montado pelo driver
if len(sys.argv) > 4 and sys.argv[3] == 'r2':
    leis = json.load(open(sys.argv[4]))
    for name, add in leis.items():
        p = os.path.join(F, name); d = json.load(open(p))
        clash = [k for k in add if k in d and k not in ('method', 'command')]
        assert not clash, (name, clash)
        for k in ('method', 'command'):
            if k in add:
                d[k] = d[k] + ' || RODADA 2: ' + add.pop(k)
        d.update(add); dump(name, d)
    sys.exit(0)

# ── gradient
g1, g2 = L('grad.json'), L('grad2.json')
T = {'h32x8': lambda i, j: i / 31, 'h': lambda i, j: i / 31, 'h_curto': lambda i, j: (i - 8) / 15,
     'h_fora': lambda i, j: (i + 8) / 47, 'v': lambda i, j: j / 31, 'diag': lambda i, j: (i + j) / 62,
     'radial': lambda i, j: math.hypot((i - 16) / 8, (j - 16) / 8),
     'radial_elipse': lambda i, j: math.hypot((i - 16) / 12, (j - 16) / 4)}
def tv(nome, i, j): return math.floor(255 * max(0.0, min(1.0, T[nome](i, j))))
solid_bad = sum(int(c[:2], 16) != tv(k.split('/')[1], i, j) or c[2:] != '0000'
                for k, v in g1.items() if k.startswith('solid/') for j, r in enumerate(v['grid']) for i, c in enumerate(r))
thr = {}
for f, n in (('bayer2', 2), ('bayer4', 4), ('bayer8', 8)):
    lo, hi, conf = {}, {}, 0
    for k, v in g1.items():
        if not k.startswith(f + '/'): continue
        nome = k.split('/')[1]
        for j, r in enumerate(v['grid']):
            for i, c in enumerate(r):
                m = (i % n, j % n); x = tv(nome, i, j)
                if c == '000000': lo[m] = max(lo.get(m, -1), x)
                elif c == 'ff0000': hi[m] = min(hi.get(m, 256), x)
                else: conf += 1
    bad = sum(1 for m in lo if m in hi and lo[m] >= hi[m])
    thr[f] = {'anchor': 'celula (x mod n, y mod n) do CANVAS', 'conflicts_with_canvas_anchor': bad, 'other_colors': conf,
              'interval_v_black_max__v_red_min': [[[lo.get((x, y), -1), hi.get((x, y), 256)] for x in range(n)] for y in range(n)]}
cases = {k: dict(v, grid=rows(v['grid'])) for k, v in g1.items()}
dump('oracle_trial_gradient.json', {
    'source': SRC, 'method': METH + ' Gradient = Shift+G; barra: Linear/Radial, lista No Dithering|Bayer 2x2|4x4|8x8 '
    '(foto docs/PixelArt/aseprite/71_gradiente_barra.png). Canvas BRANCO opaco, Tolerance 0 + Contiguous (a regiao = o canvas '
    'inteiro). FG #000000, BG #ff0000. from_cell/to_cell = centro da celula onde se prime/solta (podem cair fora do canvas).',
    'command': CMD.format('grad solid bayer2 bayer4 bayer8'), 'date': '2026-11-02',
    'law_solid': 'v = floor(255*t), t = clamp01; LINEAR: t = projeccao do centro (i,j) sobre a->b: ((p-a).(b-a))/|b-a|^2 '
    '(celulas inteiras); RADIAL: centro = PONTO MEDIO do arrasto, raios = |bx-ax|/2 e |by-ay|/2 (ELIPSE inscrita na caixa '
    'do arrasto), t = hypot(dx/rx, dy/ry) — arrasto horizontal (ry=0) pinta tudo de BG. FG em t=0, BG em t=1. '
    f'Celulas que violam a lei em todos os casos solid: {solid_bad}. Aviso: FG/BG 0->255 num so canal nao separa '
    'floor(a+(b-a)t) de outras formas de mistura com a!=0 (nao medido).',
    'law_dither': 'celula = BG se v >= limiar(x mod n, y mod n) senao FG; limiar compativel com 255*(M[y][x]+1)/(n*n+1), '
    'M = Bayer padrao (2x2: [[0,2],[3,1]]); ancora no (0,0) do CANVAS (zero conflitos com arrastos curtos, longos, fora do '
    'canvas e radiais). O intervalo medido de cada limiar vai em dither_thresholds.',
    'dither_thresholds': thr,
    'reproducibility_run2': rep(g1, g2, lambda v: v['grid']),
    'not_measured': ['Indexed (Sprite > Color Mode > Indexed): nao corrido por falta de tempo (prazo 45 min)',
                     'mistura do Solid com FG/BG com canais != 0/255 (arredondamento de a+(b-a)t)'],
    'cases': cases})

# ── fill / stroke
f1, f2 = L('fs.json'), L('fs2.json')
def mask(r0, r1, ell_grid):
    return None
fill = f1.get('f1')
chk = {}
if fill:
    H, Wd = len(fill), len(fill[0])
    M = [[fill[j][i] == '000000' for i in range(Wd)] for j in range(H)]
    border = [[M[j][i] and any(not (0 <= j + dj < H and 0 <= i + di < Wd) or not M[j + dj][i + di]
                               for di, dj in ((1, 0), (-1, 0), (0, 1), (0, -1))) for i in range(Wd)] for j in range(H)]
    for k in ('s1', 's3', 's5'):
        if k in f1:
            S = [[f1[k][j][i] == '000000' for i in range(Wd)] for j in range(H)]
            chk[k] = sum(S[j][i] != border[j][i] for j in range(H) for i in range(Wd))
dump('oracle_trial_fill_stroke.json', {
    'source': SRC, 'method': METH + ' Canvas branco 32x16, FG #000000. Rectangular Marquee (M) arrastado de (2,2) a (11,9) '
    '[10x8] e Elliptical (Shift+M) de (16,2) a (27,13) [12x12] no mesmo documento; Edit > Fill = tecla F, Edit > Stroke = '
    'tecla S (sem dialogo), ctrl+d entre as duas. Tamanho do Pencil posto a 1/3/5 antes (campo da barra, conferido a 5 px '
    'na foto).', 'command': CMD.format('fillstroke'), 'date': '2026-11-02',
    'law_fill': 'pinta exactamente a mascara da seleccao com o FG',
    'law_stroke': 'borda INTERNA de 1 celula: celulas da mascara com algum vizinho-4 fora da mascara; NAO depende do '
    f'tamanho do pincel (1, 3 e 5 dao o mesmo). Celulas que violam a lei por caso: {chk}',
    'reproducibility_run2': rep(f1, f2, lambda v: v),
    'cases': {k: {'op': {'f': 'Fill', 's': 'Stroke'}[k[0]], 'brush_size': int(k[1:]), 'rect': [[2, 2], [11, 9]],
                  'ellipse': [[16, 2], [27, 13]], 'grid': rows(v)} for k, v in f1.items()}})

# ── outline
o1, o2 = L('outline.json'), L('outline2.json')
SHAPE = {(6, 6), (7, 6), (8, 6), (6, 7), (6, 8), (7, 8)}
NB = {'circle': [(0, -1), (-1, 0), (1, 0), (0, 1)], 'square': [(a, b) for a in (-1, 0, 1) for b in (-1, 0, 1) if (a, b) != (0, 0)],
      'horizontal': [(-1, 0), (1, 0)], 'vertical': [(0, -1), (0, 1)]}
viol = {}
for k, v in o1.items():
    doc, pre, place = k.split('/')
    g = v['grid']; bad = 0
    for j in range(16):
        for i in range(16):
            c = g[j][i]; s = (i, j) in SHAPE
            if doc == 'shape_w':
                exp = '0000ff' if s else 'ffffff'
            elif place == 'outside':
                exp = '0000ff' if s else ('000000' if any((i + a, j + b) in SHAPE for a, b in NB[pre]) else 'T')
            else:
                exp = ('000000' if any((i + a, j + b) not in SHAPE for a, b in NB[pre]) else '0000ff') if s else 'T'
            got = c if c in ('000000', '0000ff', 'ffffff') else 'T'
            bad += got != exp
    viol[k] = bad
dump('oracle_trial_outline.json', {
    'source': SRC, 'method': METH + ' Forma de 6 celulas AZUIS #0000ff (6,6)(7,6)(8,6)(6,7)(6,8)(7,8) num 16x16 de fundo '
    'TRANSPARENTE (shape_t) ou BRANCO opaco (shape_w). Edit > FX > Outline (Shift+O); dialogo na foto '
    'docs/PixelArt/aseprite/70_outline_dialogo.png: Outline Color = FG #000000, Background Color = Mask, 4 presets da '
    'matriz (circle=cruz 4-viz, square=8-viz, horizontal, vertical) + matriz 3x3 editavel, Outside/Inside, canais RGBA, '
    'Preview, Tiled. matrix_read = a matriz 3x3 lida no dialogo apos o clique no preset. Transparente na grelha = xadrez '
    '(lido como T na verificacao).', 'command': CMD.format('outline'), 'date': '2026-11-02',
    'law': 'fundo = Background Color (Mask = alfa 0). OUTSIDE: celula de FUNDO com algum vizinho da matriz na forma -> '
    'Outline Color. INSIDE: celula da FORMA com algum vizinho da matriz no fundo -> Outline Color. Le o original (nao '
    'propaga). Sobre branco opaco com Background=Mask NADA muda (nao ha fundo). '
    f'Celulas que violam a lei por caso: {viol}',
    'reproducibility_run2': rep(o1, o2, lambda v: v['grid']),
    'not_measured': ['Background Color = branco (o seletor de cor do dialogo) e Outline Color diferente do FG: sem tempo; '
                     'a cor do contorno e o FG do momento', 'matriz 3x3 personalizada, Tiled, canais'],
    'cases': {k: dict(v, grid=rows(v['grid'])) for k, v in o1.items()}})

# ── blur / jumble
b1, b2 = L('blur.json'), L('blur2.json')
def counts(g): 
    fl = [c for r in g for c in r]; return {'000000': fl.count('000000'), 'ffffff': fl.count('ffffff'), 'other': len(fl) - fl.count('000000') - fl.count('ffffff')}
dump('oracle_trial_blur.json', {
    'source': SRC, 'method': METH + ' Xadrez 16x16 (celula (i+j) par = preto, impar = branco). Blur/Jumble = tecla R '
    '(alterna Blur Tool <-> Jumble Tool, conferido na barra de estado; fotos 72_/73_), tamanho no campo da barra, '
    'Opacity 100 %, pincel circulo (tamanho 3 = cruz). Blur: um clique em (8,8) (x2 = dois cliques). Jumble: 3 arrastos '
    'iguais (2,8)->(13,8).', 'command': CMD.format('blur'), 'date': '2026-11-02',
    'law_blur': 'cada celula coberta pelo pincel = floor(media 3x3 do ORIGINAL do carimbo) por canal: preto com 4 brancos '
    '-> floor(4*255/9)=113 (0x71); branco com 5 brancos -> floor(5*255/9)=141 (0x8d, nao 142 = floor e nao round); '
    'o 2.o clique le o resultado do 1.o: floor((4*255+113)/9)=125 (0x7d). Tamanho 3 = cruz de 5 celulas, todas lidas do '
    'estado antes do carimbo.',
    'law_jumble': 'so cores ja presentes (sem mistura); NAO conserva o multiconjunto (ver counts); o raio de busca: as '
    'celulas so mudam dentro da faixa do pincel; aleatorio por corrida (ver reproducibility_run2).',
    'reproducibility_run2': rep(b1, b2, lambda v: v['grid']),
    'run2_counts': {k: counts(v['grid']) for k, v in b2.items()},
    'refusal_jumble': 'a grelha exacta do Jumble NAO reproduz entre corridas (aleatorio): so a lei estatistica vale; o raio exacto da busca nao se le num xadrez binario',
    'cases': {k: dict(v, grid=rows(v['grid']), counts=counts(v['grid']), counts_before={'000000': 128, 'ffffff': 128, 'other': 0})
              for k, v in b1.items()}})
