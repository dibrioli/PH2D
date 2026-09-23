#!/usr/bin/env bash
# setup.sh — monta a pasta vendor/ do oráculo GDevelop Health a partir do ZERO.
#
# Tudo o que aqui se descarrega/extrai é de TERCEIROS e fica fora do git
# (vendor/.gitignore). Idempotente: corre-se outra vez e só refaz o que falta.
#
#   bash docs/Components/ferramentas/gdevelop_health/setup.sh
#
# Proveniência (fixada):
#   · libGD.js/.wasm e o GDJS Runtime ....... GDevelop 5.6.282 INSTALADO (AUR gdevelop-bin,
#                                              /usr/lib/gdevelop), MIT
#   · o carregador de extensões do IDE ...... npm gdcore-tools@2.0.0-gd-v5.6.269-autobuild
#                                              (arthuro555, MIT) — só usamos o dist/loaders.cjs,
#                                              que é o código do próprio IDE (newIDE
#                                              EventsFunctionsExtensionsLoader + LocalFileSystem)
#                                              empacotado; libGD e Runtime dele NÃO são usados.
#   · a extensão Health ..................... GDevelopApp/GDevelop-extensions @ COMMIT abaixo, MIT
#   · puppeteer-core ........................ npm, versão abaixo (só o cliente CDP; o browser é
#                                              o google-chrome-stable do sistema)
set -euo pipefail

AQUI="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
V="$AQUI/vendor"
GD_APP=/usr/lib/gdevelop
GD_VERSION_ESPERADA=5.6.282
EXT_COMMIT=3f71acc8f3d68cea9839654ea8ff606162a4cb39
EXT_SHA256=011f36a8d3efbc776824504da44abfa786890a6bf9028ec47e00b926415f1109
GDCORE_TOOLS=gdcore-tools@2.0.0-gd-v5.6.269-autobuild
PUPPETEER=puppeteer-core@25.12.0
ASAR=@electron/asar@4.3.0

mkdir -p "$V"
cd "$V"

versao_instalada="$(pacman -Q gdevelop-bin 2>/dev/null | awk '{print $2}' | cut -d- -f1 || true)"
if [ "$versao_instalada" != "$GD_VERSION_ESPERADA" ]; then
  echo "AVISO: gdevelop-bin instalado é '$versao_instalada', o harness foi fixado em $GD_VERSION_ESPERADA." >&2
  echo "       As fixtures registam a versão efectiva no cabeçalho; compare antes de confiar." >&2
fi

# 1. libGD (o núcleo C++ do GDevelop compilado para WASM), tirado do app.asar instalado.
mkdir -p gd/lib
if [ ! -s gd/lib/libGD.cjs ] || [ ! -s gd/lib/libGD.wasm ]; then
  tmp="$(mktemp -d)"
  ( cd "$tmp" && npx --yes "$ASAR" extract-file "$GD_APP/app.asar" www/libGD.js \
              && npx --yes "$ASAR" extract-file "$GD_APP/app.asar" www/libGD.wasm )
  cp "$tmp/libGD.js" gd/lib/libGD.cjs
  cp "$tmp/libGD.wasm" gd/lib/libGD.wasm
  rm -rf "$tmp"
fi

# 2. O runtime GDJS instalado (o exportador copia daqui para o jogo exportado).
if [ ! -d gd/Runtime ]; then
  cp -a "$GD_APP/GDJS/Runtime" gd/Runtime
fi

# 3. O carregador de extensões do IDE (gdcore-tools empacota-o).
if [ ! -s gdcore-tools/dist/loaders.cjs ]; then
  tmp="$(mktemp -d)"
  ( cd "$tmp" && npm pack --silent "$GDCORE_TOOLS" >/dev/null && tar xzf ./*.tgz )
  rm -rf gdcore-tools && mv "$tmp/package" gdcore-tools
  rm -rf "$tmp"
fi

# 4. A extensão Health, fixada por commit e por sha256.
if [ ! -s Health.json ]; then
  curl -sfL -o Health.json \
    "https://raw.githubusercontent.com/GDevelopApp/GDevelop-extensions/$EXT_COMMIT/extensions/reviewed/Health.json"
fi
echo "$EXT_SHA256  Health.json" | sha256sum -c --quiet

# 5. puppeteer-core (cliente CDP para o Chrome headless do sistema; não descarrega browser).
if [ ! -d node_modules/puppeteer-core ]; then
  [ -f package.json ] || echo '{"name":"gdevelop-health-oracle-vendor","private":true}' > package.json
  PUPPETEER_SKIP_DOWNLOAD=1 npm install --silent --no-audit --no-fund "$PUPPETEER"
fi

cat > .gitignore <<'EOF'
# Artefactos de terceiros re-obtidos por ../setup.sh — nunca vão para o git.
*
!.gitignore
EOF

echo "vendor pronto em $V"
