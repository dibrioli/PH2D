#!/usr/bin/env bash
# W18 — o Servo (nightly, ~/Apps/w18_motores/servo) a mostrar o embed do YouTube SEM ecrã, fotografado por
# WebDriver a cada 2 s: o tocador abre? o vídeo anda? Uso: sonda_servo.sh <id> <saida>
set -euo pipefail
VID="$1"; OUT="$(realpath -m "$2")"; mkdir -p "$OUT"
S=~/Apps/w18_motores/servo/servo/servoshell
W=$(mktemp -d); # O servoshell sem ecrã ignora o TERM: sem o -9 fica órfão a segurar o som (medido 12/10: 3 órfãos).
trap 'kill $SRV $SV 2>/dev/null; sleep 1; kill -9 $SV 2>/dev/null; rm -rf "$W"' EXIT
cat > "$W/index.html" <<H
<!doctype html><body style="margin:0;background:#000"><iframe width=560 height=315 frameborder=0 allow="autoplay; encrypted-media"
 referrerpolicy="strict-origin-when-cross-origin" src="https://www.youtube-nocookie.com/embed/$VID?autoplay=1&mute=1&playsinline=1"></iframe></body>
H
python3 -m http.server 18731 --bind 127.0.0.1 -d "$W" >/dev/null 2>&1 & SRV=$!
"$S" --headless --window-size=560x315 --webdriver=17000 "http://localhost:18731/" > "$OUT/servo_stdout.log" 2>&1 & SV=$!
for i in $(seq 1 30); do curl -s 127.0.0.1:17000/status >/dev/null 2>&1 && break; sleep 0.5; done
R=$(curl -s -XPOST 127.0.0.1:17000/session -H "content-type: application/json" -d "{\"capabilities\":{\"alwaysMatch\":{}}}"); echo "$R" > "$OUT/servo_sessao.json"; SID=$(echo "$R" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d[\"value\"][\"sessionId\"])")
curl -s -XPOST 127.0.0.1:17000/session/$SID/url -d '{"url":"http://localhost:18731/"}' >/dev/null
for k in $(seq 0 9); do sleep 2; curl -s 127.0.0.1:17000/session/$SID/screenshot | python3 -c "import sys,json,base64; open('$OUT/servo_$k.png','wb').write(base64.b64decode(json.load(sys.stdin)['value']))"; done
ps -o rss=,time= -p $SV > "$OUT/servo_custo.txt"; "$S" --version >> "$OUT/servo_custo.txt" 2>&1 || true
sha1sum "$OUT"/servo_*.png
