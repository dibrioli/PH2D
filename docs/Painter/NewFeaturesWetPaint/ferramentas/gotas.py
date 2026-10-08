#!/usr/bin/env python3
"""A bancada das GOTAS (escorrimento) do Rebelle — doc 06. Gera os eventos Motion IO de UMA tela:
quatro traços horizontais lado a lado (150 px, a 40 px, pressão 1, azul da bancada), com a inclinação
da tela e o comportamento das gotas dados na linha de comando.

uso: gotas.py <nome> [--tilt X Y | --sem_tilt] [--drips on|off] [--drip_size N] [--drip_length N]
              [--colunas "TOOL|preset|agua" ×4]   (agua vazia = a do preset)
Escreve {"frames":[{"events":[…]}]} em stdout. Tela 1000×640, papel branco, semente 7. Nada de SAVE (a
porta não o suporta): a foto é da janela (foto.sh).
"""
import argparse
import json

AZUL = {"r": 23, "g": 101, "b": 179}
# O Max Size dos presets usados (o SET_BRUSH leva o tamanho também em %: size_px ≈ max·(s/100)^1.5).
MAX = {"Watercolor/Round": 300, "Watercolor/Wash Round": 300, "Watercolor/Mop": 300, "Pen/Dip Pen": 100}
COLUNAS = ["WATERCOLOR|Watercolor/Round|100", "WATERCOLOR|Watercolor/Round|70",
           "WATERCOLOR|Watercolor/Round|40", "WATERCOLOR|Watercolor/Round|20"]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("nome")
    ap.add_argument("--tilt", type=float, nargs=2, default=None)
    ap.add_argument("--sem_tilt", action="store_true")
    ap.add_argument("--drips", choices=["on", "off"], default="on")
    ap.add_argument("--drip_size", type=int, default=5)
    ap.add_argument("--drip_length", type=int, default=5)
    ap.add_argument("--colunas", nargs=4, default=COLUNAS)
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
    for i, col in enumerate(a.colunas):
        tool, preset, agua = col.split("|")
        sb = {"event_type": "SET_BRUSH", "tool": tool, "preset": preset, "size_px": 40.0, "pressure": 100,
              "color": AZUL}
        sb["size"] = round(100.0 * (40.0 / MAX.get(preset, 300)) ** (2.0 / 3.0), 1)
        if agua:
            sb["water"] = float(agua)
        ev.append(sb)
        x0, y = 60 + i * 240, 150
        def p(tipo, x, pr):
            return {"event_type": tipo, "pos": {"x": round(x, 2), "y": y}, "pen_tilt": {"x": 0, "y": 0},
                    "rotation": 0.0, "pressure": pr}
        ev.append(p("POINTER_PRESS", x0, 1.0))
        for k in range(1, 51):
            ev.append(p("POINTER_MOVE", x0 + 3 * k, 1.0))
        ev.append(p("POINTER_RELEASE", x0 + 150, 0.0))
    ev.append({"event_type": "BOOKMARK", "id": "fim_" + a.nome})
    print(json.dumps({"frames": [{"events": ev}]}))


if __name__ == "__main__":
    main()
