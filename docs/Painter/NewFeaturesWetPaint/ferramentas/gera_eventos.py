#!/usr/bin/env python3
"""Gera os eventos Motion IO (JSON) do Rebelle para os MESMOS cinco gestos da sonda do PH2D
(`crates/ph2d-tool-painter/tests/it/sonda_gestos.rs`): recto a passo de rato (3 px), curva em S,
rampa de pressão 0,15→1→0,15, traço RÁPIDO (25 px por evento) e um cruzamento vermelho.

uso: gera_eventos.py <TOOL> <preset|-> <pasta_saida_windows> <nome> [--size_px N] [--sim N] [--paper P] [--seed S]
  TOOL: OIL_AND_ACRYLIC · EXPRESS_OIL · WATERCOLOR · INK_PEN · PENCIL · PASTEL · MARKER · AIRBRUSH …
  preset: "Grupo/Nome" como no Brushes panel (ex. "Watercolor/Flat 2"), ou - para ficar com o actual.
Escreve o JSON em stdout: {"frames":[{"events":[…]}]}. Tela 1000×640 px (a da sonda), papel branco.
"""
import argparse
import json
import math

AZUL = {"r": 23, "g": 101, "b": 179}
VERMELHO = {"r": 191, "g": 0, "b": 18}


TILT = {"x": 0, "y": 0}
PASTA = {"WATERCOLOR": "Watercolor", "OIL_AND_ACRYLIC": "Oil", "EXPRESS_OIL": "ExpressOil", "PENCIL": "Pencil",
         "PASTEL": "Pastel", "INK_PEN": "Ink", "MARKER": "Marker", "AIRBRUSH": "Airbrush"}


def max_do_preset(tool, preset):
    """O `max_brush_size` do preset, lido da fixture dos 281 (`fixtures/rebelle_presets_281.json`); 300 se faltar."""
    from pathlib import Path
    fx = Path(__file__).resolve().parent.parent / "fixtures" / "rebelle_presets_281.json"
    try:
        alvo = f"/{PASTA[tool]}/{preset}.rebelle-brush.png"
        for p in json.loads(fx.read_text())["pinceis"]:
            if p["file"].endswith(alvo):
                return float(p["Parameters"]["max_brush_size"])
    except (KeyError, OSError):
        pass
    return 300.0


def ptr(tipo, x, y, p, extra=None):
    e = {"event_type": tipo, "pos": {"x": round(x, 2), "y": round(y, 2)}, "pen_tilt": dict(TILT),
         "rotation": 0.0, "pressure": round(p, 3)}
    if extra:
        e.update(extra)
    return e


def gesto(ev, pts, passo, p):
    """Polilinha com `passo` px entre eventos; p(t) é a pressão ao longo do comprimento t∈[0,1]."""
    total = sum(math.dist(a, b) for a, b in zip(pts, pts[1:]))
    andado = 0.0
    ev.append(ptr("POINTER_PRESS", pts[0][0], pts[0][1], p(0.0)))
    for a, b in zip(pts, pts[1:]):
        ln = math.dist(a, b)
        n = max(1, math.ceil(ln / passo))
        for k in range(1, n + 1):
            f = k / n
            andado += ln / n
            ev.append(ptr("POINTER_MOVE", a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f, p(andado / total)))
    fim = pts[-1]
    ev.append(ptr("POINTER_RELEASE", fim[0], fim[1], 0.0))


