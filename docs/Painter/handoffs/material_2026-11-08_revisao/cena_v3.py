# Cena v3 (08/11, revisão dos 22 pincéis CSP): UMA tela por pincel, a MESMA nos dois apps.
# Colunas (800 px cada; no CSP, uma tela base_<fundo>.png por coluna): (a) camada VAZIA (alfa 0, sobre o papel) · (b) branco opaco · (c) blocos opacos
# azul 0046BE | amarelo F0C800 | vermelho C81414. Linhas (220 px): reta · S · zigue-zague · toque único.
# Uso: python3 cena_v3.py <pasta>  -> base.png (RGBA), base.rgba (cru, 2400x880x4), tracos.txt (um traço por linha,
#      já interpolado a 3 px; o toque é um ponto só), pinceis.tsv (CSP nome · tamanho CSP · PH2D nome · lote).
#      python3 cena_v3.py --xdotool <X0> <Y0> <escala> [coluna 0|1|2]  -> script xdotool (ecrã = X0 + escala*x, Y0 + escala*y).
import math, os, sys

W, H, COL, LIN = 2400, 880, 800, 220
AZUL, AMAR, VERM = (0, 70, 190), (240, 200, 0), (200, 20, 20)
COR_DO_PINCEL = (0, 160, 60)  # o verde do oráculo de 08/11

# (nome no CSP, diâmetro de fábrica do CSP, nome do preset PH2D, lote do agente)
PINCEIS = [
    ("G-pen", 10, "G-Pen", 1), ("Turnip pen", 8, "Turnip Pen", 1), ("Soft", 150, "Soft Airbrush", 1),
    ("Pencil", 10, "Sketch Pencil", 1), ("Charcoal", 20, "Charcoal Stick", 1),
    ("Round watercolor", 80, "Round Watercolor", 1), ("Wet wash", 30, "Wet Wash Mix", 1),
    ("Watercolor splash", 300, "Watercolor Splash", 1),
    ("Round mixing brush", 50, "Round Mixing", 2), ("Gouache", 80, "Gouache", 2),
    ("Dry Gouache", 80, "Dry Gouache", 2), ("Thin Gouache", 20, "Thin Gouache", 2),
    ("Gouache blender", 80, "Gouache Blender", 2), ("Oil paint", 80, "Oil Paint", 2),
    ("Paint and apply", 40, "Paint And Apply", 2),
    ("Thick oil paint", 60, "Thick Oil Paint", 3), ("Crayon", 30, "Crayon", 3), ("Pastel", 40, "Pastel", 3),
    ("Chalk", 30, "Chalk", 3), ("Pointillism", 80, "Pointillism", 3), ("Brush pen", 70, "Brush Pen", 3),
    ("Finger tip", 30, "Finger Tip", 3),
]


def lerp_poly(pts, step=3.0):
    out = [pts[0]]
    for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
        n = max(1, int(math.hypot(x1 - x0, y1 - y0) / step))
        out += [(x0 + (x1 - x0) * k / n, y0 + (y1 - y0) * k / n) for k in range(1, n + 1)]
    return out


def tracos():
    s = []
    for c in range(3):
        x0 = c * COL
        s.append(lerp_poly([(x0 + 100, 110), (x0 + 700, 110)]))  # reta
        s.append(lerp_poly([(x0 + 100 + u * 600, 330 - 65 * math.sin(u * 2 * math.pi)) for u in [i / 120 for i in range(121)]]))  # S
        s.append(lerp_poly([(x0 + 100 + i * 75, 495 if i % 2 == 0 else 605) for i in range(9)]))  # zigue-zague
        s.append([(x0 + 400.0, 770.0)])  # toque único
    return s


def base():
    px = bytearray(W * H * 4)
    for y in range(H):
        for x in range(W):
            i = (y * W + x) * 4
            if x < COL:
                continue  # (a) vazio: 0,0,0,0
            if x < 2 * COL:
                rgb = (255, 255, 255)
            else:
                k = min(2, (x - 2 * COL) * 3 // COL)
                rgb = (AZUL, AMAR, VERM)[k]
            px[i:i + 4] = bytes((*rgb, 255))
    return bytes(px)


if __name__ == '__main__':
    if sys.argv[1] == '--xdotool':
        X, Y, e = float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4])
        col = int(sys.argv[5]) if len(sys.argv) > 5 else None  # só os 4 traços da coluna, em coordenadas locais
        for j, p in enumerate(tracos()):
            if col is not None:
                if j // 4 != col:
                    continue
                p = [(x - col * COL, y) for x, y in p]
            q = [(round(X + e * x), round(Y + e * y)) for x, y in p]
            print(f"mousemove {q[0][0]} {q[0][1]}"); print("sleep 0.05"); print("mousedown 1"); print("sleep 0.03")
            for x, y in q[1:]:
                print(f"mousemove {x} {y}"); print("sleep 0.006")
            print("sleep 0.03"); print("mouseup 1"); print("sleep 0.2")
        sys.exit(0)
    d = sys.argv[1]; os.makedirs(d, exist_ok=True)
    b = base(); open(os.path.join(d, 'base.rgba'), 'wb').write(b)
    from PIL import Image
    im = Image.frombytes('RGBA', (W, H), b); im.save(os.path.join(d, 'base.png'))
    for k, n in enumerate(('a_vazio', 'b_branco', 'c_blocos')):
        im.crop((k * COL, 0, (k + 1) * COL, H)).save(os.path.join(d, f'base_{n}.png'))
    with open(os.path.join(d, 'tracos.txt'), 'w') as f:
        for p in tracos():
            f.write(' '.join(f'{x:.2f},{y:.2f}' for x, y in p) + '\n')
    with open(os.path.join(d, 'pinceis.tsv'), 'w') as f:
        for n, t, p, l in PINCEIS:
            f.write(f'{n}\t{t}\t{p}\t{l}\n')
    print(d)
