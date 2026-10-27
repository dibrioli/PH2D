#!/usr/bin/env python3
"""As 10 facas (E4): os overrides por faca sobre o Dabber de fábrica → cand/<slug>.ph2d-brush.png.
uso: facas.py <W> <ronda.json opcional: {"<slug>": {override parcial}}>"""
import json, subprocess, sys, os
W = sys.argv[1]
H = os.path.dirname(os.path.abspath(__file__))
BASE = f"{W}/assets/brushes/WetPainter/Oil/Dabber/Dabber.ph2d-brush.png"
TEX0 = {"angle_deg": 0, "flow": False, "kind": "None", "mapping": "Tiled", "offset": [0.0, 0.0],
        "params": [0.5] * 6, "rake": False, "size": [1.0, 1.0], "stencil_angle_deg": 0,
        "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}
def tex(kind, size, mapping="Tiled", params=None, angle=0):
    t = dict(TEX0, kind=kind, size=size, mapping=mapping, angle_deg=angle)
    if params: t["params"] = params
    return t
CARGA = {"angle_deg": 0, "flow": False, "kind": "Gradient", "mapping": "ViewPlane", "offset": [0.5, 0.5],
         "params": [0.5, 0.5, 0.0, 0.0, 0.5, 0.5], "rake": False, "size": [0.5, 0.5], "stencil_angle_deg": 0,
         "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}
COMUM_B = {"hardness": 1.0, "falloff": "Constant", "dab_quina": 1.0, "jitter": 0.0}
COMUM_E = {"paperGate": 0.0, "pressureOpacityCurve": 1.0, "levelingOwnStroke": 1.0}
# fixa = dabRotation 0 com o eixo longo do Rebelle (leis_lacunas lamina); segue = 1, a lâmina atravessada (90°).
F = {
 "smooth":   dict(nome="Knife Smooth", b=dict(dab_flatten=0.0, dab_angle_deg=0), e=dict(dabRotation=0, knifeRidge=3, pressureSize=0.2)),
 "smooth_2": dict(nome="Knife Smooth 2", b=dict(dab_flatten=0.8, dab_angle_deg=90), e=dict(dabRotation=1, knifeRidge=8, pressureSize=0.0)),
 "smooth_3": dict(nome="Knife Smooth 3", b=dict(dab_flatten=0.8, dab_angle_deg=90, texture=tex("Noise", [4.0, 40.0], params=[0.5]*6)), e=dict(dabRotation=1, knifeRidge=8, pressureSize=0.0)),
 "textured": dict(nome="Knife Textured", b=dict(dab_flatten=0.87, dab_angle_deg=86, texture=tex("Noise", [2.0, 60.0])), e=dict(dabRotation=0, knifeRidge=4, pressureSize=0.6)),
 "oily":     dict(nome="Knife Oily", b=dict(dab_flatten=0.69, dab_angle_deg=172, shape=CARGA), e=dict(dabRotation=0, knifeRidge=3, pressureSize=0.6)),
 "stain":    dict(nome="Knife Stain", b=dict(dab_flatten=0.8, dab_angle_deg=90, texture=tex("Noise", [5.0, 5.0])), e=dict(dabRotation=1, knifeRidge=0, paperGate=1.0, pressureSize=0.3)),
 "soft":     dict(nome="Knife Soft", b=dict(dab_flatten=0.84, dab_angle_deg=178, hardness=0.5, falloff="Smooth", texture=tex("Noise", [3.0, 30.0], angle=0)), e=dict(dabRotation=0, knifeRidge=2, pressureSize=0.6)),
 "grainy":   dict(nome="Knife Grainy", b=dict(dab_flatten=0.8, dab_angle_deg=90, jitter=0.1, shape=tex("Noise", [16.0, 16.0], mapping="ViewPlane", params=[0.6, 0.55, 0.3, 0.5, 0.5, 0.5])), e=dict(dabRotation=1, knifeRidge=2, paperGate=0.6, pressureSize=0.3)),
 "dry":      dict(nome="Knife Dry", b=dict(dab_flatten=0.74, dab_angle_deg=170), e=dict(dabRotation=0, knifeRidge=0, extDryBrush=0.6, pressureSize=0.6)),
 "dry_2":    dict(nome="Knife Dry 2", b=dict(dab_flatten=0.79, dab_angle_deg=10), e=dict(dabRotation=0, knifeRidge=0, extDryBrush=0.85, pressureSize=0.6)),
}
ronda = json.load(open(sys.argv[2])) if len(sys.argv) > 2 else {}
os.makedirs(f"{H}/cand", exist_ok=True)
for slug, f in F.items():
    b = dict(COMUM_B, **f["b"]); e = dict(COMUM_E, **f["e"])
    r = ronda.get(slug, {})
    b.update(r.get("b", {})); e.update(r.get("e", {}))
    ov = {"name": f["nome"], "group": "Knife", "engine": e, "brush": b}
    ov.update({k: v for k, v in r.items() if k not in ("b", "e")})
    j = f"{H}/cand/{slug}.json"
    json.dump(ov, open(j, "w"), indent=1)
    subprocess.run(["python3", "-I", f"{H}/E4_Knife_mk_preset.py", BASE, j, f"{H}/cand/{f['nome']}.ph2d-brush.png"], check=True)
