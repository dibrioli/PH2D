#!/usr/bin/env bash
# Refaz as fotos das maquetes A·B·C da secção InputMap da crítica do dono, em tiras de 1800 px.
# Uso: bash docs/InputMap/ferramentas/fotografa_maquetes.sh <pasta_de_saida>
# A crítica é do dono e vive fora do repo; a cópia versionada das tiras está em docs/InputMap/maquetes/.
set -euo pipefail
SAIDA="${1:?pasta de saída}"
CRITICA_DIR="/home/enio/Documentos/GameEngineSamples"
CRITICA="$CRITICA_DIR/critica_ph2d.html"
test -f "$CRITICA" || { echo "✗ não achei $CRITICA" >&2; exit 2; }
mkdir -p "$SAIDA"
TMP_HTML="$CRITICA_DIR/.inputmap_only_$$.html"   # tem de morar ao lado da crítica: as imagens são relativas
trap 'rm -f "$TMP_HTML"' EXIT
python3 - "$CRITICA" "$TMP_HTML" <<'EOF'
import sys
s = open(sys.argv[1], encoding="utf-8").read()
k = s.find('id="inputmap"'); assert k > 0, "secção inputmap ausente"
i = s.rfind("<section", 0, k + 20); assert i > 0
j = s.find("<section", k + 10); assert j > k
head = s[: s.find("<body")]
open(sys.argv[2], "w", encoding="utf-8").write(
    head + '<body><main style="max-width:1400px;margin:0 auto">' + s[i:j] + "</main></body></html>")
EOF
timeout 90 google-chrome-stable --headless=new --disable-gpu --hide-scrollbars \
  --window-size=1400,20000 --screenshot="$SAIDA/inteira.png" "file://$TMP_HTML" >/dev/null 2>&1
python3 - "$SAIDA" <<'EOF'
import sys
from PIL import Image
import numpy as np
d = sys.argv[1]
im = Image.open(f"{d}/inteira.png").convert("RGB")
a = np.array(im.convert("L"))
rows = np.where(a.std(axis=1) > 3)[0]; assert len(rows), "foto vazia"
H = int(rows.max()) + 20
n = 0
for k, y in enumerate(range(0, H, 1800)):
    im.crop((0, y, 1400, min(H, y + 1800))).save(f"{d}/tira_{k:02d}.png"); n += 1
print(f"✓ {n} tiras em {d} (altura {H})")
EOF