def curva_s(x0, x1, cy, amp):
    return [(x0 + (x1 - x0) * i / 120, cy + amp * math.sin(i / 120 * math.tau * 1.5)) for i in range(121)]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("tool"); ap.add_argument("preset"); ap.add_argument("pasta"); ap.add_argument("nome")
    ap.add_argument("--size_px", type=float, default=40.0); ap.add_argument("--sim", type=int, default=240)
    ap.add_argument("--paper", default="Default/EX00 White Simple"); ap.add_argument("--seed", type=int, default=7)
    ap.add_argument("--water", type=float, default=None); ap.add_argument("--opacity", type=float, default=None)
    ap.add_argument("--so_tracos", action="store_true", help="não cria artwork nem grava — só os traços")
    ap.add_argument("--tilt", type=float, nargs=2, default=None, help="pen_tilt x y (graus) em todos os eventos")
    ap.add_argument("--dirty", action="store_true", help="dirty_brush no SET_BRUSH")
    ap.add_argument("--paint_type", default=None)
    ap.add_argument("--cor_base", default=None, help="r,g,b da linha 3 (o azul da bancada por omissão)")
    ap.add_argument("--cor_cruz", default=None, help="r,g,b do cruzamento (o vermelho por omissão)")
    ap.add_argument("--seca_cruz", type=int, default=None,
                    help="SIMULATION N + DRY antes do cruzamento vermelho: o azul da linha 3 SECO (prompt 42)")
    a = ap.parse_args()
    ev = []
    if not a.so_tracos:
        ev.append({"event_type": "NEW_ARTWORK", "width": 1000, "height": 640, "units": "px", "dpi": 72,
                   "paper": {"preset": a.paper, "color": {"r": 255, "g": 255, "b": 255}, "deckled_edges": False,
                             "paper_scale": 100, "visible": True}})
        ev.append({"event_type": "SET_RANDOM_SEED", "seed": a.seed})
    sb = {"event_type": "SET_BRUSH", "tool": a.tool, "size_px": a.size_px, "pressure": 100, "color": AZUL}
    if a.preset != "-":
        sb["preset"] = a.preset
    if a.water is not None:
        sb["water"] = a.water
    if a.opacity is not None:
        sb["opacity"] = a.opacity
    if a.dirty:
        sb["dirty_brush"] = True
    if a.paint_type:
        sb["paint_type"] = a.paint_type
    if a.tilt:
        global TILT
        TILT = {"x": a.tilt[0], "y": a.tilt[1]}
    ev.append(sb)
    cheio = lambda t: 1.0
    # REPETE_PARAMS — medido 2026-10-07: o SET_BRUSH só vale para o 1.º traço e o SET_BRUSH_PARAMS não repõe
    # opacidade/água nem o tamanho nas ferramentas finas. Reenvia-se o SET_BRUSH COMPLETO antes de CADA gesto,
    # com o tamanho em px E em % do Max Size do preset (size_px ≈ max·(s/100)^1.5, conferido em 5 presets).
    MAX = {"Bristle/Bristle Flat": 400, "Round/Hairy": 300, "Watercolor/Flat 2": 300, "Watercolor/Round": 300,
           "Pen/Bullet": 100, "Graphite Pencil/8B": 80, "Pastel Dry/Dry Thin": 300, "Marker/Liner": 300,
           "Default/Spray 2": 700}
    mx = MAX.get(a.preset) or max_do_preset(a.tool, a.preset)
    sb["size"] = round(100.0 * (a.size_px / mx) ** (2.0 / 3.0), 1)
    cor_actual = {"cor": AZUL}
    def gesto_rep(ev, pts, passo, p):
        e = dict(sb); e["color"] = cor_actual["cor"]
        ev.append(e)
        gesto(ev, pts, passo, p)
    # linha 1 — recto a passo de rato · curva em S
    gesto_rep(ev, [(60, 120), (560, 120)], 3, cheio)
    gesto_rep(ev, curva_s(600, 960, 120, 25), 3, cheio)
    # linha 2 — rampa de pressão · traço RÁPIDO
    gesto_rep(ev, [(60, 300), (560, 300)], 3, lambda t: 0.15 + 0.85 * math.sin(t * math.pi))
    gesto_rep(ev, [(600, 300), (960, 300)], 25, cheio)
    # linha 3 — recto, cruzamento vermelho e um recto vermelho
    rgb = lambda t: dict(zip("rgb", map(int, t.split(","))))
    if a.cor_base:
        cor_actual["cor"] = rgb(a.cor_base)
        ev.append({"event_type": "SET_BRUSH_COLOR", "color": cor_actual["cor"]})
    gesto_rep(ev, [(60, 480), (560, 480)], 3, cheio)
    if a.seca_cruz is not None:
        ev.append({"event_type": "SIMULATION", "repeats": a.seca_cruz})
        ev.append({"event_type": "DRY"})
    cor_actual["cor"] = rgb(a.cor_cruz) if a.cor_cruz else VERMELHO
    ev.append({"event_type": "SET_BRUSH_COLOR", "color": cor_actual["cor"]})
    gesto_rep(ev, [(300, 400), (300, 560)], 3, cheio)
    cor_actual["cor"] = VERMELHO
    gesto_rep(ev, [(700, 480), (960, 480)], 3, cheio)
    cor_actual["cor"] = AZUL
    ev.append({"event_type": "SET_BRUSH_COLOR", "color": AZUL})
    if not a.so_tracos:
        base = a.pasta.rstrip("/\\") + "/" + a.nome
        ev.append({"event_type": "SAVE", "rgba_canvas": {"filename": base + "_t0.png"}, "rgba_dry": {"filename": base + "_t0_dry.png"}})
        if a.sim > 0:
            ev.append({"event_type": "SIMULATION", "repeats": a.sim})
            ev.append({"event_type": "SAVE", "rgba_canvas": {"filename": base + "_sim.png"}})
        ev.append({"event_type": "DRY"})
        ev.append({"event_type": "SAVE", "rgba_canvas": {"filename": base + "_dry.png"}, "rgba_dry": {"filename": base + "_dry_dry.png"}})
        ev.append({"event_type": "BOOKMARK", "id": "fim_" + a.nome, "comment": "gravado " + base})
    print(json.dumps({"frames": [{"events": ev}]}))


if __name__ == "__main__":
    main()
