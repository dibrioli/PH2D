# Pinceladas-sonda por WebSocket (JSON por linha). uso: tracos.py > t.jsonl
import json, math
E = [{"event_type": "NEW_ARTWORK", "width": 1200, "height": 800, "units": "px", "dpi": 72,
      "paper": {"preset": "Default/EX00 White Simple", "color": {"r": 255, "g": 255, "b": 255}, "deckled_edges": False, "paper_scale": 100, "visible": True}},
     {"event_type": "SET_RANDOM_SEED", "seed": 11},
     {"event_type": "SET_ENGINE_PARAMS", "realshader": True, "impasto_depth": 8, "gloss": 10, "paper_texture": 0}]
AZ = {"r": 20, "g": 150, "b": 230}; BR = {"r": 240, "g": 240, "b": 236}; ES = {"r": 30, "g": 25, "b": 60}
def p(t, x, y, pr): return {"event_type": t, "pos": {"x": round(x, 2), "y": round(y, 2)}, "pen_tilt": {"x": 0, "y": 0}, "rotation": 0.0, "pressure": pr}
def traco(preset, cor, pts, size, passo=3):
    E.append({"event_type": "SET_BRUSH", "tool": "OIL_AND_ACRYLIC", "preset": preset, "size_px": size, "pressure": 100, "color": cor})
    E.append(p("POINTER_PRESS", *pts[0], 1.0))
    for a, b in zip(pts, pts[1:]):
        n = max(1, math.ceil(math.dist(a, b) / passo))
        for k in range(1, n + 1):
            f = k / n; E.append(p("POINTER_MOVE", a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f, 1.0))
    E.append(p("POINTER_RELEASE", *pts[-1], 0.0))
# 1 espátula lisa horizontal (degrau em cima e em baixo) · 2 vertical · 3 bristle sulcos · 4 cruz (sombra de uma crista sobre outra)
traco("Knife/Knife Smooth", AZ, [(80, 110), (560, 110)], 120)
traco("Knife/Knife Smooth", AZ, [(700, 60), (700, 420)], 120)
traco("Flat/Flat Oily", AZ, [(850, 100), (1150, 100)], 100)
traco("Flat/Thick", AZ, [(850, 260), (1150, 260)], 100)
traco("Knife/Knife Smooth", BR, [(80, 300), (560, 300)], 120)
traco("Knife/Knife Smooth", ES, [(80, 480), (560, 480)], 120)
# cúpula: espiral curta e grossa
pts = [(320 + 30 * math.cos(t / 6) * (1 - t / 80), 660 + 30 * math.sin(t / 6) * (1 - t / 80)) for t in range(80)]
traco("Flat/Thick", AZ, pts, 70, 2)
traco("Knife/Knife Textured", AZ, [(700, 560), (1150, 560)], 120)
traco("Flat/Flat Oily", ES, [(850, 720), (1150, 720)], 100)
for e in E: print(json.dumps(e))
