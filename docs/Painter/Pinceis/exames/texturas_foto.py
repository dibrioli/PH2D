#!/usr/bin/env python3
"""A identidade de um carimbo/textura na cena `tamanhos` da bateria (J4 `texturas`) — as MESMAS contas do
`Identidade::de` de `crates/ph2d-tool-painter/tests/it/familias/texturas.rs` (o portão por foto).

Na faixa do recto de 40 px (y 120–260) e na do de 200 px (y 300–580), x 120–880: a cobertura c = máx|px − 255| ÷
232 (o azul cheio = 1); as MANCHAS = componentes 4-ligados de c ≥ 0,3 com área ≥ 3 px.
  cob       — a cobertura média da faixa
  pontos    — manchas por 100 px de comprimento
  gota      — a área mediana das manchas (px)
  alto      — a mediana de altura ÷ largura da caixa das manchas com área ≥ 10 (a relva: lâminas de pé)
  contraste — p90 ÷ p50 da cobertura dos píxeis com c ≥ 0,1 (o bokeh: o aro sobre o miolo)
  cv_area   — o desvio-padrão ÷ a média das áreas (a retícula: pontos iguais)

uso: texturas_foto.py <tamanhos_A.png|.ppm> …
"""
import sys
from PIL import Image

FAIXAS = {"40": (120, 260), "200": (300, 580)}
X0, X1 = 120, 880


def identidade(caminho):
    im = Image.open(caminho).convert("RGB")
    w = im.width
    px = im.load()
    out = {}
    for nome, (y0, y1) in FAIXAS.items():
        W, H = X1 - X0, y1 - y0
        c = [[max(abs(255 - v) for v in px[X0 + x, y0 + y]) / 232.0 for x in range(W)] for y in range(H)]
        tot = sum(sum(r) for r in c)
        cob = tot / (W * H)
        vis = [[False] * W for _ in range(H)]
        areas, altos = [], []
        for y in range(H):
            for x in range(W):
                if vis[y][x] or c[y][x] < 0.3:
                    continue
                pilha = [(x, y)]
                vis[y][x] = True
                n, xa, xb, ya, yb = 0, x, x, y, y
                while pilha:
                    a, b = pilha.pop()
                    n += 1
                    xa, xb, ya, yb = min(xa, a), max(xb, a), min(ya, b), max(yb, b)
                    for da, db in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                        p, q = a + da, b + db
                        if 0 <= p < W and 0 <= q < H and not vis[q][p] and c[q][p] >= 0.3:
                            vis[q][p] = True
                            pilha.append((p, q))
                if n >= 3:
                    areas.append(n)
                    if n >= 10:
                        altos.append((yb - ya + 1) / (xb - xa + 1))
        tinta = sorted(v for r in c for v in r if v >= 0.1)
        med = lambda v: sorted(v)[len(v) // 2] if v else 0.0
        q = lambda f: tinta[min(len(tinta) - 1, int(f * len(tinta)))] if tinta else 0.0
        m = sum(areas) / len(areas) if areas else 0.0
        sd = (sum((a - m) ** 2 for a in areas) / len(areas)) ** 0.5 if areas else 0.0
        out[nome] = dict(
            cob=round(cob, 3),
            pontos=round(len(areas) / (W / 100.0), 2),
            gota=med(areas),
            alto=round(med(altos), 2),
            contraste=round(q(0.9) / max(q(0.5), 1e-6), 2),
            cv_area=round(sd / m, 2) if m else 0.0,
        )
    return out


if __name__ == "__main__":
    for f in sys.argv[1:]:
        print(f.split("/")[-2] if "/" in f else f, f.split("/")[-1])
        for k, v in identidade(f).items():
            print("  ", k, v)
