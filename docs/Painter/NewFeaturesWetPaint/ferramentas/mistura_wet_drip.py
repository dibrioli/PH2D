#!/usr/bin/env python3
"""A bancada da MISTURA com escorrimento (10/10, prompt 16): azul ESCURO escorrido sobre um traço VERMELHO, no Rebelle,
eventos Motion IO — a cena do smoke do dono (`fixtures/smoke_dono_mistura_wet_drip/`), o equivalente do Wet Drip.

Tela 1000×640 branca (`Default/EX00 White Simple`), semente 7, sem SET_ENGINE_PARAMS (as gotas de fábrica, 5/5: a
mesma condição da linha «Wet Drip · Rebelle Round água 100» do doc 05 §4b, que escorre 344–348 px).
  - o VERMELHO (200, 30, 30), o da sonda: Watercolor/Round 60 px, água DO PRESET, tela DIREITA, um traço horizontal
    y = 400, x 120 → 880 (a banda y ≈ 370–430);
  - `seco`: SIMULATION 400 + DRY a seguir ao vermelho; `molhado`: nada (o azul cai no vermelho ainda molhado);
  - depois `SET_CANVAS_TILT (0, 1)` ligado (o máximo, a tela inclinada para baixo);
  - cada PASSADA de AZUL ESCURO (20, 50, 110), Watercolor/Round 60 px, água 100, pressão 1, dois gestos:
      E (esquerda) x 150 → 450 em y = 190 + 10·(k−1) — ACIMA do vermelho; só os fios lá chegam (pontas dos fios);
      D (direita)  x 550 → 850 em y = 340 + 10·(k−1) — a orla de baixo do azul cai DENTRO do vermelho (a orla de
      baixo da mancha e a fronteira azul/vermelho).
Um ponto a cada 3 px; o SET_BRUSH completo antes de cada gesto (o 8.3.4 só o aplica ao 1.º traço depois dele).

Ficheiros: <nome>_base.json (tela + vermelho [+ DRY] + tilt) e <nome>_p<k>.json (a passada k, com os RELEASE).

uso: mistura_wet_drip.py <dir> <nome> --estado molhado|seco [--passadas 4] [--opacidade 0..100]
`--opacidade 100` = o equivalente da Opacity 1 do nosso Wet Drip (sem ela, a do preset: o traço sai DILUÍDO, ~0,67 de
branco, doc 05 §4b).
"""
import argparse
import json
import os

VERMELHO = {"r": 200, "g": 30, "b": 30}
AZUL_ESCURO = {"r": 20, "g": 50, "b": 110}
SIZE_PX = 60.0


def set_brush(cor, agua, opac=None):
    sb = {"event_type": "SET_BRUSH", "tool": "WATERCOLOR", "preset": "Watercolor/Round", "size_px": SIZE_PX,
          "pressure": 100, "color": cor, "size": round(100.0 * (SIZE_PX / 300.0) ** (2.0 / 3.0), 1)}
    if agua is not None:
        sb["water"] = float(agua)
    if opac is not None:
        sb["opacity"] = float(opac)
    return sb


def ponto(tipo, x, y, pr):
    return {"event_type": tipo, "pos": {"x": round(x, 2), "y": round(y, 2)}, "pen_tilt": {"x": 0, "y": 0},
            "rotation": 0.0, "pressure": pr}


def linha(x0, y0, x1, y1, passo=3.0):
    n = max(1, int(max(abs(x1 - x0), abs(y1 - y0)) / passo))
    return [(x0 + (x1 - x0) * k / n, y0 + (y1 - y0) * k / n) for k in range(n + 1)]


def gesto(cor, agua, pts, opac=None):
    return ([set_brush(cor, agua, opac), ponto("POINTER_PRESS", *pts[0], 1.0)]
            + [ponto("POINTER_MOVE", x, y, 1.0) for x, y in pts[1:]]
            + [ponto("POINTER_RELEASE", *pts[-1], 0.0)])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("dir")
    ap.add_argument("nome")
    ap.add_argument("--estado", choices=["molhado", "seco"], default="molhado")
    ap.add_argument("--passadas", type=int, default=4)
    ap.add_argument("--opacidade", type=float, default=None, help="0–100, a Opacity do SET_BRUSH nos DOIS (sem = a do preset)")
    a = ap.parse_args()
    base = [
        {"event_type": "NEW_ARTWORK", "width": 1000, "height": 640, "units": "px", "dpi": 72,
         "paper": {"preset": "Default/EX00 White Simple", "color": {"r": 255, "g": 255, "b": 255},
                   "deckled_edges": False, "paper_scale": 100, "visible": True}},
        {"event_type": "SET_RANDOM_SEED", "seed": 7},
        {"event_type": "SET_CANVAS_TILT", "tilt": {"x": 0, "y": 0}, "enabled": False},
    ]
    base += gesto(VERMELHO, None, linha(120, 400, 880, 400), a.opacidade)
    if a.estado == "seco":
        base += [{"event_type": "SIMULATION", "repeats": 400}, {"event_type": "DRY"}]
    base.append({"event_type": "SET_CANVAS_TILT", "tilt": {"x": 0, "y": 1}, "enabled": True})
    with open(os.path.join(a.dir, f"{a.nome}_base.json"), "w") as f:
        json.dump({"frames": [{"events": base}]}, f)
    for k in range(1, a.passadas + 1):
        ev = gesto(AZUL_ESCURO, 100, linha(150, 190 + 10 * (k - 1), 450, 190 + 10 * (k - 1)), a.opacidade)
        ev += gesto(AZUL_ESCURO, 100, linha(550, 340 + 10 * (k - 1), 850, 340 + 10 * (k - 1)), a.opacidade)
        with open(os.path.join(a.dir, f"{a.nome}_p{k}.json"), "w") as f:
            json.dump({"frames": [{"events": ev}]}, f)


if __name__ == "__main__":
    main()
