"""O VETOR da amostra de arte (plano 06 §3) — o boneco verde de capacete e o cenário do Kenney Pixel Platformer
(CC0), redesenhados em vetor PLANO: formas arredondadas, cor chapada, UM tom mais escuro para o volume, olhos
brancos com pupila, sem contorno.

    python3 -I docs/GameEngineFuncional/ferramentas/arte_svg.py <poses.json do boneco_blender.py> <pasta de saída>

Escreve `boneco.svg` (a folha 8 × 4: parado · correr · saltar · cair, e agachado · a andar agachado nas células
livres das linhas 2 e 3, cada desenho na pose que o Blender deu),
`chao.svg`, `plataforma.svg`, `terra.svg` (azulejos que repetem sem costura), `moeda.svg` (6 × 1, a girar) e
`bandeira.svg` (4 × 1, a ondular). Só `fill` e caminhos absolutos — nada de `transform`, `stroke`, `opacity` ou
`clipPath` —, para o SVG abrir no PH2D como vetor editável e rasterizar igual em qualquer leitor.
"""

import json
import math
import os
import sys

K = 0.5522847498  # alça de Bézier de um quarto de círculo

# Paleta do pacote (histogramas dos PNG) e os tons escuros derivados dela.
P = {
    "verde_claro": "#5AD28C",
    "verde_meio": "#34995E",
    "verde_escuro": "#346547",
    "vidro": "#BEDEF1",
    "vidro_escuro": "#9CC4DF",
    "branco": "#FFFFFF",
    "olho": "#434A5F",
    "relva": "#36E377",
    "relva_escura": "#2EB082",
    "terra": "#CB815E",
    "terra_escura": "#9F5A52",
    "ouro": "#F4B41B",
    "ouro_escuro": "#CC9127",
    "ouro_claro": "#FEE481",
    "bandeira": "#FC683B",
    "bandeira_escura": "#DD442C",
    "mastro": "#959AB1",
    "mastro_claro": "#DCE1E7",
}

COLS, ROWS, CEL = 8, 4, 100.0  # a folha do boneco, em unidades do SVG por célula
# Onde cada ciclo começa na folha (linha, coluna). O agachar (plano 10) ocupa as células que jump e fall
# deixam livres: a grelha gravada nos ficheiros (8 × 4) não muda.
POSICOES = [("idle", 0, 0), ("run", 1, 0), ("jump", 2, 0), ("fall", 3, 0), ("crouch", 2, 4), ("crouch_walk", 3, 4)]


def f(v):
    return f"{v:.2f}".rstrip("0").rstrip(".")


class Caminho:
    """Um caminho de cúbicas em coordenadas locais, levado ao SVG por uma função de ponto."""

    def __init__(self):
        self.ops = []

    def m(self, p):
        self.ops.append(("M", [p]))
        return self

    def l(self, p):
        self.ops.append(("L", [p]))
        return self

    def c(self, a, b, p):
        self.ops.append(("C", [a, b, p]))
        return self

    def z(self):
        self.ops.append(("Z", []))
        return self

    def svg(self, para, cor):
        d = " ".join(op + " ".join(f"{f(x)},{f(y)}" for x, y in map(para, pts)) for op, pts in self.ops)
        return f'<path fill="{cor}" d="{d}"/>'


def rrect(cx, cy, w, h, r, cam=None):
    """Retângulo arredondado (y para CIMA), começa no topo e anda no sentido horário."""
    cam = cam or Caminho()
    x0, x1, y0, y1 = cx - w / 2, cx + w / 2, cy - h / 2, cy + h / 2
    r = min(r, w / 2, h / 2)
    k = r * K
    cam.m((x0 + r, y1)).l((x1 - r, y1)).c((x1 - r + k, y1), (x1, y1 - r + k), (x1, y1 - r))
    cam.l((x1, y0 + r)).c((x1, y0 + r - k), (x1 - r + k, y0), (x1 - r, y0))
    cam.l((x0 + r, y0)).c((x0 + r - k, y0), (x0, y0 + r - k), (x0, y0 + r))
    cam.l((x0, y1 - r)).c((x0, y1 - r + k), (x0 + r - k, y1), (x0 + r, y1)).z()
    return cam


def elipse(cx, cy, rx, ry, rot=0.0, cam=None):
    cam = cam or Caminho()
    cs, sn = math.cos(rot), math.sin(rot)

    def p(u, v):
        return (cx + u * cs - v * sn, cy + u * sn + v * cs)

    cam.m(p(rx, 0))
    for (a, b, c) in [
        ((rx, ry * K), (rx * K, ry), (0, ry)),
        ((-rx * K, ry), (-rx, ry * K), (-rx, 0)),
        ((-rx, -ry * K), (-rx * K, -ry), (0, -ry)),
        ((rx * K, -ry), (rx, -ry * K), (rx, 0)),
    ]:
        cam.c(p(*a), p(*b), p(*c))
    return cam.z()


