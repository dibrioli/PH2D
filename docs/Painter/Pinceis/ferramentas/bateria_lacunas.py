#!/usr/bin/env python3
"""A bateria das LACUNAS (06_LEIS_DAS_LACUNAS.md §8): os gestos que a `bateria.json` não tem, num ficheiro À PARTE.

⛔ `bateria.json` é a fonte única dos oráculos e não muda (mudá-la obrigaria a refazer todos). Esta bateria usa o
MESMO formato e as MESMAS funções (`bateria.py`), tela 1000×640, papel branco, cada cena acaba em simula 240 →
foto A → seca → foto B, e só é pintada pelos presets que a lei pede (as listas `lista_lacunas_*.txt`).

Ops novas (os consumidores antigos não as vêem — a `bateria.json` não as tem):
  · traco `"mesmo_pincel": true` — o Rebelle NÃO recebe `SET_BRUSH` antes deste traço (a memória entre traços, L3)
  · `{"op":"cor_fundo","rgb":[…]}` — a 2.ª cor (fundo) do Krita (L11); o Rebelle recusa-a, o MyPaint ignora-a
  · traço de um ponto só (dois eventos no mesmo sítio) = um TOQUE: um carimbo isolado (L4, L19)

Cenas:
  memoria     L3 — h1 (novo) → v1 SEM SET_BRUSH → h2 SEM SET_BRUSH → v2 novo (controlo) → h3 novo (controlo), 40 px
  canto       L3 — canto de 90° (recto → vertical, e vertical → recto) e zigue-zague de pernas de 60 px a ±45°, 20 px
  efeito      L7 — faixas CINZA 50 % e PASTEL (S 0,3) da base, secas; o efeito azul 1× (x 250) e 10× (x 500)
  velocidade  L12 — rectos de 880 px a 3 · 6 · 12 · 25 · 50 px por evento (10 ms/evento: 300…5000 px/s), 40 px
  carimbos    L19 — toques a 1,5·D (60 px) em três linhas: pressão 1 · 0,6 · 0,3, 40 px
  carimbos_3d L4 — toques a 3·D (120 px), as mesmas três pressões
  fundo       L11 — frente AZUL, fundo AMARELO: recto lento, rampa 0→1, curva em S (só o Krita)
  gestos · tamanhos — cópia EXACTA das da `bateria.json` (para o preset com parâmetro mudado: L6 dual, L4 spacing)

uso: python3 -I bateria_lacunas.py > bateria_lacunas.json
"""
import json
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from bateria import AZUL, H, VERMELHO, W, bateria, curva_s, fim_de_cena, traco  # noqa: E402,F401

CINZA = [128, 128, 128]
PASTEL = [230, 184, 161]  # HSV: H 20°, S 0,30, V 0,90 — longe do matiz do azul (210°)
AMARELO = [230, 190, 0]


def mesmo(op):
    op["mesmo_pincel"] = True
    return op


def toque(nome, size, x, y, p):
    return {"op": "traco", "nome": nome, "size_px": size, "passo": 0.0, "pts": [[x, y, p], [x, y, p]]}


def zigue(x0, y0, perna, n):
    d = perna / math.sqrt(2.0)
    return [(x0 + i * d, y0 + (d if i % 2 else 0.0)) for i in range(n + 1)]


def toques(prefixo, size, passo_px, linhas):
    out = []
    for y, p in linhas:
        x = 80.0
        k = 0
        while x <= 920.0:
            out.append(toque(f"{prefixo}_p{int(round(p * 100)):03d}_{k:02d}", size, x, y, p))
            x += passo_px
            k += 1
    return out


def lacunas():
    velha = {c["nome"]: c for c in bateria()["cenas"]}
    p0 = [{"op": "pincel", "quem": "preset"}, {"op": "cor", "rgb": AZUL}]
    memoria = p0 + [traco("h1", 40, [(60, 100), (460, 100)]),
                    mesmo(traco("v1_sem_set", 40, [(560, 60), (560, 300)])),
                    mesmo(traco("h2_sem_set", 40, [(60, 220), (460, 220)])),
                    traco("v2_controlo", 40, [(700, 60), (700, 300)]),
                    traco("h3_controlo", 40, [(60, 340), (460, 340)])] + fim_de_cena()
    canto = p0 + [traco("canto_h_v", 20, [(100, 80), (400, 80), (400, 330)]),
                  traco("canto_v_h", 20, [(560, 380), (560, 600), (860, 600)]),
                  traco("zigue", 20, zigue(520, 100, 60.0, 8)),
                  traco("zigue_2", 20, zigue(100, 420, 60.0, 6))] + fim_de_cena()
    efeito = [{"op": "pincel", "quem": "base"}, {"op": "cor", "rgb": CINZA},
              traco("faixa_cinza", 60, [(60, 150), (940, 150)]),
              {"op": "cor", "rgb": PASTEL},
              traco("faixa_pastel", 60, [(60, 470), (940, 470)]),
              {"op": "simula", "quadros": 240}, {"op": "seca"},
              {"op": "pincel", "quem": "preset"}, {"op": "cor", "rgb": AZUL},
              traco("efeito_1x", 40, [(250, 60), (250, 580)])]
    efeito += [traco(f"efeito_10x_{i:02d}", 40, [(500, 60), (500, 580)]) for i in range(10)]
    efeito += fim_de_cena()
    velocidade = p0 + [traco(f"recto_{v:02d}px", 40, [(60, y), (940, y)], passo=float(v))
                       for v, y in ((3, 70), (6, 190), (12, 310), (25, 430), (50, 550))] + fim_de_cena()
    linhas = ((120, 1.0), (320, 0.6), (520, 0.3))
    carimbos = p0 + toques("toque", 40, 60.0, linhas) + fim_de_cena()
    carimbos_3d = p0 + toques("toque", 40, 120.0, linhas) + fim_de_cena()
    fundo = p0 + [{"op": "cor_fundo", "rgb": AMARELO},
                  traco("recto_lento", 40, [(60, 90), (560, 90)]),
                  traco("curva_s", 40, curva_s(60, 560, 240, 45)),
                  traco("rampa_0_1", 40, [(60, 400), (560, 400)], p=lambda t: t),
                  traco("rampa_afunila", 40, [(60, 540), (560, 540)],
                        p=lambda t: 0.15 + 0.85 * math.sin(t * math.pi))] + fim_de_cena()
    cenas = [("memoria", memoria), ("canto", canto), ("efeito", efeito), ("velocidade", velocidade),
             ("carimbos", carimbos), ("carimbos_3d", carimbos_3d), ("fundo", fundo)]
    return {"versao": 1, "tela": [W, H], "papel": [255, 255, 255], "de": "bateria_lacunas.py",
            "cores": {"cinza": CINZA, "pastel": PASTEL, "amarelo": AMARELO, "azul": AZUL},
            "cenas": [{"nome": n, "ops": o} for n, o in cenas] + [velha["gestos"], velha["tamanhos"]]}


if __name__ == "__main__":
    print(json.dumps(lacunas(), separators=(",", ":")))
