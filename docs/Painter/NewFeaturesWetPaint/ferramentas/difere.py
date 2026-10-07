import sys, json
sys.path.insert(0, sys.argv[0].rsplit('/',1)[0])
def comp(o):
    if isinstance(o, dict):
        if "points" in o and "fromParams" in o:
            return "curve" + ("[P]" if o.get("fromParams") else "") + str([(round(p["x"],2), round(p["y"],2)) for p in o["points"]])
        return {k: comp(v) for k, v in o.items()}
    if isinstance(o, list): return [comp(x) for x in o]
    return o
def flat(o, p=""):
    if isinstance(o, dict):
        for k, v in o.items(): yield from flat(v, f"{p}.{k}" if p else k)
    elif isinstance(o, list):
        for i, v in enumerate(o): yield from flat(v, f"{p}[{i}]")
    else: yield p, o
base = json.load(open(sys.argv[1])); B = dict(flat(comp(base["Parameters"]))); B.update(("X."+k, v) for k, v in flat(base["ParametersEx"] or {}))
for f in sys.argv[2:]:
    r = json.load(open(f)); P = dict(flat(comp(r["Parameters"]))); P.update(("X."+k, v) for k, v in flat(r["ParametersEx"] or {}))
    print(f"\n===== {r['file'].replace('.rebelle-brush.png','')} | {r['BrushID'].replace('AQUARELLE_','')} | tex {r['Textures']}")
    bg_on = r["Parameters"].get("background_texture_params", [])
    for k in sorted(set(P) | set(B)):
        if k.startswith("background_texture_params["):
            idx = int(k.split("[")[1].split("]")[0])
            if idx >= len(bg_on) or not bg_on[idx].get("enabled"): continue
        if k.startswith("dual_brush") and not P.get("dual_brush_enabled"): continue
        if k.startswith("bristle_noise_st"): continue
        if k.startswith("meta") or k.startswith("X.volume_params.length") or k.startswith("X.volume_params.enabled"): continue
        if P.get(k) != B.get(k):
            v = P.get(k, "∅"); print(f"  {k} = {v}")
