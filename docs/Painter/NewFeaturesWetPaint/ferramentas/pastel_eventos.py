#!/usr/bin/env python3
"""Os gestos do prompt 35 para o Rebelle «Pastel Dry / Dry Thin», os MESMOS da sonda do PH2D
(`crates/ph2d-tool-painter/src/tool/paint/wetpaint/sonda_pastel_tempo.rs`): o azul da
bancada, pressão 1, 4 px por evento, um recto de 220 px e um zig-zague fechado. Tela 1000×640: a 640×360 o
Rebelle abre a 221 % e o tamanho do pincel é medido no ECRÃ (29 px saíam 14, medido 14/10).

uso: pastel_eventos.py <pasta_saida> [--size_px 29] [--preset "Pastel Dry/Dry Thin"]
Escreve em <pasta_saida>: `setup.json` (artwork + semente) e, por gesto, `<g>_a.json` (o SET_BRUSH, o PRESS e
a primeira metade) e `<g>_b.json` (o resto e o RELEASE) — a foto «a meio» tira-se entre os dois.
"""
import argparse
import json
import math
from pathlib import Path

AZUL = {"r": 23, "g": 101, "b": 179}
PASSO = 4.0
GESTOS = {
    "recto": [(60.0, 70.0), (280.0, 70.0)],
    "zigue": [(70.0, 190.0), (420.0, 150.0), (90.0, 240.0), (440.0, 200.0), (120.0, 290.0),
              (460.0, 250.0), (150.0, 320.0)],
}


def amostra(pts):
    out = [pts[0]]
    for a, b in zip(pts, pts[1:]):
        n = max(1, math.ceil(math.dist(a, b) / PASSO))
        out += [(a[0] + (b[0] - a[0]) * k / n, a[1] + (b[1] - a[1]) * k / n) for k in range(1, n + 1)]
    return out


def ptr(tipo, x, y, p):
    return {"event_type": tipo, "pos": {"x": round(x, 2), "y": round(y, 2)}, "pen_tilt": {"x": 0, "y": 0},
            "rotation": 0.0, "pressure": p}


def doc(ev):
    return json.dumps({"frames": [{"events": ev}]})


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("pasta")
    ap.add_argument("--size_px", type=float, default=29.0)
    ap.add_argument("--preset", default="Pastel Dry/Dry Thin")
    ap.add_argument("--tool", default="PASTEL", help="PASTEL · PENCIL · … (gera_eventos.py)")
    ap.add_argument("--max", type=float, default=300.0, help="o max_brush_size do preset (fixture dos 281)")
    a = ap.parse_args()
    out = Path(a.pasta)
    out.mkdir(parents=True, exist_ok=True)
    (out / "setup.json").write_text(doc([
        {"event_type": "NEW_ARTWORK", "width": 1000, "height": 640, "units": "px", "dpi": 72,
         "paper": {"preset": "Default/EX00 White Simple", "color": {"r": 255, "g": 255, "b": 255},
                   "deckled_edges": False, "paper_scale": 100, "visible": True}},
        {"event_type": "SET_RANDOM_SEED", "seed": 7},
    ]))
    # O tamanho em px E em % do Max Size (gera_eventos.py, REPETE_PARAMS); a opacidade é a do preset.
    sb = {"event_type": "SET_BRUSH", "tool": a.tool, "preset": a.preset, "size_px": a.size_px,
          "size": round(100.0 * (a.size_px / a.max) ** (2.0 / 3.0), 1), "pressure": 100, "color": AZUL}
    for g, pts in GESTOS.items():
        s = amostra(pts)
        meio = len(s) // 2
        ev_a = [sb, ptr("POINTER_PRESS", *s[0], 1.0)] + [ptr("POINTER_MOVE", *q, 1.0) for q in s[1:meio + 1]]
        ev_b = [ptr("POINTER_MOVE", *q, 1.0) for q in s[meio + 1:]] + [ptr("POINTER_RELEASE", *s[-1], 0.0)]
        (out / f"{g}_a.json").write_text(doc(ev_a))
        (out / f"{g}_b.json").write_text(doc(ev_b))


if __name__ == "__main__":
    main()
