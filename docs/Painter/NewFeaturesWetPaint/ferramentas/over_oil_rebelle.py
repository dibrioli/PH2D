#!/usr/bin/env python3
"""A cena do Over Oil (J9, 30/10) no Rebelle — a MESMA da sonda `wetpaint::agua_sobre_oleo_tests::diag_aguada_por_cima_do_oleo`,
deslocada de (+380, +240) numa tela 1000×640: papel (150, 130, 100), óleo `Dabber/Dabber` 30 px azul (40, 90, 220) na
horizontal y = 90, x 40 → 200 a passo 4; depois a aguada `Watercolor/Round` 40 px (`--aguada`) amarela (220, 200, 40) na vertical
x = 120, y 30 → 142 a passo 2,8.

Três ficheiros por variante, para a foto A MEIO (o ponteiro ainda em baixo):
  <nome>_0.json  NEW_ARTWORK + semente + o óleo + (molhado: SIMULATION 120 · seco: SIMULATION 120 + DRY)
  <nome>_1.json  a aguada até ao último MOVE (sem o RELEASE)
  <nome>_2.json  o RELEASE + SIMULATION 600 (a nossa sonda: 600 passos de 1/60 s)

uso: over_oil_rebelle.py <dir> <nome> --oleo molhado|seco [--papel r,g,b]
"""
import argparse
import json
import os

DX, DY = 380, 240


def ponto(tipo, x, y, pr):
    return {"event_type": tipo, "pos": {"x": round(x + DX, 2), "y": round(y + DY, 2)}, "pen_tilt": {"x": 0, "y": 0},
            "rotation": 0.0, "pressure": pr}


def set_brush(tool, preset, size_px, mx, cor):
    return {"event_type": "SET_BRUSH", "tool": tool, "preset": preset, "size_px": size_px, "pressure": 100,
            "color": dict(zip("rgb", cor)), "size": round(100.0 * (size_px / mx) ** (2.0 / 3.0), 1)}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("dir")
    ap.add_argument("nome")
    ap.add_argument("--oleo", choices=["molhado", "seco", "nenhum"], default="molhado")
    ap.add_argument("--papel", default="150,130,100")
    ap.add_argument("--aguada", default="Watercolor/Round")
    ap.add_argument("--tam", type=float, default=40.0, help="size_px da aguada (calibrado à largura da sonda)")
    a = ap.parse_args()
    papel = dict(zip("rgb", map(int, a.papel.split(","))))
    ev0 = [{"event_type": "NEW_ARTWORK", "width": 1000, "height": 640, "units": "px", "dpi": 72,
            "paper": {"preset": "Default/EX00 White Simple", "color": papel, "deckled_edges": False,
                      "paper_scale": 100, "visible": True}},
           {"event_type": "SET_RANDOM_SEED", "seed": 7},
           {"event_type": "SET_CANVAS_TILT", "tilt": {"x": 0, "y": 0}, "enabled": False}]
    if a.oleo != "nenhum":
        ev0.append(set_brush("OIL_AND_ACRYLIC", "Dabber/Dabber", 30.0, 300.0, (40, 90, 220)))
        pts = [(40.0 + 4.0 * k, 90.0) for k in range(41)]
        ev0 += [ponto("POINTER_PRESS", *pts[0], 1.0)] + [ponto("POINTER_MOVE", x, y, 1.0) for x, y in pts[1:]]
        ev0.append(ponto("POINTER_RELEASE", *pts[-1], 0.0))
        ev0.append({"event_type": "SIMULATION", "repeats": 120})
        if a.oleo == "seco":
            ev0.append({"event_type": "DRY"})
    # o Flat do Rebelle não gira a lâmina: na vertical pinta o fio. O Round a 40 px dá a largura da sonda; o
    # SET_BRUSH vai DUAS vezes (a troca de ferramenta come o 1.º, medido 30/10: 3 · 10 · 16 px em três telas).
    sb = set_brush("WATERCOLOR", a.aguada, a.tam, 300.0, (220, 200, 40))
    ev1 = [sb, dict(sb)]
    pts = [(120.0, 30.0 + 2.8 * k) for k in range(41)]
    ev1 += [ponto("POINTER_PRESS", *pts[0], 1.0)] + [ponto("POINTER_MOVE", x, y, 1.0) for x, y in pts[1:]]
    ev2 = [ponto("POINTER_RELEASE", 120.0, 142.0, 0.0), {"event_type": "SIMULATION", "repeats": 600}]
    for i, ev in enumerate((ev0, ev1, ev2)):
        json.dump({"frames": [{"events": ev}]}, open(os.path.join(a.dir, f"{a.nome}_{i}.json"), "w"))


if __name__ == "__main__":
    main()
