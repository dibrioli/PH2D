#!/usr/bin/env bash
# Oráculo Godot (MIT, `pacman -Qi godot`): mede a taxonomia NÓ × RECURSO do Godot instalado.
# Corre o binário headless (`--dump-extension-api`), nunca lê o fonte. Saída = fixture com cabeçalho.
# Uso: bash docs/ObjectsAndComponents/ferramentas/godot_taxonomia.sh [saida.txt]
set -euo pipefail
out="${1:-/dev/stdout}"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
( cd "$tmp" && timeout 60 godot --headless --dump-extension-api >/dev/null 2>&1 )
test -s "$tmp/extension_api.json" || { echo "✗ godot não gerou extension_api.json" >&2; exit 2; }
python3 -I - "$tmp/extension_api.json" >"$out" <<'EOF'
import json, sys
api = json.load(open(sys.argv[1]))
cls = {c["name"]: c for c in api["classes"]}
def anc(n):
    out = []
    while n in cls and cls[n].get("inherits"):
        n = cls[n]["inherits"]; out.append(n)
    return out
def subs(root):
    return sorted(n for n in cls if root in anc(n))
print(f"# Oráculo: {api['header']['version_full_name']} — taxonomia medida por --dump-extension-api")
print(f"classes {len(cls)}")
for root in ["Node", "Node2D", "CanvasItem", "Control", "Resource"]:
    print(f"{root}_descendentes {len(subs(root))}")
assert len(subs("Node2D")) > 0, "controlo positivo: Node2D sem descendentes = API mal lida"
print("\n## Node2D (o que vive na árvore e tem posição 2D)")
print(", ".join(subs("Node2D")))
editor = lambda n: (n.startswith("Editor") or n.startswith("OpenXR") or "3D" in n
                    or "Node3D" in anc(n) or n.endswith("EditorPlugin"))
plain = [n for n in subs("Node") if "CanvasItem" not in anc(n) and n != "CanvasItem" and not editor(n)
         and "Viewport" not in anc(n) and n != "Viewport"]
print("\n## Node sem desenho (não é CanvasItem) — o 'componente-como-nó'")
print(", ".join(plain))
print("\n## Recursos que um jogo 2D toca (dados no Inspector / FileSystem, não na árvore)")
for root in ["Animation", "AnimationLibrary", "SpriteFrames", "Shape2D", "Texture2D", "AudioStream",
             "Curve", "Gradient", "PhysicsMaterial", "TileSet", "PackedScene", "Material", "Theme"]:
    if root in cls:
        s = subs(root)
        print(f"{root} (+{len(s)} subclasses){': ' + ', '.join(s[:12]) if s else ''}")
EOF
