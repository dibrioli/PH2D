#!/usr/bin/env python3
"""W23 (o vão do losango e do rectângulo redondo): gera entradas/w23_vao_casos.json para corre_fidelidade.mjs.

  V_*  a lei do VÃO do legado {focus, gap} no 0.18.1 nas formas que a W10 deixou «sem lei»: UMA forma por
       caso, setas soltas de 36 ângulos × 13 focos presas pelo FIM (e outras tantas pelo INÍCIO); o editor
       re-liga-as ao EMPURRAR a forma (as peças do gera_w10_casos.py). Forma × cantos × tamanho × vão ×
       rotação, mais `gap 0` e o «ponto fixo dentro» como o NOSSO escritor o grava (legacy_focus.rs:
       focus da recta vizinho → ponta, gap = −distância ao NOSSO contorno).
  Tamanhos dos dois lados do limiar do raio adaptativo (type 3: min(S/4, 32), limiar S = 128):
  200×120 e 120×200 (r 30), 60×40 (r 10), 300×200 (r 32, o tecto).
Uso: python3 gera_w23_vao_casos.py <pasta_entradas>
"""
import json
import math
import sys
from pathlib import Path


def base(i, t, x, y, w, h, **kw):
    e = {
        "id": i, "type": t, "x": x, "y": y, "width": w, "height": h, "angle": 0,
        "strokeColor": "#1e1e1e", "backgroundColor": "transparent", "fillStyle": "solid",
        "strokeWidth": 2, "strokeStyle": "solid", "roughness": 0, "opacity": 100,
        "groupIds": [], "frameId": None, "roundness": None, "seed": sum(map(ord, i)) * 7919 % 2**31,
        "version": 1, "versionNonce": 1, "isDeleted": False, "boundElements": None,
        "updated": 1, "link": None, "locked": False,
    }
    e.update(kw)
    return e


def arrow(i, p0, p1, sb=None, eb=None):
    """A seta SÓ com o que a geometria lê — o passo `restore` do editor completa o resto (a entrada densa
    pesava 101 MB com o elemento inteiro)."""
    p0, p1 = [round(v, 4) for v in p0], [round(v, 4) for v in p1]
    rel = [[0.0, 0.0], [round(p1[0] - p0[0], 4), round(p1[1] - p0[1], 4)]]
    a = {"id": i, "type": "arrow", "x": p0[0], "y": p0[1], "width": abs(rel[1][0]), "height": abs(rel[1][1]),
         "points": rel}
    a.update({"startBinding": sb} if sb else {"endBinding": eb})
    return a


CX, CY = 640.0, 400.0
FOCOS = [round(-0.9 + 0.18 * k, 2) for k in range(11)]  # −0,9 … 0,9
ANGULOS = [3.75 + 15 * k for k in range(24)]  # fora dos eixos (os empates da W10)
CAMPOS = ["id", "type", "x", "y", "width", "height", "angle", "roundness", "points", "startBinding", "endBinding"]
PASSOS = [{"tipo": "restore", "guardar": False}, {"tipo": "montar", "guardar": False},
          {"tipo": "empurrar", "ids": ["F"]}]
FORMAS = [  # (sufixo, tipo, roundness)
    ("losango", "diamond", None),
    ("losango_r2", "diamond", {"type": 2}),  # o que o NOSSO escritor grava num losango redondo
    ("losango_r3", "diamond", {"type": 3}),
    ("retangulo_r3", "rectangle", {"type": 3}),
]
TAMANHOS = [(120, 200), (60, 40), (300, 200)]  # além do 200×120, que leva a varredura inteira dos vãos
VAOS = [0.01, 1, 3, 6, 10, 20, -5]
VAOS_CURTOS = [1, 6, 20, -5]


def nome_g(g):
    return ("neg" + str(-g) if g < 0 else str(g)).replace(".", "p")


def forma(tipo, w, h, angle, rnd):
    return base("F", tipo, CX - w / 2, CY - h / 2, w, h, angle=angle, roundness=rnd)


