#!/usr/bin/env bash
# Instala o oráculo da COLABORAÇÃO do Excalidraw FORA do repo ($COLAB_HOME, padrão
# ~/Apps/excalidraw-colab-oracle): o app do excalidraw.com (monorepo, excalidraw-app/) e o servidor de
# salas excalidraw-room, cada um num commit FIXO; yarn 1.22.22 local; build de produção do app com
# TODOS os serviços externos apontados para 127.0.0.1 (a sala na 3102). Idempotente.
# Nunca se lê o fonte (CLAUDE.md §0.9): só LICENSE, package.json, .env*, README e config de build.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COLAB_HOME="${COLAB_HOME:-$HOME/Apps/excalidraw-colab-oracle}"
EXCALIDRAW_COMMIT="4c00f31ddcc20086bac1020428819fe0b9a30bce"   # master 2026-10-08
ROOM_COMMIT="03ff435860b508d7cd9e005cfc90f7977ae2a593"         # master (último commit 2023-12-15)
ORACLE_PW="${ORACLE_PW:-$HOME/Apps/excalidraw-oracle}"           # playwright 1.63.0 já instalado

mkdir -p "$COLAB_HOME"
cd "$COLAB_HOME"

clone_at() { # url dir commit
  if [[ ! -d "$2/.git" ]]; then timeout 600 git clone --filter=blob:none "$1" "$2"; fi
  (cd "$2" && git fetch -q origin "$3" 2>/dev/null || true; git checkout -q --detach "$3")
  [[ "$(cd "$2" && git rev-parse HEAD)" == "$3" ]] || { echo "commit errado em $2" >&2; exit 1; }
}
clone_at https://github.com/excalidraw/excalidraw.git excalidraw "$EXCALIDRAW_COMMIT"
clone_at https://github.com/excalidraw/excalidraw-room.git excalidraw-room "$ROOM_COMMIT"

if [[ ! -x tools/node_modules/.bin/yarn ]]; then
  timeout 300 npm install --prefix tools --no-audit --no-fund yarn@1.22.22
fi
YARN="$COLAB_HOME/tools/node_modules/.bin/yarn"

# Servidor de salas (socket.io): PORT e CORS_ORIGIN são as variáveis do .env.development dele.
(cd excalidraw-room && timeout 600 "$YARN" install --frozen-lockfile --ignore-engines --non-interactive \
  && timeout 300 "$YARN" build)

# App: build de produção; as variáveis do processo têm prioridade sobre os .env* do monorepo.
(cd excalidraw && timeout 900 "$YARN" install --frozen-lockfile --ignore-engines --non-interactive --ignore-scripts)
OFF="http://127.0.0.1:9"   # porta fechada: qualquer tentativa falha localmente (e o Playwright ainda a bloqueia)
(cd excalidraw/excalidraw-app && \
  VITE_APP_WS_SERVER_URL="http://127.0.0.1:3102" \
  VITE_APP_BACKEND_V2_GET_URL="$OFF/api/v2/" VITE_APP_BACKEND_V2_POST_URL="$OFF/api/v2/post/" \
  VITE_APP_LIBRARY_URL="$OFF" VITE_APP_LIBRARY_BACKEND="$OFF" \
  VITE_APP_PLUS_LP="$OFF" VITE_APP_PLUS_APP="$OFF" VITE_APP_AI_BACKEND="$OFF" \
  VITE_APP_FIREBASE_CONFIG='{"apiKey":"oracle-offline","authDomain":"127.0.0.1","projectId":"ph2d-oracle-offline","storageBucket":"ph2d-oracle-offline.invalid","messagingSenderId":"0","appId":"1:0:web:0"}' \
  VITE_APP_DISABLE_SENTRY=true VITE_APP_ENABLE_TRACKING=false VITE_APP_ENABLE_ESLINT=false \
  VITE_APP_ENABLE_PWA=false VITE_APP_DISABLE_PREVENT_UNLOAD=true \
  timeout 1200 node ../node_modules/vite/bin/vite.js build --mode production)

# Harness em $COLAB_HOME/harness (subpasta: um node_modules na raiz seria visto pelo tsc da sala):
# package.json deste directório (só playwright, versão exacta) — reusa o node_modules do oráculo de
# desenho se lá estiver o 1.63.0.
mkdir -p "$COLAB_HOME/harness"; cp -f "$HERE/package.json" "$COLAB_HOME/harness/package.json"
if [[ -f "$ORACLE_PW/node_modules/playwright/package.json" ]] && \
   [[ "$(node -p "require('$ORACLE_PW/node_modules/playwright/package.json').version")" == "1.63.0" ]]; then
  ln -sfn "$ORACLE_PW/node_modules" "$COLAB_HOME/harness/node_modules"
else
  (cd "$COLAB_HOME/harness" && timeout 600 npm install --no-audit --no-fund --save-exact && timeout 900 npx playwright install chromium)
fi

node "$HERE/licencas.mjs" "$COLAB_HOME"
du -sh "$COLAB_HOME" 2>/dev/null || true
echo "OK: oráculo da colaboração em $COLAB_HOME"