def capsula(a, b, r):
    """A forma de uma perna: o segmento a→b com meia-grossura r e pontas redondas."""
    dx, dy = b[0] - a[0], b[1] - a[1]
    n = math.hypot(dx, dy) or 1e-9
    ux, uy = dx / n, dy / n
    nx, ny = -uy * r, ux * r
    tx, ty = ux * r * K, uy * r * K
    cam = Caminho()
    cam.m((a[0] + nx, a[1] + ny)).l((b[0] + nx, b[1] + ny))
    cam.c((b[0] + nx + tx, b[1] + ny + ty), (b[0] + ux * r + nx * K, b[1] + uy * r + ny * K), (b[0] + ux * r, b[1] + uy * r))
    cam.c((b[0] + ux * r - nx * K, b[1] + uy * r - ny * K), (b[0] - nx + tx, b[1] - ny + ty), (b[0] - nx, b[1] - ny))
    cam.l((a[0] - nx, a[1] - ny))
    cam.c((a[0] - nx - tx, a[1] - ny - ty), (a[0] - ux * r - nx * K, a[1] - uy * r - ny * K), (a[0] - ux * r, a[1] - uy * r))
    cam.c((a[0] - ux * r + nx * K, a[1] - uy * r + ny * K), (a[0] + nx - tx, a[1] + ny - ty), (a[0] + nx, a[1] + ny))
    return cam.z()


def svg(w, h, corpo, titulo):
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{f(w)}" height="{f(h)}" viewBox="0 0 {f(w)} {f(h)}">\n'
        f"<!-- {titulo} — gerado por docs/GameEngineFuncional/ferramentas/arte_svg.py; arte de "
        f"Kenney Pixel Platformer (CC0) redesenhada. -->\n" + "\n".join(corpo) + "\n</svg>\n"
    )


# ---------------------------------------------------------------- o boneco

def boneco(poses):
    cab = poses["cabecalho"]
    sola_margem = cab["celula"]["sola_margem"]
    m = cab["medidas"]
    perna_r = m["perna_r"]
    corpo = []
    for nome, row, col0 in POSICOES:
        for i, d in enumerate(poses["ciclos"][nome]["desenhos"]):
            col = col0 + i
            ox, oy = col * CEL, row * CEL

            def mundo(p, ox=ox, oy=oy):
                return (ox + (p[0] + 0.5) * CEL, oy + (1.0 - sola_margem - p[1]) * CEL)

            def afim(mat, para=mundo):
                a, b, c, dd, e, ff = mat
                return lambda p: para((a * p[0] + c * p[1] + e, b * p[0] + dd * p[1] + ff))

            tronco, cabeca = afim(d["tronco"]), afim(d["cabeca"])
            g = [f'<g id="{nome}_{i}">']
            pe = d["pernas"]
            g.append(capsula(*map(mundo, pe["l"][0:2]), perna_r * CEL).svg(lambda p: p, P["verde_escuro"]))
            g.append(capsula(*map(mundo, pe["l"][1:3]), perna_r * CEL).svg(lambda p: p, P["verde_escuro"]))
            tw, th = m["tronco"]
            g.append(rrect(0, 0.045, tw - 0.04, th, 0.055).svg(tronco, P["verde_meio"]))
            g.append(capsula(*map(mundo, pe["r"][0:2]), perna_r * CEL).svg(lambda p: p, P["verde_claro"]))
            g.append(capsula(*map(mundo, pe["r"][1:3]), perna_r * CEL).svg(lambda p: p, P["verde_claro"]))
            r = m["capacete_r"]
            g.append(elipse(0, 0.03, r, r).svg(cabeca, P["vidro_escuro"]))
            g.append(elipse(-0.022, 0.052, r - 0.032, r - 0.032).svg(cabeca, P["vidro"]))
            cw, ch = m["cabeca"]
            g.append(rrect(0.03, -0.01, cw, ch, 0.08).svg(cabeca, P["verde_meio"]))
            g.append(rrect(0.05, 0.0075, cw - 0.04, ch - 0.035, 0.08).svg(cabeca, P["verde_claro"]))
            for (x, rx) in ((0.07, 0.042), (0.158, 0.034)):
                g.append(elipse(x, 0.03, rx, 0.052).svg(cabeca, P["branco"]))
                g.append(elipse(x + 0.013, 0.024, 0.022, 0.026).svg(cabeca, P["olho"]))
            g.append(rrect(0.118, -0.075, 0.046, 0.018, 0.009).svg(cabeca, P["olho"]))
            g.append(elipse(-0.135, 0.175, 0.055, 0.024, math.radians(48)).svg(cabeca, P["branco"]))
            g.append(elipse(-0.075, 0.232, 0.016, 0.016).svg(cabeca, P["branco"]))
            g.append("</g>")
            corpo.extend(g)
    return svg(COLS * CEL, ROWS * CEL, corpo, "boneco de fábrica 8×4 (idle·run·jump·fall · crouch·crouch_walk)")


# ---------------------------------------------------------------- o cenário (y para BAIXO, unidades = SVG)

def ident(p):
    return p


def recorte(x0, y0, x1, y1):
    return f'<path fill="{{}}" d="M{f(x0)},{f(y0)} H{f(x1)} V{f(y1)} H{f(x0)} Z"/>'


