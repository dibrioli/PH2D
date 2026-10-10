#!/usr/bin/env bash
# corre_tudo.sh — DUAS corridas completas dos 23 casos (servidores próprios, parados pelo PID no fim),
# depois compara.mjs junta-as em ../saidas/excalidraw/<caso>.json + meta.json.
# Pré-requisito: instala.sh (1× por máquina).
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COLAB_HOME="${COLAB_HOME:-$HOME/Apps/excalidraw-colab-oracle}"
SAIDA="$HERE/../saidas/excalidraw"
STAMP="$(date +%Y%m%d-%H%M%S)"; RUNS="$COLAB_HOME/corridas/$STAMP"; mkdir -p "$RUNS"
bash "$HERE/servidores.sh" stop >/dev/null 2>&1 || true
bash "$HERE/servidores.sh" start
trap 'bash "$HERE/servidores.sh" stop' EXIT
for i in 1 2; do
  echo "=== corrida $i ($(cut -d' ' -f1-3 /proc/loadavg))"
  PRAZO_CORRIDA=2400 bash "$HERE/corre.sh" "$RUNS/$i"
done
X="$COLAB_HOME/excalidraw"; R="$COLAB_HOME/excalidraw-room"
node -e '
const fs = require("fs"), cp = require("child_process"), [X, R, H, here] = process.argv.slice(1);
const v = (p) => { try { return JSON.parse(fs.readFileSync(p, "utf8")).version; } catch { return null; } };
const sh = (c, d) => cp.execSync(c, { cwd: d }).toString().trim();
const lic = JSON.parse(cp.execSync(`node ${here}/licencas.mjs ${H} --json`).toString());
fs.writeFileSync(process.argv[5] || "/dev/stdout", JSON.stringify({
  excalidraw: { repo: "https://github.com/excalidraw/excalidraw", commit: sh("git rev-parse HEAD", X), data_do_commit: sh("git log -1 --format=%ci", X), app: "excalidraw-app/ (build de produção, vite)", pacote_editor: v(X + "/packages/excalidraw/package.json") },
  excalidraw_room: { repo: "https://github.com/excalidraw/excalidraw-room", commit: sh("git rev-parse HEAD", R), data_do_commit: sh("git log -1 --format=%ci", R) },
  versoes: { node: process.version, playwright: v(H + "/harness/node_modules/playwright/package.json"), "socket.io-client (app)": v(X + "/excalidraw-app/node_modules/socket.io-client/package.json") ?? v(X + "/node_modules/socket.io-client/package.json"), "socket.io (sala)": v(R + "/node_modules/socket.io/package.json"), yarn: v(H + "/tools/node_modules/yarn/package.json") },
  local: { app: "http://127.0.0.1:5101 (serve_app.mjs sobre o build)", sala: "127.0.0.1:3102 (node -r so_local.cjs dist/index.js, PORT=3102, CORS_ORIGIN=http://127.0.0.1:5101)",
    env_do_build: "VITE_APP_WS_SERVER_URL=http://127.0.0.1:3102; BACKEND_V2/LIBRARY/PLUS/AI = http://127.0.0.1:9 (porta fechada); VITE_APP_FIREBASE_CONFIG = projecto fictício ph2d-oracle-offline; SENTRY off, TRACKING off, PWA off",
    rede: "context.route: tudo o que não é 127.0.0.1/localhost é abortado e registado; as fontes do CDN são servidas do próprio build local; serviceWorkers: block" },
  licencas: lic,
  como_reproduzir: ["cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone", "bash docs/MiroClone/ferramentas/colab_oracle/excalidraw_colab/instala.sh", "bash docs/MiroClone/ferramentas/colab_oracle/excalidraw_colab/corre_tudo.sh"],
}, null, 1));
' "$X" "$R" "$COLAB_HOME" "$HERE" "$RUNS/meta_extra.json"
node "$HERE/compara.mjs" "$RUNS/1" "$RUNS/2" "$SAIDA" "$RUNS/meta_extra.json"
echo "corridas brutas: $RUNS"
