import sys, os, json, collections
root = sys.argv[1]
vals = collections.defaultdict(list); ids = collections.Counter(); outros = collections.Counter()
def flat(o, p=""):
    if isinstance(o, dict):
        for k, v in o.items(): yield from flat(v, f"{p}.{k}" if p else k)
    elif isinstance(o, list):
        if o and all(isinstance(x, dict) for x in o) and p.endswith("points"):
            yield p, "pts[%d]" % len(o)
        else:
            for i, v in enumerate(o): yield from flat(v, f"{p}[{i}]")
    else:
        yield p, o
for dp, dn, fn in os.walk(root):
    for f in fn:
        r = json.load(open(os.path.join(dp, f)))
        ids[r["BrushID"]] += 1
        for k in r["outros"]: outros[k] += 1
        for k, v in flat(r["Parameters"] or {}, "P"): vals[k].append(v)
        for k, v in flat(r["ParametersEx"] or {}, "X"): vals[k].append(v)
print("BrushIDs:", dict(ids)); print("outros chunks:", dict(outros))
import re
for k in sorted(vals):
    v = vals[k]; n = len(v)
    if all(isinstance(x, bool) for x in v):
        print(f"{k:90s} bool  true={sum(v)}/{n}")
    elif all(isinstance(x, (int, float)) and not isinstance(x, bool) for x in v):
        print(f"{k:90s} num   min={min(v):g} max={max(v):g} distintos={len(set(v))} n={n}")
    else:
        c = collections.Counter(map(str, v)); print(f"{k:90s} enum  {dict(c.most_common(8))} n={n}")
