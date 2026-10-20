#!/usr/bin/env python3
"""W23 bloco 6 — gera as entradas NOSSAS para o oráculo Excalidraw (canal next), corridas por
`corre_bloco6.mjs`:

  entradas/bloco6_laco.json   casos do LAÇO: elementos + o rasto do laço (cena = ecrã), ~4 px entre pontos
  entradas/bloco6_traco.json  os traços do QuickShape (`oraculo_quickshape/entradas/<nome>.txt`, px), só
                              transladados para o miolo do ecrã 1280×800 (escala 1: todos cabem)

Determinístico (sem acaso). Uso: python3 bloco6_gera_entradas.py [pasta_saida]
"""
import glob
import hashlib
import json
import math
import os
import sys

AQUI = os.path.dirname(os.path.abspath(__file__))
OUT = sys.argv[1] if len(sys.argv) > 1 else AQUI
REPO = os.path.abspath(os.path.join(AQUI, "../../../../.."))
QS = os.path.join(REPO, "docs/UI_New_and_Simple/vetor_nova_ui/oraculo_quickshape")
STEP = 4.0


def base(i, t, x, y, w, h, **kw):
    e = {
        "id": i, "type": t, "x": x, "y": y, "width": w, "height": h, "angle": 0,
        "strokeColor": "#1e1e1e", "backgroundColor": "transparent", "fillStyle": "solid",
        "strokeWidth": 2, "strokeStyle": "solid", "roughness": 0, "opacity": 100,
        "groupIds": [], "frameId": None, "roundness": None,
        "seed": sum(map(ord, i)) * 7919 % 2**31,
        "version": 1, "versionNonce": 1, "isDeleted": False, "boundElements": None,
        "updated": 1, "link": None, "locked": False,
    }
    e.update(kw)
    return e


def linear(i, t, x, y, pts, **kw):
    xs = [p[0] for p in pts]
    ys = [p[1] for p in pts]
    e = base(i, t, x, y, max(xs) - min(xs), max(ys) - min(ys), points=pts, **kw)
    if t in ("line", "arrow"):
        e.update(startBinding=None, endBinding=None, startArrowhead=None,
                 endArrowhead="arrow" if t == "arrow" else None, lastCommittedPoint=None)
    if t == "freedraw":
        e.update(pressures=[], simulatePressure=True, lastCommittedPoint=None)
    return e


def resample(poly, step=STEP):
    """Polilinha amostrada a `step` px (como um rato a ~60 Hz); termina no último vértice."""
    out = [list(poly[0])]
    carry = 0.0
    for (x0, y0), (x1, y1) in zip(poly, poly[1:]):
        d = math.hypot(x1 - x0, y1 - y0)
        if d == 0:
            continue
        t = carry if carry > 0 else step
        while t < d:
            out.append([round(x0 + (x1 - x0) * t / d, 2), round(y0 + (y1 - y0) * t / d, 2)])
            t += step
        carry = t - d
        out.append([x1, y1])
    return out


def loop(poly):
    """Laço fechado: volta ao 1.º vértice."""
    return resample(list(poly) + [poly[0]])


def elipse_pts(cx, cy, a, b, n=None):
    per = math.pi * (3 * (a + b) - math.sqrt((3 * a + b) * (a + 3 * b)))
    n = n or int(per / STEP)
    return [[round(cx + a * math.cos(2 * math.pi * k / n), 2), round(cy + b * math.sin(2 * math.pi * k / n), 2)]
            for k in range(n + 1)]


def quadrado(x0, y0, x1, y1, inicio="br"):
    """Quadrado no sentido horário, a começar num canto (o 1.º ponto é onde o rato carrega)."""
    cantos = {"tl": (x0, y0), "tr": (x1, y0), "br": (x1, y1), "bl": (x0, y1)}
    ordem = ["tl", "tr", "br", "bl"]
    k = ordem.index(inicio)
    return [cantos[c] for c in ordem[k:] + ordem[:k]]


casos = []


def caso(nome, descricao, elementos, laco, esperado=None):
    casos.append({"nome": nome, "descricao": descricao, "elementos": elementos, "laco": laco,
                  "fracao_dentro": esperado or {}})


# 1 — a fracção da área dentro do laço (95 · 89 · 60), laço = o quadrado [0,0]-[1000,1000].
# O rato carrega no canto (1000,1000), fora de qualquer botão.
caso("1_fracao", "4 rectângulos 100×100: 100 % · 95 % · 89 % · 60 % dentro do laço quadrado [0,0]-[1000,1000]",
     [base("r100", "rectangle", 100, 100, 100, 100), base("r95", "rectangle", 905, 300, 100, 100),
      base("r89", "rectangle", 911, 500, 100, 100), base("r60", "rectangle", 940, 700, 100, 100)],
     loop(quadrado(0, 0, 1000, 1000)), {"r100": 1.0, "r95": 0.95, "r89": 0.89, "r60": 0.60})