def caso_vao(nome, desc, tipo, rnd, gap, w, h, angle=0.0, inicio=False):
    f = forma(tipo, w, h, angle, rnd)
    r = max(w, h) / 2 + 140
    ars = []
    for th in ANGULOS:
        s = [CX + r * math.cos(math.radians(th)), CY + r * math.sin(math.radians(th))]
        for k, fo in enumerate(FOCOS):
            b = {"elementId": "F", "focus": fo, "gap": gap}
            ars.append(arrow(f"e{th}_{k}", s, [CX, CY], eb=b))
            if inicio:  # presa pelo INÍCIO (a W10 mostrou-a igual; aqui só no 200×120 g 6, o controlo)
                ars.append(arrow(f"s{th}_{k}", [CX, CY], s, sb=b))
    f["boundElements"] = [{"id": a["id"], "type": "arrow"} for a in ars]
    return {"nome": nome, "descricao": desc, "elementos": [f] + ars, "passos": PASSOS, "compacto": True,
            "campos": CAMPOS}


# ---- o «ponto fixo dentro» como o NOSSO escritor o grava (legacy_focus.rs, formas sem lei medida) ----
UNTIE = 1.0 + 1e-6


def contorno_nosso(tipo, rnd, w, h, n=4000):
    """O NOSSO contorno (ph2d-board-geom::outline), local, centro na origem: polilinha fina."""
    hw, hh = w / 2, h / 2
    if tipo == "rectangle":
        r = min(min(w, h) * 0.25, 32.0)
        cantos = [((hw, -hh), (hw, -hh + r), (hw - r, -hh)), ((hw, hh), (hw - r, hh), (hw, hh - r)),
                  ((-hw, hh), (-hw, hh - r), (-hw + r, hh)), ((-hw, -hh), (-hw + r, -hh), (-hw, -hh + r))]
        pts = []
        for v, a, b in cantos:  # quadrática a → (v) → b
            for i in range(n // 4 + 1):
                t = i / (n // 4)
                pts.append(((1 - t) ** 2 * a[0] + 2 * t * (1 - t) * v[0] + t * t * b[0],
                            (1 - t) ** 2 * a[1] + 2 * t * (1 - t) * v[1] + t * t * b[1]))
        return pts + pts[:1]
    vs = [(0, -hh), (hw, 0), (0, hh), (-hw, 0)]
    if rnd is None:
        return vs + vs[:1]
    pts = []
    for i in range(4):  # ¼ de cada aresta a partir do vértice, quadrática com o controlo no vértice
        v, p, q = vs[i], vs[i - 1], vs[(i + 1) % 4]
        a = (v[0] + 0.25 * (p[0] - v[0]), v[1] + 0.25 * (p[1] - v[1]))
        b = (v[0] + 0.25 * (q[0] - v[0]), v[1] + 0.25 * (q[1] - v[1]))
        for j in range(n // 4 + 1):
            t = j / (n // 4)
            pts.append(((1 - t) ** 2 * a[0] + 2 * t * (1 - t) * v[0] + t * t * b[0],
                        (1 - t) ** 2 * a[1] + 2 * t * (1 - t) * v[1] + t * t * b[1]))
    return pts + pts[:1]


def dist_poly(p, poly):
    best = math.inf
    for (ax, ay), (bx, by) in zip(poly, poly[1:]):
        dx, dy = bx - ax, by - ay
        L = dx * dx + dy * dy
        t = 0 if L == 0 else max(0, min(1, ((p[0] - ax) * dx + (p[1] - ay) * dy) / L))
        best = min(best, math.hypot(p[0] - ax - t * dx, p[1] - ay - t * dy))
    return best


def focus_nosso(tipo, hw, hh, a, p):
    u = (p[0] - a[0], p[1] - a[1])
    L = math.hypot(*u)
    nx, ny = abs(u[1] / L), abs(u[0] / L)
    sup = max(hw * nx, hh * ny) if tipo == "diamond" else hw * nx + hh * ny
    sd = (u[0] * -a[1] - u[1] * -a[0]) / L
    return sd / sup * UNTIE


def caso_dentro(nome, desc, tipo, rnd, w, h):
    """Pontas fixas DENTRO da forma (fracções da meia-caixa), vindas de 12 direcções."""
    f = forma(tipo, w, h, 0.0, rnd)
    hw, hh = w / 2, h / 2
    poly = contorno_nosso(tipo, rnd, w, h)
    r = max(w, h) / 2 + 140
    ars = []
    pontos = [(0.0, 0.0), (0.3, 0.0), (0.0, 0.3), (0.2, 0.2), (-0.25, 0.1), (0.4, -0.15), (0.1, -0.5),
              (-0.35, -0.25)]
    if tipo == "rectangle":
        pontos += [(0.85, 0.75), (-0.8, 0.8), (0.9, -0.6)]  # junto dos cantos redondos (dentro)
    for i, (fx, fy) in enumerate(pontos):
        p = (fx * hw, fy * hh)
        for th in range(15, 360, 30):
            a = (r * math.cos(math.radians(th)), r * math.sin(math.radians(th)))
            fo = focus_nosso(tipo, hw, hh, a, p)
            g = -dist_poly(p, poly)
            g = math.copysign(max(abs(g), 0.01), g) if g != 0 else 0.01
            b = {"elementId": "F", "focus": fo, "gap": g}
            ars.append(arrow(f"e{i}_{th}", [CX + a[0], CY + a[1]], [CX + p[0], CY + p[1]], eb=b))
    f["boundElements"] = [{"id": a["id"], "type": "arrow"} for a in ars]
    return {"nome": nome, "descricao": desc, "elementos": [f] + ars, "passos": PASSOS, "compacto": True,
            "campos": CAMPOS}


casos = []
for suf, tipo, rnd in FORMAS:
    for g in VAOS:
        casos.append(caso_vao(f"V_{suf}_200x120_g{nome_g(g)}", f"{tipo} {rnd} 200×120, gap {g}", tipo, rnd, g, 200, 120,
                              inicio=g == 6))
    for w, h in TAMANHOS:
        for g in VAOS_CURTOS:
            casos.append(caso_vao(f"V_{suf}_{w}x{h}_g{nome_g(g)}", f"{tipo} {rnd} {w}×{h}, gap {g}", tipo, rnd, g, w, h))
    for g in (6, -5):
        casos.append(caso_vao(f"V_{suf}_200x120_rot_g{nome_g(g)}", f"{tipo} {rnd} 200×120 rodado 0,4 rad, gap {g}",
                              tipo, rnd, g, 200, 120, angle=0.4))
    casos.append(caso_vao(f"V_{suf}_200x120_g0", f"{tipo} {rnd} 200×120, gap 0 (a ponta no foco)", tipo, rnd, 0, 200, 120))
    for w, h in ((200, 120), (60, 40)):
        casos.append(caso_dentro(f"V_{suf}_{w}x{h}_dentro", f"{tipo} {rnd} {w}×{h}: ponto fixo dentro, o NOSSO par",
                                 tipo, rnd, w, h))
    # B9: medidas FRACCIONÁRIAS (as nossas formas raramente são pares): o vértice do losango a ⌊w/2⌋+1 só se
    # distingue de w/2+1 (e o raio R(⌊w/2⌋+1) de R(w/2+1)) fora dos inteiros pares.
    for g in (6, -5):
        casos.append(caso_vao(f"V_{suf}_173p4x97p8_g{nome_g(g)}", f"{tipo} {rnd} 173,4×97,8, gap {g}", tipo, rnd, g,
                              173.4, 97.8))
    casos.append(caso_dentro(f"V_{suf}_173p4x97p8_dentro", f"{tipo} {rnd} 173,4×97,8: ponto fixo dentro, o NOSSO par",
                             tipo, rnd, 173.4, 97.8))
# Controlo: o rectângulo de cantos vivos (lei exacta da W10: caixa de Chebyshev + gap).
for g in (6, -5):
    casos.append(caso_vao(f"V_retangulo_200x120_g{nome_g(g)}", f"rectangle vivo 200×120, gap {g} (controlo)",
                          "rectangle", None, g, 200, 120))

out = Path(sys.argv[1]) / "w23_vao_casos.json"
out.write_text(json.dumps({"casos": casos}, ensure_ascii=False, separators=(",", ":")) + "\n")
print(out.name, len(casos), sum(len(c["elementos"]) - 1 for c in casos), "setas")
