# v2 (08/11): a cena de MISTURA com cor de arrasto colorida, sobre blocos SECOS idênticos nos dois apps.
# Uso: python3 cena_v2.py <pasta>  -> base_csp.png (tela do CSP), base_celula.rgba (420x280, PH2D),
#      a.txt (linhas, cena A da v1), y.txt (amarelo sobre o azul), b.txt (azul sobre amarelo e vermelho + traço de régua)
import sys, os
from PIL import Image, ImageDraw
from cena import cena_a, lerp_poly

W, H = 420, 280
AZUL, AMAR, VERM = (0, 70, 190), (240, 200, 0), (200, 20, 20)
BLOCOS = [(15, 135, AZUL), (150, 270, AMAR), (285, 405, VERM)]
CELULAS = [(10 + c * 430, 10 + r * 290) for r in range(2) for c in range(3)]  # 6 pincéis do CSP
SWATCH = {'amarelo': (35, 615), 'azul': (95, 615)}  # centro das amostras (conta-gotas Alt+clique)

def arrastos(x0, x1):
    s = []
    for i, y in enumerate((50, 90, 130, 170)):
        s.append([(x0 - 10, y), (x1 + 10, y)] if i % 2 == 0 else [(x1 + 10, y), (x0 - 10, y)])
    s.append([(x0 - 5, 25), (x1 + 5, 215)]); s.append([(x1 + 5, 25), (x0 - 5, 215)])
    return s

def cena_y():
    return arrastos(15, 135)

def cena_b():
    return arrastos(150, 270) + arrastos(285, 405) + [[(30, 255), (390, 255)]]  # o último: a régua da largura

def celula():
    im = Image.new('RGB', (W, H), 'white'); d = ImageDraw.Draw(im)
    for x0, x1, c in BLOCOS:
        d.rectangle([x0, 20, x1 - 1, 219], fill=c)
    return im

def escreve(p, strokes):
    with open(p, 'w') as f:
        for s in strokes:
            f.write(' '.join(f'{x:.2f},{y:.2f}' for x, y in lerp_poly([tuple(q) for q in s])) + '\n')

def xdotool(strokes, X, Y):
    out = []
    for s in strokes:
        p = lerp_poly([tuple(q) for q in s])
        out += [f"mousemove {round(X+p[0][0])} {round(Y+p[0][1])}", "sleep 0.05", "mousedown 1", "sleep 0.03"]
        for x, y in p[1:]:
            out += [f"mousemove {round(X+x)} {round(Y+y)}", "sleep 0.006"]
        out += ["sleep 0.03", "mouseup 1", "sleep 0.15"]
    return '\n'.join(out) + '\n'

if __name__ == '__main__':
    d = sys.argv[1]; os.makedirs(d, exist_ok=True)
    c = celula(); open(os.path.join(d, 'base_celula.rgba'), 'wb').write(c.convert('RGBA').tobytes())
    t = Image.new('RGB', (1300, 650), 'white')
    for x, y in CELULAS: t.paste(c, (x, y))
    dd = ImageDraw.Draw(t)
    dd.rectangle([20, 600, 50, 630], fill=AMAR); dd.rectangle([80, 600, 110, 630], fill=AZUL)
    t.save(os.path.join(d, 'base_csp.png'))
    escreve(os.path.join(d, 'a.txt'), cena_a()); escreve(os.path.join(d, 'y.txt'), cena_y()); escreve(os.path.join(d, 'b.txt'), cena_b())
