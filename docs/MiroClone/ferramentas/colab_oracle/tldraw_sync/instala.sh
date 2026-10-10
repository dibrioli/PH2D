#!/usr/bin/env bash
# Instala o oráculo tldraw-sync FORA do repo (em $ORACLE_HOME, padrão ~/Apps/tldraw-sync-oracle):
# node_modules com versões exactas (package.json + package-lock.json versionados aqui), o Chromium do
# Playwright (~/.cache/ms-playwright, partilhado) e a página (www/: bundle esbuild de src/).
# Idempotente: correr de novo reconstrói www/ a partir de src/ deste directório.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ORACLE_HOME="${ORACLE_HOME:-$HOME/Apps/tldraw-sync-oracle}"

mkdir -p "$ORACLE_HOME"
cp -f "$HERE/package.json" "$ORACLE_HOME/package.json"
cp -f "$HERE/package-lock.json" "$ORACLE_HOME/package-lock.json"
(cd "$ORACLE_HOME" && timeout 600 npm ci --no-audit --no-fund)
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
cp -f "$ORACLE_HOME/node_modules/tldraw/tldraw.css" "$ORACLE_HOME/www/tldraw.css"
for d in embed-icons fonts icons translations; do cp -r "$ORACLE_HOME/node_modules/@tldraw/assets/$d" "$ORACLE_HOME/www/$d"; done

echo "--- licença dos pacotes instalados (campo license do package.json) ---"
(cd "$ORACLE_HOME" && node "$HERE/../../tldraw_oracle/licencas.mjs" | awk -F'\t' '{print $3}' | sort | uniq -c)
echo "--- LICENSE.md do tldraw instalado aponta para: ---"
cat "$ORACLE_HOME/node_modules/tldraw/LICENSE.md"
du -sh "$ORACLE_HOME" 2>/dev/null || true
echo "OK: oráculo em $ORACLE_HOME"