# 1b — o limiar mais fino (100 × 60, linhas de 120 px), o mesmo laço.
fr = [0.99, 0.97, 0.93, 0.91, 0.90, 0.85, 0.75, 0.50]
caso("1b_limiar", "8 rectângulos 100×60 com 99 · 97 · 93 · 91 · 90 · 85 · 75 · 50 % da largura dentro do laço [0,0]-[1000,1000]",
     [base(f"f{int(f * 100)}", "rectangle", 1000 - 100 * f, 40 + 120 * k, 100, 60) for k, f in enumerate(fr)],
     loop(quadrado(0, 0, 1000, 1000)), {f"f{int(f * 100)}": f for f in fr})

# 2 — elipse 300×150 no centro (500,400); laço = elipse 324×174 (12 px fora do contorno), os cantos da
# caixa (350,325)… ficam FORA do laço ((150/162)²+(75/87)² = 1,60 > 1).
caso("2_elipse", "elipse 300×150 centrada em (500,400); laço elíptico 12 px fora do contorno, cantos da caixa fora do laço",
     [base("el", "ellipse", 350, 325, 300, 150)],
     elipse_pts(500, 400, 162, 87))

# 3 — losango 200×200 em (400,300) (vértices (500,300) (600,400) (500,500) (400,400)); laço = losango de
# semi-diagonal 115 (cantos da caixa: |dx|+|dy| = 200 > 115 ⇒ fora).
caso("3_losango", "losango 200×200 em (400,300); laço = losango maior (semi-diagonal 115 contra 100), cantos da caixa fora",
     [base("lo", "diamond", 400, 300, 200, 200)],
     loop([(500, 285), (615, 400), (500, 515), (385, 400)]))

# 4 — rectângulo 100×100 rodado 45° em (500,500) (centro (550,550), contorno = losango de semi-diagonal
# 70,7); laço = losango de semi-diagonal 80 (os cantos da caixa local e da caixa do mundo ficam fora).
caso("4_rodado", "rectângulo 100×100 rodado 45° em (500,500); laço a abraçar o contorno rodado (semi-diagonal 80 contra 70,7)",
     [base("rr", "rectangle", 500, 500, 100, 100, angle=math.pi / 4)],
     loop([(550, 470), (630, 550), (550, 630), (470, 550)]))

# 5 — laço em oito (laço-gravata): dois triângulos que se cruzam em (400,400); um 80×80 em cada lóbulo.
caso("5_oito", "laço em gravata (cruza-se em (400,400)) com um rectângulo 80×80 em cada lóbulo",
     [base("lobE", "rectangle", 220, 360, 80, 80), base("lobD", "rectangle", 500, 360, 80, 80)],
     loop([(200, 200), (600, 600), (600, 200), (200, 600)]))

# 6 — laço em «C» côncavo: [50,100]-[1050,1000] com o entalhe [500,450]-[1050,650] cortado pela direita;
# rectângulo [200,200]-[800,800]: os 4 cantos DENTRO do laço e da caixa do laço, o entalhe come 1/6 da área.
C = [(50, 100), (1050, 100), (1050, 450), (500, 450), (500, 650), (1050, 650), (1050, 1000), (50, 1000)]
caso("6_C_fundo", "laço em C [50,100]-[1050,1000] com entalhe [500,450]-[1050,650]; rectângulo [200,200]-[800,800]: cantos dentro, 83,3 % da área",
     [base("rc", "rectangle", 200, 200, 600, 600)], loop(C), {"rc": 1 - 300 * 200 / 360000})
C2 = [(50, 100), (1050, 100), (1050, 450), (760, 450), (760, 650), (1050, 650), (1050, 1000), (50, 1000)]
caso("6b_C_raso", "o mesmo C com o entalhe raso [760,450]-[1050,650]: o contorno do rectângulo atravessa-o, 97,8 % da área dentro",
     [base("rc", "rectangle", 200, 200, 600, 600)], loop(C2), {"rc": 1 - 40 * 200 / 360000})

# 7 — cadeado: um trancado e um livre, os dois inteiros dentro do laço.
caso("7_trancado", "rectângulo trancado e rectângulo livre, os dois inteiros dentro do laço [100,100]-[700,600]",
     [base("tranc", "rectangle", 200, 200, 100, 100, locked=True), base("livre", "rectangle", 400, 200, 100, 100)],
     loop(quadrado(100, 100, 700, 600)))

