import sys, os, json, collections
root = sys.argv[1]
curvas = collections.defaultdict(lambda: collections.Counter()); ids = collections.Counter()
def walk(o, p=""):
    if isinstance(o, dict):
        if "points" in o and "fromParams" in o:
            pts = tuple((round(q["x"],2), round(q["y"],2)) for q in o["points"])
            curvas[p][(o.get("function"), o.get("fromParams"), pts)] += 1
            return
        for k, v in o.items(): walk(v, f"{p}.{k}" if p else k)
    elif isinstance(o, list):
        for i, v in enumerate(o): walk(v, f"{p}[{i}]")
for dp, dn, fn in os.walk(root):
    for f in fn:
        r = json.load(open(os.path.join(dp, f))); walk(r["Parameters"])
for k in sorted(curvas):
    if "background_texture_params[" in k and "[0]" not in k: continue
    c = curvas[k]; tot = sum(c.values())
    print(f"\n{k}  (n={tot}, distintas={len(c)})")
    for (fn_, fp, pts), n in c.most_common(4):
        print(f"   {n:3d}× fn={fn_} fromParams={fp} pts={list(pts)}")
