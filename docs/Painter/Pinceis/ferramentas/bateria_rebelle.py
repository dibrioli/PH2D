#!/usr/bin/env python3
"""A BATERIA do EXAME (`bateria.json`) em eventos Motion IO do Rebelle 8, para UMA cena e UM preset.

Escreve `<saida>_A.json` (do NEW_ARTWORK até à foto A: traços + SIMULATION) e `<saida>_B.json` (da foto A
até à foto B: o DRY). Cada parte termina num BOOKMARK `<id>_A`/`<id>_B`, que o `ws_bateria.mjs` espera.

As leis herdadas de `NewFeaturesWetPaint/ferramentas/gera_eventos.py` (medidas 2026-10-07):
  · REPETE_PARAMS — o SET_BRUSH só vale para o traço seguinte: reenvia-se COMPLETO antes de CADA traço;
  · o tamanho vai em px (`size_px`) E em % do Max Size do preset: size = 100·(px/max)^(2/3)
    (size_px ≈ max·(s/100)^1,5), com o max lido de `rebelle_presets_281.json`; acima do max os dois vão ao
    max (size_px > max carimba quadrados soltos — medido 19/10 no Graphite 8B; o README lista os cortes).
Ops: pincel (preset|base) · cor · traco (pontos EXACTOS com a pressão) · seca = DRY · simula = SIMULATION ·
foto = corte da parte. A base da cena «sobre» é o próprio preset, salvo `--base TOOL:Grupo/Nome` (as
ferramentas sem tinta: Eraser, Blend, Smudge, Water, Dry, Clone, e os Marker/Effect).

A `bateria_lacunas.json` acrescenta (a `bateria.json` não os tem; a saída dela não muda): o traço
`"mesmo_pincel": true` vai SEM SET_BRUSH (a memória entre traços, L3), e `--params '{…}'` junta campos ao
SET_BRUSH do PRESET (ex.: `{"dual_brush_enabled": false}`; a base fica como é).

uso: bateria_rebelle.py <bateria.json> <cena> <TOOL> <Grupo/Nome> <saida> [--base TOOL:Grupo/Nome]
     [--seed 7] [--paper "Default/EX00 White Simple"] [--params JSON]
"""
import argparse
import json
import os
from pathlib import Path

AQUI = Path(__file__).resolve().parent
FX = AQUI.parent.parent / "NewFeaturesWetPaint" / "fixtures" / "rebelle_presets_281.json"
# A pasta de cada TOOL em `Brushes/` e `Brushes Bristle/` (os nomes do enum do Motion IO).
PASTA = {"WATERCOLOR": "Watercolor", "OIL_AND_ACRYLIC": "Oil", "EXPRESS_OIL": "ExpressOil", "PENCIL": "Pencil",
         "PASTEL": "Pastel", "INK_PEN": "Ink", "MARKER": "Marker", "AIRBRUSH": "Airbrush", "SMUDGE": "Smudge",
         "ERASER": "Eraser", "BLEND": "Blend", "WATER": "Wet", "DRY": "Dry", "CLONE": "Clone"}
_MAX = None


def max_do_preset(tool, preset):
    global _MAX
    if _MAX is None:
        _MAX = {}
        for p in json.loads(FX.read_text())["pinceis"]:
            partes = p["file"].split("/")
            chave = (partes[-3], partes[-2] + "/" + partes[-1].split(".rebelle-brush.png")[0])
            _MAX[chave] = float(p["Parameters"]["max_brush_size"])
    return _MAX.get((PASTA[tool], preset), 300.0)


def pct(size_px, mx):
    return round(min(100.0, 100.0 * (size_px / mx) ** (2.0 / 3.0)), 1)


def ptr(tipo, x, y, p):
    return {"event_type": tipo, "pos": {"x": x, "y": y}, "pen_tilt": {"x": 0, "y": 0}, "rotation": 0.0,
            "pressure": p}