# 8 — linha de 2 pontos inteira dentro; seta que atravessa a fronteira (metade dentro).
caso("8_linha_seta", "linha de 2 pontos inteira dentro do laço [100,100]-[700,700]; seta horizontal 500→900 (metade dentro)",
     [linear("lin", "line", 200, 300, [[0, 0], [200, 100]]), linear("seta", "arrow", 500, 400, [[0, 0], [400, 0]])],
     loop(quadrado(100, 100, 700, 700)), {"lin": 1.0, "seta": 0.5})

# 9 — traço livre inteiro dentro; outro com metade dentro.
onda = lambda n, amp: [[float(4 * k), round(amp * math.sin(k / 6.0), 2)] for k in range(n)]
caso("9_livre", "traço livre (onda 300 px) inteiro dentro do laço [100,100]-[700,700]; outro de 400 px de 500 a 900 (metade dentro)",
     [linear("fd_in", "freedraw", 200, 300, onda(76, 20)), linear("fd_meio", "freedraw", 500, 500, onda(101, 20))],
     loop(quadrado(100, 100, 700, 700)), {"fd_in": 1.0, "fd_meio": 0.5})
# (a caixa do freedraw: o restore recalcula x/y/width/height a partir dos pontos; os pontos acima têm y negativos)

# 10 — grupo: dois rectângulos agrupados, só um inteiro dentro do laço.
caso("10_grupo", "dois rectângulos do grupo g1: um inteiro dentro do laço [100,100]-[600,600], o outro fora",
     [base("gA", "rectangle", 200, 200, 100, 100, groupIds=["g1"]), base("gB", "rectangle", 700, 200, 100, 100, groupIds=["g1"])],
     loop(quadrado(100, 100, 600, 600)))

laco = {
    "descricao": "W23 bloco 6 — o laço do Excalidraw next: que elementos o laço selecciona (cena = ecrã, zoom 1)",
    "tela": [1400, 1100],
    "modos": ["contain", "overlap"],
    "casos": casos,
}

# ---- B: os traços do QuickShape (px da tela virtual 1600×1000) transladados para o ecrã 1280×800 ----
CENTRO = (760, 410)  # o painel de propriedades ocupa x < ~230 à esquerda; a barra y < 60
SEGURA = (260, 80, 1260, 740)
tracos = []
for f in sorted(glob.glob(os.path.join(QS, "entradas", "*.txt"))):
    nome = os.path.basename(f)[:-4]
    raw = open(f, "rb").read()
    pts = []
    varia = ""
    for l in raw.decode("utf-8").splitlines():
        if l.startswith("# varia:"):
            varia = l.split(":", 1)[1].strip()
        elif l.strip() and not l.startswith("#"):
            x, y = map(float, l.split()[:2])
            pts.append((x, y))
    xs = [p[0] for p in pts]
    ys = [p[1] for p in pts]
    cx, cy = (min(xs) + max(xs)) / 2, (min(ys) + max(ys)) / 2
    dx, dy = round(CENTRO[0] - cx), round(CENTRO[1] - cy)
    tela = [[round(x + dx, 3), round(y + dy, 3)] for x, y in pts]
    tx = [p[0] for p in tela]
    ty = [p[1] for p in tela]
    assert SEGURA[0] <= min(tx) and max(tx) <= SEGURA[2] and SEGURA[1] <= min(ty) and max(ty) <= SEGURA[3], nome
    tracos.append({"nome": nome, "varia": varia,
                   "fonte": f"docs/UI_New_and_Simple/vetor_nova_ui/oraculo_quickshape/entradas/{nome}.txt",
                   "fonte_sha256": hashlib.sha256(raw).hexdigest(),
                   "transformacao": {"escala": 1, "dx": dx, "dy": dy, "nota": "ecrã = fonte·escala + (dx,dy)"},
                   "pontos": tela})
assert len(tracos) == 80, len(tracos)

traco = {
    "descricao": "W23 bloco 6 — os 80 traços do QuickShape tocados pelo RATO no Excalidraw next: «Draw to shape» e, de controlo, «Draw»",
    "tela": [1280, 800],
    "ferramentas": ["autoshape", "freedraw"],
    "tracos": tracos,
}

for nome, obj in (("bloco6_laco.json", laco), ("bloco6_traco.json", traco)):
    p = os.path.join(OUT, nome)
    with open(p, "w", encoding="utf-8") as fh:
        json.dump(obj, fh, ensure_ascii=False, separators=(",", ":"))
        fh.write("\n")
    print(p, os.path.getsize(p), "bytes")
