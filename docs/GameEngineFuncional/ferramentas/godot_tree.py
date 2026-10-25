"""Lê o dump `godot --doctool` e grava a árvore dos nós 2D e sem posição (JSON)."""
import json
import re
import sys
import xml.etree.ElementTree as ET
from collections import Counter
from pathlib import Path

root, out = Path(sys.argv[1]), Path(sys.argv[2])
TAG = re.compile(r"\[/?[a-z_]+[^\]]*\]")
classes = {}
for f in root.rglob("*.xml"):
    try:
        t = ET.parse(f).getroot()
    except ET.ParseError:
        continue
    if t.tag != "class":
        continue

    def names(tag, sub, t=t):
        e = t.find(tag)
        return [x.get("name") for x in e.findall(sub)] if e is not None else []

    brief = " ".join((t.findtext("brief_description") or "").split())
    classes[t.get("name")] = dict(
        name=t.get("name"),
        inherits=t.get("inherits"),
        deprecated=t.get("deprecated") is not None,
        experimental=t.get("experimental") is not None,
        brief=TAG.sub("", brief),
        properties=names("members", "member"),
        methods=names("methods", "method"),
        signals=names("signals", "signal"),
    )

kids = {}
for c in classes.values():
    kids.setdefault(c["inherits"], []).append(c["name"])


def desc(n):
    r = [n]
    for k in sorted(kids.get(n, [])):
        r += desc(k)
    return r


def chain(n):
    r = []
    while n:
        r.append(n)
        n = classes.get(n, {}).get("inherits")
    return r


skip = set(desc("Node3D")) | (set(desc("Viewport")) - {"Viewport", "Window", "SubViewport"})
rows = []
for n in desc("Node"):
    if n in skip or "3D" in n or n.startswith(("Editor", "OpenXR", "XR", "Script")):
        continue
    ch = chain(n)
    fam = next((f for f in ("Control", "Node2D", "CanvasItem") if f in ch), "Node")
    rows.append(dict(classes[n], family=fam, chain=" < ".join(ch)))
out.write_text(json.dumps(rows, ensure_ascii=False, indent=1))
print(len(rows), Counter(r["family"] for r in rows))