def eventos(cena, tool, preset, base, seed, paper, ident, params=None):
    partes, ev = {}, []
    ev.append({"event_type": "NEW_ARTWORK", "width": 1000, "height": 640, "units": "px", "dpi": 72,
               "paper": {"preset": paper, "color": {"r": 255, "g": 255, "b": 255}, "deckled_edges": False,
                         "paper_scale": 100, "visible": True}})
    ev.append({"event_type": "SET_RANDOM_SEED", "seed": seed})
    quem = (tool, preset)
    cor = {"r": 0, "g": 0, "b": 0}
    ultimo = None
    for op in cena["ops"]:
        o = op["op"]
        if o == "pincel":
            quem = (tool, preset) if op["quem"] == "preset" or base is None else base
        elif o == "cor":
            cor = dict(zip("rgb", op["rgb"]))
        elif o == "traco":
            t, pr = quem
            mx = max_do_preset(t, pr)
            # Acima do Max Size o Rebelle não pinta maior: com size_px > max carimba QUADRADOS soltos
            # (medido 19/10, Graphite 8B a 200 px); o traço vai então ao max, com os dois campos coerentes.
            px = min(float(op["size_px"]), mx)
            if not op.get("mesmo_pincel"):  # bateria_lacunas: a memória entre traços (L3) — sem SET_BRUSH
                sb = {"event_type": "SET_BRUSH", "tool": t, "preset": pr, "size_px": px, "size": pct(px, mx),
                      "pressure": 100, "color": dict(cor)}
                if params and quem == (tool, preset):
                    sb.update(params)
                # PH2D_REBELLE_REPETE_TROCA=1 (J4 mistura): no 1.º traço depois de TROCAR de ferramenta o Rebelle pinta
                # ao tamanho da anterior (medido: o cruza_1 do Smudge Soft 68 px × 15 no cruza_2; Square Hard 75 × 39) —
                # o SET_BRUSH vai DUAS vezes na troca. Sem a variável, os eventos de sempre.
                if os.environ.get("PH2D_REBELLE_REPETE_TROCA") == "1" and ultimo is not None and ultimo != t:
                    ev.append(dict(sb))
                    ev.append({"event_type": "SET_BRUSH_COLOR", "color": dict(cor)})
                ev.append(sb)
                ev.append({"event_type": "SET_BRUSH_COLOR", "color": dict(cor)})
                ultimo = t
            pts = op["pts"]
            ev.append(ptr("POINTER_PRESS", *pts[0]))
            ev.extend(ptr("POINTER_MOVE", *q) for q in pts[1:])
            ev.append(ptr("POINTER_RELEASE", pts[-1][0], pts[-1][1], 0.0))
        elif o == "seca":
            ev.append({"event_type": "DRY"})
        elif o == "simula":
            ev.append({"event_type": "SIMULATION", "repeats": int(op["quadros"])})
        elif o == "foto":
            ev.append({"event_type": "BOOKMARK", "id": f"{ident}_{op['nome']}", "comment": "foto"})
            partes[op["nome"]] = ev
            ev = []
        else:
            raise SystemExit(f"op desconhecida: {o}")
    assert set(partes) == {"A", "B"} and not ev, "a cena tem de acabar em foto A … foto B"
    return partes


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("bateria"); ap.add_argument("cena"); ap.add_argument("tool"); ap.add_argument("preset")
    ap.add_argument("saida")
    ap.add_argument("--base", default=None, help="TOOL:Grupo/Nome do pincel das faixas (cena «sobre»)")
    ap.add_argument("--seed", type=int, default=7)
    ap.add_argument("--paper", default="Default/EX00 White Simple")
    ap.add_argument("--params", default="", help="JSON com campos a juntar ao SET_BRUSH do PRESET (não da base)")
    a = ap.parse_args()
    assert a.tool in PASTA, f"TOOL desconhecido: {a.tool}"
    bat = json.loads(Path(a.bateria).read_text())
    cena = next(c for c in bat["cenas"] if c["nome"] == a.cena)
    base = None
    if a.base:
        bt, bp = a.base.split(":", 1)
        assert bt in PASTA, f"TOOL da base desconhecido: {bt}"
        base = (bt, bp)
    ident = Path(a.saida).name
    params = json.loads(a.params) if a.params else None
    for nome, ev in eventos(cena, a.tool, a.preset, base, a.seed, a.paper, ident, params).items():
        Path(f"{a.saida}_{nome}.json").write_text(json.dumps({"frames": [{"events": ev}]}, separators=(",", ":")))


if __name__ == "__main__":
    main()
