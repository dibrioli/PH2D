#!/usr/bin/env python3
"""A bancada da MISTURA EM CAMADAS (10/10, prompt 15): amarelo sobre azul SECO, 1…10 passadas, eventos Motion IO.

A geometria é a da sonda `wetpaint/mistura_camadas_tests.rs` (tela 600×400) deslocada de (+200, +120) numa tela
1000×640. Tudo Watercolor/Round a 30 px, pressão 1, um ponto a cada 3 px, a água DO PRESET:
  - a referência: um traço AMARELO no branco, y = 360, x 100 → 500 (a cor do amarelo sozinho);
  - a mancha AZUL (30, 100, 220): 9 traços horizontais y = 120 … 280 (passo 20), x 150 → 450; SIMULATION 400 + DRY;
  - cada PASSADA de amarelo (245, 205, 25): UM gesto em serpentina sobre o miolo, linhas y = 170 · 190 · 210 · 230,
    x 250 ↔ 350.
O miolo lê-se em x 285–315, y 185–215 (coordenadas da sonda).

Ficheiros: <nome>_base.json (tela + referência + azul + DRY) e <nome>_pk.json (a passada k, com o RELEASE). Entre
passadas, o modo `molhado` só espera (o runner fotografa ~1,5 s depois do Up); o `seco` junta SIMULATION 400 + DRY ao
fim de cada passada (a velatura clássica: cada camada seca antes da seguinte).

uso: camadas.py <dir> <nome> --estado molhado|seco [--passadas 10]
"""
import argparse
import json
import os

AZUL = {"r": 30, "g": 100, "b": 220}
AMARELO = {"r": 245, "g": 205, "b": 25}
DX, DY = 200, 120


def set_brush(cor):
    size_px = 30.0
    return {"event_type": "SET_BRUSH", "tool": "WATERCOLOR", "preset": "Watercolor/Round", "size_px": size_px,
            "pressure": 100, "color": cor, "size": round(100.0 * (size_px / 300.0) ** (2.0 / 3.0), 1)}


def ponto(tipo, x, y, pr):
    return {"event_type": tipo, "pos": {"x": round(x + DX, 2), "y": round(y + DY, 2)}, "pen_tilt": {"x": 0, "y": 0},
            "rotation": 0.0, "pressure": pr}


def gesto(cor, pts):
    """SET_BRUSH completo (só vale para o 1.º traço: reenvia-se antes de cada gesto) + PRESS, MOVEs, RELEASE."""
    return ([set_brush(cor), ponto("POINTER_PRESS", *pts[0], 1.0)]
            + [ponto("POINTER_MOVE", x, y, 1.0) for x, y in pts[1:]]
            + [ponto("POINTER_RELEASE", *pts[-1], 0.0)])


def linha(x0, y0, x1, y1, passo=3.0):
    n = max(1, int(max(abs(x1 - x0), abs(y1 - y0)) / passo))
    return [(x0 + (x1 - x0) * k / n, y0 + (y1 - y0) * k / n) for k in range(n + 1)]


def serpentina():
    pts = []
    for j, y in enumerate([170, 190, 210, 230]):
        x0, x1 = (250, 350) if j % 2 == 0 else (350, 250)
        if pts:
            pts += linha(pts[-1][0], pts[-1][1], x0, y)[1:]
        pts += linha(x0, y, x1, y)[1 if pts else 0:]
    return pts


def seca():
    return [{"event_type": "SIMULATION", "repeats": 400}, {"event_type": "DRY"}]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("dir")
    ap.add_argument("nome")
    ap.add_argument("--estado", choices=["molhado", "seco"], default="molhado")
    ap.add_argument("--passadas", type=int, default=10)
    a = ap.parse_args()
    base = [
        {"event_type": "NEW_ARTWORK", "width": 1000, "height": 640, "units": "px", "dpi": 72,
         "paper": {"preset": "Default/EX00 White Simple", "color": {"r": 255, "g": 255, "b": 255},
                   "deckled_edges": False, "paper_scale": 100, "visible": True}},
        {"event_type": "SET_RANDOM_SEED", "seed": 7},
        {"event_type": "SET_CANVAS_TILT", "tilt": {"x": 0, "y": 0}, "enabled": False},
    ]
    base += gesto(AMARELO, linha(100, 360, 500, 360))
    for y in range(120, 281, 20):
        base += gesto(AZUL, linha(150, y, 450, y))
    base += seca()
    docs = {"base": base}
    for k in range(1, a.passadas + 1):
        ev = gesto(AMARELO, serpentina())
        if a.estado == "seco":
            ev += seca()
        docs[f"p{k}"] = ev
    for nome, ev in docs.items():
        with open(os.path.join(a.dir, f"{a.nome}_{nome}.json"), "w") as f:
            json.dump({"frames": [{"events": ev}]}, f)


if __name__ == "__main__":
    main()
