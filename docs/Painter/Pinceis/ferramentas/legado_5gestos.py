#!/usr/bin/env python3
"""A cena ANTIGA dos 5 gestos (`sonda_gestos::pinta_cena`, as fotos `c_ws_<x>_A/B.png` do Rebelle de 07–16/10) no
formato do `bateria.json` — ⚠️ NÃO é a bateria do EXAME: serve só para validar o encanamento da bancada contra as
fotos antigas enquanto as da bateria não existem. Uma cena, `legado`; o 1.º traço chama-se `recto_lento` (a
largura que calibra a escala). Simula 480 quadros (os 8 s da cena) antes da foto A; B depois do seca (o `DRY`).

uso: python3 -I legado_5gestos.py > legado_5gestos.json
"""
import json
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from bateria import AZUL, VERMELHO, W, H, curva_s, traco  # noqa: E402


def legado():
    ops = [{"op": "pincel", "quem": "preset"}, {"op": "cor", "rgb": AZUL},
           traco("recto_lento", 40, [(60, 120), (560, 120)]),
           traco("curva_s", 40, curva_s(600, 960, 120, 25)),
           traco("rampa_afunila", 40, [(60, 300), (560, 300)], p=lambda t: 0.15 + 0.85 * math.sin(t * math.pi)),
           traco("recto_rapido", 40, [(600, 300), (960, 300)], passo=25.0),
           traco("recto_baixo", 40, [(60, 480), (560, 480)]),
           {"op": "cor", "rgb": VERMELHO},
           traco("cruza_vermelho", 40, [(300, 400), (300, 560)]),
           traco("recto_vermelho", 40, [(700, 480), (960, 480)]),
           {"op": "simula", "quadros": 480}, {"op": "foto", "nome": "A"}, {"op": "seca"}, {"op": "foto", "nome": "B"}]
    return {"versao": 1, "legado": True, "tela": [W, H], "papel": [255, 255, 255],
            "cenas": [{"nome": "legado", "ops": ops}]}


if __name__ == "__main__":
    print(json.dumps(legado(), separators=(",", ":")))
