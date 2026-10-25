# Gera as partes de eventos das cenas `dentro` e `sobre` (Watercolor/Liner, mesma receita da bateria `sobre`).
import json, sys
from pathlib import Path
OUT = Path(sys.argv[1])
PRE = "Watercolor/Liner"; SZ = {60.0: 34.2, 40.0: 26.1}
VERM = {"r": 191, "g": 0, "b": 18}; AZUL = {"r": 23, "g": 101, "b": 179}; CAST = {"r": 60, "g": 36, "b": 20}
def ptr(t, x, y, p): return {"event_type": t, "pos": {"x": x, "y": y}, "pen_tilt": {"x": 0, "y": 0}, "rotation": 0.0, "pressure": p}
def traco(a, b, px, cor, passo=3.0):
    (x0, y0), (x1, y1) = a, b
    L = ((x1-x0)**2 + (y1-y0)**2) ** .5; n = max(1, round(L / passo))
    pts = [(round(x0+(x1-x0)*i/n, 2), round(y0+(y1-y0)*i/n, 2)) for i in range(n+1)]
    ev = [{"event_type": "SET_BRUSH", "tool": "WATERCOLOR", "preset": PRE, "size_px": px, "size": SZ[px], "pressure": 100, "color": dict(cor)},
          {"event_type": "SET_BRUSH_COLOR", "color": dict(cor)}, ptr("POINTER_PRESS", *pts[0], 1.0)]
    ev += [ptr("POINTER_MOVE", x, y, 1.0) for x, y in pts[1:]]
    ev.append(ptr("POINTER_RELEASE", *pts[-1], 0.0)); return ev
def novo(): return [{"event_type": "NEW_ARTWORK", "width": 1000, "height": 640, "units": "px", "dpi": 72,
    "paper": {"preset": "Default/EX00 White Simple", "color": {"r": 255, "g": 255, "b": 255}, "deckled_edges": False, "paper_scale": 100, "visible": True}},
    {"event_type": "SET_RANDOM_SEED", "seed": 7}]
def bm(i): return [{"event_type": "BOOKMARK", "id": i, "comment": "foto"}]
def w(nome, ev): (OUT / f"{nome}.json").write_text(json.dumps({"frames": [{"events": ev}]}, separators=(",", ":")))
# comum: 0 = documento novo; S = SIMULATION 240; D = DRY
for c in ("dentro", "sobre"):
    w(f"{c}_0", novo() + bm(f"{c}_0"))
    w(f"{c}_S", [{"event_type": "SIMULATION", "repeats": 240}] + bm(f"{c}_S"))
    w(f"{c}_D", [{"event_type": "DRY"}] + bm(f"{c}_D"))
# dentro: faixa vermelha 60 px de (300,320) a (700,320); traço castanho 40 px de (433,330) a (850,330)
w("dentro_1", traco((300, 320), (700, 320), 60.0, VERM) + traco((433, 330), (850, 330), 40.0, CAST) + bm("dentro_1"))
# sobre: a receita da bateria (faixa seca y=470 + SIM 240 + DRY; faixa molhada y=150; 3 azuis verticais; azul no papel y=310)
s = traco((60, 470), (940, 470), 60.0, VERM) + [{"event_type": "SIMULATION", "repeats": 240}, {"event_type": "DRY"}]
s += traco((60, 150), (940, 150), 60.0, VERM)
s += traco((250, 60), (250, 580), 40.0, AZUL) + traco((500, 60), (500, 580), 40.0, AZUL) + traco((750, 60), (750, 580), 40.0, AZUL, 25.0)
s += traco((60, 310), (940, 310), 40.0, AZUL)
w("sobre_1", s + bm("sobre_1"))
print("ok")
