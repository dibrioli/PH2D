# v2 (08/11): mede larguras e monta as folhas _v2. Uso: python3 compoe_v2.py <pasta_cena> <pasta_csp_b> <saida> [medir]
# Largura [M]: linhas = no CUME do S (x≈280, y≈100, a tangente é horizontal) a EXTENSÃO vertical (1.º ao último
# píxel com luminância < 160, o grão não parte a medida), mínimo entre as colunas 272..288; mistura = a régua azul (y=255), píxeis a meio caminho
# entre o branco e o azul, mediana das colunas 60..360.
import sys, os, glob, re
import numpy as np
from PIL import Image, ImageDraw, ImageFont

A_SHEET1 = 'comparacao_csp_A_linhas_1.png'; A_SHEET2 = 'comparacao_csp_A_linhas_2.png'
CSP_A = {  # célula do CSP recortada da folha v1 (o lado CSP da v1 estava certo): folha, x0, y0
    'G-Pen': (A_SHEET1, 10, 66, 'G-pen', 8), 'Turnip Pen': (A_SHEET1, 10, 342, 'Turnip pen', 8),
    'Sketch Pencil': (A_SHEET1, 10, 618, 'Pencil', 8), 'Charcoal Stick': (A_SHEET2, 10, 66, 'Charcoal', 16),
    'Soft Airbrush': (A_SHEET2, 10, 342, 'Airbrush Soft', 30), 'Round Watercolor': (A_SHEET2, 10, 618, 'Round watercolor', 16),
    'Watercolor Basic': (A_SHEET2, 10, 618, 'Round watercolor', 16),
}

def lum(im):
    return np.asarray(im.convert('L')).astype(int)

def largura_linha(im):
    a = lum(im); best = None
    for x in range(272, 289):
        ys = np.nonzero(a[70:135, x] < 160)[0]
        m = int(ys[-1] - ys[0] + 1) if len(ys) else 0
        if m and (best is None or m < best): best = m
    return best or 0

def largura_regua(im):
    a = np.asarray(im.convert('RGB')).astype(float)
    alvo = np.array([0, 70, 190.]); branco = np.array([255, 255, 255.])
    d = np.linalg.norm(a - branco, axis=2) > 0.5 * np.linalg.norm(alvo - branco)
    ws = [int(d[228:280, x].sum()) for x in range(60, 361, 5)]
    return int(np.median(ws))

def rgba(p, w, h):
    return Image.frombytes('RGBA', (w, h), open(p, 'rb').read()).convert('RGB')

def folha(titulo, linhas, saida, ncol=2):
    try:
        f = ImageFont.truetype('/usr/share/fonts/TTF/DejaVuSans.ttf', 15)
    except OSError:
        f = ImageFont.load_default()
    cw = max(im.width for row in linhas for _, im in row); ch = max(im.height for row in linhas for _, im in row)
    W = 10 + ncol * (cw + 10); H = 40 + len(linhas) * (ch + 34)
    s = Image.new('RGB', (W, H), (38, 38, 38)); d = ImageDraw.Draw(s)
    d.text((10, 10), titulo, fill='white', font=f)
    for r, row in enumerate(linhas):
        for c, (rot, im) in enumerate(row):
            x = 10 + c * (cw + 10); y = 40 + r * (ch + 34)
            d.text((x, y), rot, fill='white', font=f); s.paste(im, (x, y + 24))
    s.save(saida)

if __name__ == '__main__':
    cena, cspb, saida = sys.argv[1], sys.argv[2], sys.argv[3]
    medir = len(sys.argv) > 4
    here = os.path.dirname(os.path.abspath(__file__))
    out = os.path.join(cena, 'out'); pares = []
    for nome, (sh, x0, y0, rot_csp, diam) in CSP_A.items():
        csp = Image.open(os.path.join(here, sh)).convert('RGB').crop((x0, y0, x0 + 420, y0 + 240))
        wc = largura_linha(csp); cand = []
        for p in glob.glob(os.path.join(out, f"v2a_{nome.replace(' ', '_')}_r*.rgba")):
            r = float(re.search(r'_r([0-9.]+)\.rgba', p).group(1)); im = rgba(p, 420, 240)
            cand.append((r, largura_linha(im), im))
        cand.sort(key=lambda t: t[0])
        print(f'{nome}: CSP {rot_csp} {diam}px -> {wc}px | PH2D', ', '.join(f'r{r:g}={w}px' for r, w, _ in cand))
        r, w, im = min(cand, key=lambda t: (abs(t[1] - wc), abs(t[0] - diam / 2)))
        pares.append(((f'CSP: {rot_csp} {diam}px  [M] {wc}px', csp), (f'PH2D: {nome} Size {2*r:g} (r {r:g})  [M] {w}px', im)))
    if medir:
        sys.exit(0)
    folha('A v2. Linhas, mesma largura MEDIDA (cume do S) - esquerda Clip Studio 1.13.2, direita PH2D', pares[:4], os.path.join(saida, 'comparacao_csp_A_linhas_1_v2.png'))
    folha('A v2. Linhas, mesma largura MEDIDA (cume do S) - esquerda Clip Studio 1.13.2, direita PH2D', pares[4:], os.path.join(saida, 'comparacao_csp_A_linhas_2_v2.png'))
    B = [('Round mixing brush', 'pigment_hard_round', 'Hard Round + Pigment mix'),
         ('Gouache', 'composite_dono', 'Composite (receita do dono)'),
         ('Oil paint', 'watercolor_basic_pigment', 'Watercolor Basic, Pigment mixing 1'),
         ('Thick oil paint', 'dabber', 'Dabber'),
         ('Blend', 'smear_basic', 'Smear Basic'),
         ('Finger tip', 'blender_soft', 'Blender Soft')]
    pb = []
    for i, (c, rot, p) in enumerate(B):
        ci = Image.open(os.path.join(cspb, f'csp_b{i}.png')).convert('RGB'); pi = rgba(os.path.join(out, f'v2b_{rot}.rgba'), 420, 280)
        wc, wp = largura_regua(ci), largura_regua(pi)
        print(f'B {c}: CSP {wc}px | PH2D {p}: {wp}px')
        lc = f'CSP: {c} 30  [M] {wc}px' if i < 4 else f'CSP: {c} 30'
        lp = f'PH2D: {p} Size 30  [M] {wp}px' if i < 4 else f'PH2D: {p} Size 30'
        pb.append(((lc, ci), (lp, pi)))
    t = 'B v2. Amarelo sobre azul seco, azul sobre amarelo e vermelho secos (+ regua azul) - esquerda CSP, direita PH2D'
    folha(t, pb[:3], os.path.join(saida, 'comparacao_csp_B_mistura_1_v2.png'))
    folha(t, pb[3:], os.path.join(saida, 'comparacao_csp_B_mistura_2_v2.png'))
