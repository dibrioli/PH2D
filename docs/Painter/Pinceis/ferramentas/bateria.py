#!/usr/bin/env python3
"""A BATERIA do EXAME do pincel (00_MISSAO §3.1) — a fonte ÚNICA dos gestos, igual em todos os lados.

Cada consumidor (o Rebelle por `ws_eventos`, Krita, MyPaint, a bancada dos nossos 4 motores) lê o MESMO
`bateria.json` e reproduz os pontos tal como estão (já reamostrados ao `passo` = px entre eventos; o passo
grande é o traço RÁPIDO). Tela 1000×640 px, papel branco, y para baixo. Três cenas, uma foto cada:

  gestos   — tamanho 40: recto lento · recto rápido · curva em S · espiral · rampa de pressão 0→1 ·
             rampa 0,15→1→0,15 (afunila nas duas pontas) · vertical
  tamanhos — o recto lento a 5, 40 e 200 px
  sobre    — duas faixas VERMELHAS (60 px): a de baixo SECA, a de cima MOLHADA; o pincel (AZUL) cruza as
             duas em três verticais e corre um recto horizontal sobre o papel entre elas

Ops: {"op":"cor","rgb":[r,g,b]} · {"op":"pincel","quem":"preset"|"base"} (a base é o pincel das faixas:
o mesmo preset por omissão) · {"op":"traco","nome","size_px","pts":[[x,y,pressao],…]} ·
{"op":"seca"} (secar tudo o que está na tela) · {"op":"simula","quadros":N} (deixar a tinta correr).
No fim de cada cena: simula 240 → foto A (molhada) → seca → foto B.

uso: python3 -I bateria.py > bateria.json
"""
import json
import math

W, H = 1000, 640
AZUL = [23, 101, 179]
VERMELHO = [191, 0, 18]


def amostra(pts, passo, p):
    """Polilinha → eventos a `passo` px; p(t) com t∈[0,1] ao longo do comprimento (a lei de gera_eventos.gesto)."""
    total = sum(math.dist(a, b) for a, b in zip(pts, pts[1:]))
    out = [[round(pts[0][0], 2), round(pts[0][1], 2), round(p(0.0), 3)]]
    andado = 0.0
    for a, b in zip(pts, pts[1:]):
        ln = math.dist(a, b)
        n = max(1, math.ceil(ln / passo))
        for k in range(1, n + 1):
            f = k / n
            andado += ln / n
            out.append([round(a[0] + (b[0] - a[0]) * f, 2), round(a[1] + (b[1] - a[1]) * f, 2),
                        round(p(andado / total), 3)])
    return out


def traco(nome, size, pts, passo=3.0, p=lambda t: 1.0):
    return {"op": "traco", "nome": nome, "size_px": size, "passo": passo, "pts": amostra(pts, passo, p)}


def curva_s(x0, x1, cy, amp):
    return [(x0 + (x1 - x0) * i / 120, cy + amp * math.sin(i / 120 * math.tau * 1.5)) for i in range(121)]


def espiral(cx, cy, r0, r1, voltas):
    n = 360
    return [(cx + (r0 + (r1 - r0) * i / n) * math.cos(i / n * math.tau * voltas),
             cy + (r0 + (r1 - r0) * i / n) * math.sin(i / n * math.tau * voltas)) for i in range(n + 1)]


def fim_de_cena():
    return [{"op": "simula", "quadros": 240}, {"op": "foto", "nome": "A"}, {"op": "seca"}, {"op": "foto", "nome": "B"}]


def bateria():
    gestos = [{"op": "pincel", "quem": "preset"}, {"op": "cor", "rgb": AZUL},
              traco("recto_lento", 40, [(60, 90), (560, 90)]),
              traco("recto_rapido", 40, [(620, 90), (960, 90)], passo=25.0),
              traco("curva_s", 40, curva_s(60, 560, 240, 45)),
              traco("espiral", 40, espiral(800, 280, 12, 120, 3)),
              traco("rampa_0_1", 40, [(60, 400), (560, 400)], p=lambda t: t),
              traco("rampa_afunila", 40, [(60, 540), (560, 540)], p=lambda t: 0.15 + 0.85 * math.sin(t * math.pi)),
              traco("vertical", 40, [(700, 440), (700, 610)]),
              traco("vertical_rapido", 40, [(900, 440), (900, 610)], passo=25.0)] + fim_de_cena()
    tamanhos = [{"op": "pincel", "quem": "preset"}, {"op": "cor", "rgb": AZUL},
                traco("recto_5", 5, [(60, 70), (940, 70)]),
                traco("recto_40", 40, [(60, 190), (940, 190)]),
                traco("recto_200", 200, [(60, 440), (940, 440)])] + fim_de_cena()
    sobre = [{"op": "pincel", "quem": "base"}, {"op": "cor", "rgb": VERMELHO},
             traco("faixa_seca", 60, [(60, 470), (940, 470)]),
             {"op": "simula", "quadros": 240}, {"op": "seca"},
             traco("faixa_molhada", 60, [(60, 150), (940, 150)]),
             {"op": "pincel", "quem": "preset"}, {"op": "cor", "rgb": AZUL},
             traco("cruza_1", 40, [(250, 60), (250, 580)]),
             traco("cruza_2", 40, [(500, 60), (500, 580)]),
             traco("cruza_3_rapido", 40, [(750, 60), (750, 580)], passo=25.0),
             traco("papel", 40, [(60, 310), (940, 310)])] + fim_de_cena()
    return {"versao": 1, "tela": [W, H], "papel": [255, 255, 255],
            "cenas": [{"nome": "gestos", "ops": gestos}, {"nome": "tamanhos", "ops": tamanhos},
                      {"nome": "sobre", "ops": sobre}]}


if __name__ == "__main__":
    print(json.dumps(bateria(), separators=(",", ":")))
