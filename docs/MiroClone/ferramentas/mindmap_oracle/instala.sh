#!/usr/bin/env bash
# Instala o oráculo de mapa mental FORA do repo (em $ORACLE_HOME, padrão ~/Apps/mindmap-oracle):
# node_modules com versões exactas (package.json + package-lock.json versionados aqui), o Chromium do
# Playwright (~/.cache/ms-playwright, partilhado com os oráculos tldraw/Excalidraw) e a página (www/:
# bundle esbuild das três bibliotecas pelos pontos de entrada públicos + o CSS que o README manda usar).
# Idempotente: correr de novo reconstrói www/ a partir de src/ deste directório.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ORACLE_HOME="${ORACLE_HOME:-$HOME/Apps/mindmap-oracle}"
export PATH="$HOME/.local/bin:$PATH"

mkdir -p "$ORACLE_HOME"
cp -f "$HERE/package.json" "$ORACLE_HOME/package.json"
if [[ -f "$HERE/package-lock.json" ]]; then
  cp -f "$HERE/package-lock.json" "$ORACLE_HOME/package-lock.json"
  (cd "$ORACLE_HOME" && timeout 600 npm ci --no-audit --no-fund)
else
  (cd "$ORACLE_HOME" && timeout 600 npm install --no-audit --no-fund --save-exact)
  cp -f "$ORACLE_HOME/package-lock.json" "$HERE/package-lock.json"
fi

(cd "$ORACLE_HOME" && timeout 900 npx playwright install chromium)

rm -rf "$ORACLE_HOME/www" "$ORACLE_HOME/src"
mkdir -p "$ORACLE_HOME/www"
cp -r "$HERE/src" "$ORACLE_HOME/src"
cp -f "$HERE/src/index.html" "$ORACLE_HOME/www/index.html"
(cd "$ORACLE_HOME" && timeout 300 npx esbuild src/oracle.js \
  --bundle --format=esm --outdir=www --minify \
  --define:process.env.NODE_ENV='"production"' \
  --loader:.woff2=file --loader:.woff=file --loader:.ttf=file --loader:.svg=file --loader:.css=empty \
  --log-level=warning)
cp -f "$ORACLE_HOME/node_modules/mind-elixir/dist/MindElixir.css" "$ORACLE_HOME/www/mind-elixir.css"
cp -f "$ORACLE_HOME/node_modules/simple-mind-map/dist/simpleMindMap.esm.css" "$ORACLE_HOME/www/simple-mind-map.css"

echo "--- licença dos pacotes instalados (package.json + ficheiro LICENSE* do pacote) ---"
(cd "$ORACLE_HOME" && node "$HERE/licencas.mjs" --resumo)
du -sh "$ORACLE_HOME" "$HOME/.cache/ms-playwright" 2>/dev/null || true
echo "OK: oráculo em $ORACLE_HOME"
