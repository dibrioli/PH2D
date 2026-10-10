#!/usr/bin/env python3
"""A bancada da W1e no Rebelle (09/10): a MISTURA (F6) e o PAPEL do pincel seco (F3), eventos Motion IO.

Três ficheiros por tela, para a foto DURANTE o traço (o ponteiro ainda em baixo):
  <nome>_0.json  NEW_ARTWORK + semente + tela direita (o zoom 100 % mete-se depois, por clique)
  <nome>_1.json  os traços até ao último MOVE do último gesto (sem o RELEASE)
  <nome>_2.json  o RELEASE do último gesto

uso: mistura_papel.py cruz <dir> <nome> --estado molhado|seco [--agua N]   (sem --agua = a do preset)
     mistura_papel.py papel <dir> <nome> --papel "Grupo/Nome"

`cruz` — a geometria da sonda `wetpaint/sonda_mistura.rs` deslocada de (+200, +170) numa tela 1000×640:
vermelho (200, 30, 30) horizontal y = 320, x 300 → 700; azul (30, 60, 210) vertical x = 500, y 210 → 430,
Watercolor/Round a 30 px, pressão 1, um ponto a cada 3 px. `seco`: SIMULATION +400 e DRY entre os dois.
`papel` — três traços de 600 px a 60 px (x 200 → 800) em y = 150 · 320 · 490: Watercolor/Flat Dry,
Watercolor/Bristle e Watercolor/Round, cada um com a água DO PRESET, o azul da bancada das gotas.
"""
import argparse
import json
import os

VERMELHO = {"r": 200, "g": 30, "b": 30}
AZUL_CRUZ = {"r": 30, "g": 60, "b": 210}
AZUL = {"r": 23, "g": 101, "b": 179}


def set_brush(preset, size_px, cor, agua):
    sb = {"event_type": "SET_BRUSH", "tool": "WATERCOLOR", "preset": preset, "size_px": size_px,
          "pressure": 100, "color": cor}
    sb["size"] = round(100.0 * (size_px / 300.0) ** (2.0 / 3.0), 1)
    if agua is not None:
        sb["water"] = float(agua)
    return sb


def ponto(tipo, x, y, pr):
    return {"event_type": tipo, "pos": {"x": round(x, 2), "y": round(y, 2)}, "pen_tilt": {"x": 0, "y": 0},
            "rotation": 0.0, "pressure": pr}


def gesto(pts):
    """PRESS + MOVEs (sem o RELEASE) e o RELEASE à parte."""
    return ([ponto("POINTER_PRESS", *pts[0], 1.0)] + [ponto("POINTER_MOVE", x, y, 1.0) for x, y in pts[1:]],
            ponto("POINTER_RELEASE", *pts[-1], 0.0))


def linha(x0, y0, x1, y1, passo=3.0):
    n = int(max(abs(x1 - x0), abs(y1 - y0)) / passo)
    return [(x0 + (x1 - x0) * k / n, y0 + (y1 - y0) * k / n) for k in range(n + 1)]


def tela(papel):
    return [
        {"event_type": "NEW_ARTWORK", "width": 1000, "height": 640, "units": "px", "dpi": 72,
         "paper": {"preset": papel, "color": {"r": 255, "g": 255, "b": 255}, "deckled_edges": False,
                   "paper_scale": 100, "visible": True}},
        {"event_type": "SET_RANDOM_SEED", "seed": 7},
        {"event_type": "SET_CANVAS_TILT", "tilt": {"x": 0, "y": 0}, "enabled": False},
    ]


def motor(influencia, textura):
    """A influência do papel NÃO vem no NEW_ARTWORK: é do motor (⛔ SET_PAPER derruba o 8.3.4). Os valores
    de fábrica por papel (Visual Settings): Rough 6 · Cold 3, textura 5."""
    if influencia is None:
        return []
    return [{"event_type": "SET_ENGINE_PARAMS", "texture_influence": influencia, "paper_texture": textura,
             "paint_texture": textura}]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("modo", choices=["cruz", "papel"])
    ap.add_argument("dir")
    ap.add_argument("nome")
    ap.add_argument("--estado", choices=["molhado", "seco"], default="molhado")
    ap.add_argument("--agua", type=float, default=None)
    ap.add_argument("--papel", default="Default/EX00 White Simple")
    ap.add_argument("--influencia", type=float, default=None, help="0–10, texture_influence")
    ap.add_argument("--textura", type=float, default=5.0, help="0–10, paper_texture e paint_texture")
    a = ap.parse_args()
    if a.modo == "cruz":
        t0 = tela("Default/EX00 White Simple")
        corpo, fim_v = gesto(linha(300, 320, 700, 320))
        t1 = [set_brush("Watercolor/Round", 30.0, VERMELHO, a.agua)] + corpo + [fim_v]
        if a.estado == "seco":
            t1 += [{"event_type": "SIMULATION", "repeats": 400}, {"event_type": "DRY"}]
        corpo, fim = gesto(linha(500, 210, 500, 430))
        t1 += [set_brush("Watercolor/Round", 30.0, AZUL_CRUZ, a.agua)] + corpo
        t2 = [fim]
    else:
        # Uma vez, no início: depois de um SET_BRUSH, um SET_ENGINE_PARAMS reduz o pincel a um fio (medido).
        t0 = tela(a.papel) + motor(a.influencia, a.textura)
        t1, t2 = [], []
        for k, preset in enumerate(["Watercolor/Flat Dry", "Watercolor/Bristle", "Watercolor/Round"]):
            corpo, fim = gesto(linha(200, 150 + 170 * k, 800, 150 + 170 * k))
            t1 += [set_brush(preset, 60.0, AZUL, None)] + corpo
            if k < 2:
                t1.append(fim)
            else:
                t2 = [fim]
    for i, ev in enumerate([t0, t1, t2]):
        with open(os.path.join(a.dir, f"{a.nome}_{i}.json"), "w") as f:
            json.dump({"frames": [{"events": ev}]}, f)


if __name__ == "__main__":
    main()