def retangulo(x0, y0, x1, y1, cor):
    return recorte(x0, y0, x1, y1).format(cor)


def relva(y_topo, fundo, w=100.0, n=4):
    """A faixa de relva com a borda de baixo em arcos (período w/n — repete sem costura)."""
    passo = w / n
    cam = Caminho().m((0, y_topo)).l((w, y_topo)).l((w, fundo))
    for i in range(n, 0, -1):
        xa, xb = i * passo, (i - 1) * passo
        meio = (xa + xb) / 2
        cam.c((xa, fundo + passo * 0.3), (meio + passo * 0.25, fundo + passo * 0.42), (meio, fundo + passo * 0.42))
        cam.c((meio - passo * 0.25, fundo + passo * 0.42), (xb, fundo + passo * 0.3), (xb, fundo))
    return cam.z()


PEDRAS = [(14, 46, 7, 4), (52, 36, 5, 3.5), (78, 58, 8, 4.5), (34, 74, 6, 4), (66, 86, 5, 3), (12, 90, 4, 3)]


def azulejo(h, com_relva=True, base_escura=False):
    corpo = [retangulo(0, 0, 100, h, P["terra"])]
    for (x, y, rx, ry) in PEDRAS:
        if y * h / 100 + ry < h - (10 if base_escura else 0):
            corpo.append(elipse(x, y * h / 100, rx, ry).svg(ident, P["terra_escura"]))
    if base_escura:
        corpo.append(retangulo(0, h - 8, 100, h, P["terra_escura"]))
    if com_relva:
        corpo.append(relva(0, 13).svg(ident, P["relva_escura"]))
        corpo.append(relva(0, 9).svg(ident, P["relva"]))
    return corpo


def moeda():
    corpo, r, esp = [], 38.0, 9.0
    for i in range(6):
        th = math.radians(i * 30)
        c = math.cos(th)
        cx, cy = i * 100 + 50, 50
        rx = r * abs(c)
        lado = -1 if c >= 0 else 1
        t = esp * math.sin(th) * lado
        if abs(t) > 0.01:
            corpo.append(elipse(cx + t, cy, rx, r).svg(ident, P["ouro_escuro"]))
            corpo.append(retangulo(min(cx, cx + t), cy - r, max(cx, cx + t), cy + r, P["ouro_escuro"]))
        if rx > 0.5:
            corpo.append(elipse(cx, cy, rx, r).svg(ident, P["ouro"]))
            corpo.append(elipse(cx, cy, rx * 0.62, r * 0.62).svg(ident, P["ouro_escuro"]))
            corpo.append(elipse(cx, cy, rx * 0.47, r * 0.47).svg(ident, P["ouro"]))
            corpo.append(elipse(cx - rx * 0.42, cy - r * 0.42, rx * 0.16, r * 0.1, -0.8 * (1 if c >= 0 else -1)).svg(ident, P["ouro_claro"]))
    return svg(600, 100, corpo, "moeda 6×1 (a girar)")


def bandeira():
    corpo = []
    for i in range(4):
        ox = i * 100
        fase = i * math.pi / 2
        x0, x1, n = 26.0, 90.0, 16

        def onda(x, y):
            a = 5.0 * (x - x0) / (x1 - x0)
            return (ox + x, y + a * math.sin((x - x0) / 64 * 2 * math.pi - fase))

        def faixa(ya, yb, cor):
            xs = [x0 + (x1 - x0) * k / n for k in range(n + 1)]
            cam = Caminho().m(onda(xs[0], ya))
            for x in xs[1:]:
                cam.l(onda(x, ya))
            for x in reversed(xs):
                cam.l(onda(x, yb))
            return cam.z().svg(ident, cor)

        corpo.append(retangulo(ox + 18, 12, ox + 26, 194, P["mastro"]))
        corpo.append(retangulo(ox + 18, 12, ox + 21, 194, P["mastro_claro"]))
        corpo.append(rrect(ox + 22, 194, 24, 12, 4).svg(ident, P["mastro"]))
        corpo.append(elipse(ox + 22, 11, 7, 7).svg(ident, P["mastro_claro"]))
        corpo.append(faixa(18, 70, P["bandeira"]))
        corpo.append(faixa(54, 70, P["bandeira_escura"]))
    return svg(400, 200, corpo, "bandeira 4×1 (a ondular)")


def main():
    poses = json.load(open(sys.argv[1]))
    out = sys.argv[2]
    os.makedirs(out, exist_ok=True)
    arte = {
        "boneco.svg": boneco(poses),
        "chao.svg": svg(100, 100, azulejo(100), "chão 1×1 m (repete na horizontal)"),
        "plataforma.svg": svg(100, 50, azulejo(50, base_escura=True), "plataforma 1×0,5 m (repete na horizontal)"),
        "terra.svg": svg(100, 100, azulejo(100, com_relva=False), "terra 1×1 m (repete nos dois eixos)"),
        "moeda.svg": moeda(),
        "bandeira.svg": bandeira(),
    }
    for nome, txt in arte.items():
        with open(os.path.join(out, nome), "w") as fh:
            fh.write(txt)
    print("[arte_svg] ok →", out, sorted(arte))


main()
