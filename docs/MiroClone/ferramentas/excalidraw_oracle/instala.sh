#!/usr/bin/env bash
# Instala o oráculo Excalidraw FORA do repo (em $ORACLE_HOME, padrão ~/Apps/excalidraw-oracle):
# node_modules com versões exactas (package.json + package-lock.json versionados aqui), Chromium do
# Playwright (~/.cache/ms-playwright) e a página servida localmente (www/: bundle esbuild + fontes).
# Idempotente: correr de novo reconstrói www/ a partir de src/ deste directório.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ORACLE_HOME="${ORACLE_HOME:-$HOME/Apps/excalidraw-oracle}"
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

# Chromium do Playwright (headless shell incluído).
(cd "$ORACLE_HOME" && timeout 900 npx playwright install chromium)

# Página: bundle do src/oracle.js + fontes self-hosted (EXCALIDRAW_ASSET_PATH="/").
# src/ é copiado para lá: o esbuild resolve "@excalidraw/excalidraw" a partir da pasta do ficheiro.
rm -rf "$ORACLE_HOME/www" "$ORACLE_HOME/src"
mkdir -p "$ORACLE_HOME/www"
cp -r "$HERE/src" "$ORACLE_HOME/src"
cp -f "$HERE/src/index.html" "$ORACLE_HOME/www/index.html"
(cd "$ORACLE_HOME" && timeout 300 npx esbuild src/oracle.js \
  --bundle --format=esm --splitting --outdir=www --minify \
  --define:process.env.NODE_ENV='"production"' \
  --loader:.woff2=file --loader:.ttf=file --loader:.wasm=file --loader:.css=empty \
  --log-level=warning)
cp -f "$ORACLE_HOME/node_modules/@excalidraw/excalidraw/dist/prod/index.css" "$ORACLE_HOME/www/index.css"
FONTS_SRC="$ORACLE_HOME/node_modules/@excalidraw/excalidraw/dist/prod/fonts"
if [[ -d "$FONTS_SRC" ]]; then
  cp -r "$FONTS_SRC" "$ORACLE_HOME/www/fonts"
else
  echo "AVISO: $FONTS_SRC não existe — as fontes não serão servidas" >&2
fi

echo "--- licença do pacote instalado ---"
node -e 'for (const p of ["@excalidraw/excalidraw","roughjs","perfect-freehand","react","react-dom","playwright","esbuild"]) { try { const j=require(require("path").join(process.argv[1],"node_modules",p,"package.json")); console.log(p, j.version, j.license); } catch(e) { console.log(p, "AUSENTE") } }' "$ORACLE_HOME"
du -sh "$ORACLE_HOME" "$HOME/.cache/ms-playwright" 2>/dev/null || true
echo "OK: oráculo em $ORACLE_HOME"
