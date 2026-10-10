#!/usr/bin/env bash
# W9 (fidelidade): instala o canal `next` do Excalidraw (o do excalidraw.com; versão fixada em
# next/package.json + lock) FORA do repo, em $ORACLE_NEXT_HOME (padrão ~/Apps/excalidraw-oracle-next),
# e constrói a página independente da versão (src/oracle_w9.js) em `www_w9/` das DUAS instalações
# (a 0.18.1 do instala.sh e a next). Requer instala.sh corrido antes (Chromium do Playwright).
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OLD="${ORACLE_HOME:-$HOME/Apps/excalidraw-oracle}"
NEXT="${ORACLE_NEXT_HOME:-$HOME/Apps/excalidraw-oracle-next}"
export PATH="$HOME/.local/bin:$PATH"

mkdir -p "$NEXT"
cp -f "$HERE/next/package.json" "$HERE/next/package-lock.json" "$NEXT/"
(cd "$NEXT" && timeout 600 npm ci --no-audit --no-fund)

for H in "$OLD" "$NEXT"; do
  rm -rf "$H/www_w9" "$H/src_w9"
  mkdir -p "$H/www_w9" "$H/src_w9"
  cp -f "$HERE/src/oracle_w9.js" "$H/src_w9/oracle.js"
  cp -f "$HERE/src/index.html" "$H/www_w9/index.html"
  (cd "$H" && timeout 300 npx esbuild src_w9/oracle.js \
    --bundle --format=esm --splitting --outdir=www_w9 --minify \
    --define:process.env.NODE_ENV='"production"' \
    --loader:.woff2=file --loader:.ttf=file --loader:.wasm=file --loader:.css=empty \
    --log-level=warning)
  cp -f "$H/node_modules/@excalidraw/excalidraw/dist/prod/index.css" "$H/www_w9/index.css"
  [[ -d "$H/node_modules/@excalidraw/excalidraw/dist/prod/fonts" ]] && cp -r "$H/node_modules/@excalidraw/excalidraw/dist/prod/fonts" "$H/www_w9/fonts"
  node -e 'const p=require(process.argv[1]+"/node_modules/@excalidraw/excalidraw/package.json"); console.log(process.argv[1], p.version, p.license)' "$H"
done
echo "OK: páginas W9 em $OLD/www_w9 e $NEXT/www_w9"
