#!/usr/bin/env python3
"""A bancada das GOTAS (escorrimento) do Rebelle — doc 06. Gera os eventos Motion IO de UMA tela:
quatro traços horizontais lado a lado (150 px, a 40 px, pressão 1, azul da bancada), com a inclinação
da tela e o comportamento das gotas dados na linha de comando.

uso: gotas.py <nome> [--tilt X Y | --sem_tilt] [--drips on|off] [--drip_size N] [--drip_length N]
              [--colunas "TOOL|preset|agua" ×4]   (agua vazia = a do preset)
              [--forma s [--size_px 100] [--agua 100]]
`--forma s`: em vez dos quatro traços, UM traço em S pela tela (sobe da esquerda-baixo, arco, vale, sobe à
direita; y = 180 − 85·sin θ, θ ∈ [−π/2, 2π + π/4], x 100 → 900), um ponto a cada 3 px de arco,
Watercolor/Round, pressão 1, o mesmo azul — a orla de baixo de uma CURVA com a tela inclinada.
Escreve {"frames":[{"events":[…]}]} em stdout. Tela 1000×640, papel branco, semente 7. Nada de SAVE (a
porta não o suporta): a foto é da janela (foto.sh).
"""
import argparse
import json
import math

AZUL = {"r": 23, "g": 101, "b": 179}
# O Max Size dos presets usados (o SET_BRUSH leva o tamanho também em %: size_px ≈ max·(s/100)^1.5).
MAX = {"Watercolor/Round": 300, "Watercolor/Wash Round": 300, "Watercolor/Mop": 300, "Pen/Dip Pen": 100}
COLUNAS = ["WATERCOLOR|Watercolor/Round|100", "WATERCOLOR|Watercolor/Round|70",
           "WATERCOLOR|Watercolor/Round|40", "WATERCOLOR|Watercolor/Round|20"]


def set_brush(tool, preset, size_px, agua):
    sb = {"event_type": "SET_BRUSH", "tool": tool, "preset": preset, "size_px": size_px, "pressure": 100,
          "color": AZUL}
    sb["size"] = round(100.0 * (size_px / MAX.get(preset, 300)) ** (2.0 / 3.0), 1)
    if agua:
        sb["water"] = float(agua)
    return sb


def ponto(tipo, x, y, pr):
    return {"event_type": tipo, "pos": {"x": round(x, 2), "y": round(y, 2)}, "pen_tilt": {"x": 0, "y": 0},
            "rotation": 0.0, "pressure": pr}


def s_pontos(passo=3.0):
    """O S da tela 1000×640, reamostrado a `passo` px de comprimento de arco."""
    x0, x1, yc, amp = 100.0, 900.0, 180.0, 85.0
    t0, t1 = -math.pi / 2, 2 * math.pi + math.pi / 4
    fino = [(x0 + (x1 - x0) * k / 8000, yc - amp * math.sin(t0 + (t1 - t0) * k / 8000)) for k in range(8001)]
    pts, acc = [fino[0]], 0.0
    for (ax, ay), (bx, by) in zip(fino, fino[1:]):
        acc += math.hypot(bx - ax, by - ay)
        if acc >= passo:
            pts.append((bx, by))
            acc = 0.0
    return pts


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("nome")
    ap.add_argument("--tilt", type=float, nargs=2, default=None)
    ap.add_argument("--sem_tilt", action="store_true")
    ap.add_argument("--drips", choices=["on", "off"], default="on")
    ap.add_argument("--drip_size", type=int, default=5)
    ap.add_argument("--drip_length", type=int, default=5)
    ap.add_argument("--colunas", nargs=4, default=COLUNAS)
    ap.add_argument("--forma", choices=["tracos", "s"], default="tracos")
    ap.add_argument("--size_px", type=float, default=100.0, help="só --forma s")
    ap.add_argument("--agua", default="100", help="só --forma s (vazia = a do preset)")
    a = ap.parse_args()
    ev = [
        {"event_type": "NEW_ARTWORK", "width": 1000, "height": 640, "units": "px", "dpi": 72,
         "paper": {"preset": "Default/EX00 White Simple", "color": {"r": 255, "g": 255, "b": 255},
                   "deckled_edges": False, "paper_scale": 100, "visible": True}},
        {"event_type": "SET_RANDOM_SEED", "seed": 7},
        {"event_type": "SET_ENGINE_PARAMS", "create_drips": a.drips == "on", "drip_size": a.drip_size,
         "drip_length": a.drip_length},
    ]
    if a.sem_tilt:
        ev.append({"event_type": "SET_CANVAS_TILT", "tilt": {"x": 0, "y": 0}, "enabled": False})
    elif a.tilt:
        ev.append({"event_type": "SET_CANVAS_TILT", "tilt": {"x": a.tilt[0], "y": a.tilt[1]}, "enabled": True})
    if a.forma == "s":
        ev.append(set_brush("WATERCOLOR", "Watercolor/Round", a.size_px, a.agua))
        pts = s_pontos()
        ev.append(ponto("POINTER_PRESS", *pts[0], 1.0))
        ev += [ponto("POINTER_MOVE", x, y, 1.0) for (x, y) in pts[1:-1]]
        ev.append(ponto("POINTER_RELEASE", *pts[-1], 0.0))
    for i, col in enumerate(a.colunas if a.forma == "tracos" else []):
        tool, preset, agua = col.split("|")
        ev.append(set_brush(tool, preset, 40.0, agua))
        x0, y = 60 + i * 240, 150
        ev.append(ponto("POINTER_PRESS", x0, y, 1.0))
        for k in range(1, 51):
            ev.append(ponto("POINTER_MOVE", x0 + 3 * k, y, 1.0))
        ev.append(ponto("POINTER_RELEASE", x0 + 150, y, 0.0))
    ev.append({"event_type": "BOOKMARK", "id": "fim_" + a.nome})
    print(json.dumps({"frames": [{"events": ev}]}))


if __name__ == "__main__":
    main()
