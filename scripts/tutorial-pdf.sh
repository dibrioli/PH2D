#!/usr/bin/env bash
# Gera o PDF de um tutorial a partir da fonte HTML (doc 103 §3).
#
# ⚠️ FERRAMENTA MEDIDA nesta máquina (2026-09-05): não há typst/pandoc/weasyprint/
# wkhtmltopdf/xelatex/libreoffice. Há `google-chrome-stable`, e é por isso que o
# caminho é HTML -> Chrome headless -> PDF. Se um dia houver `typst`, ele é melhor
# (determinístico, sem browser) — mas trocar exige re-medir, não supor.
#
# Uso:  bash scripts/tutorial-pdf.sh "docs/Motion Nodes/tutoriais/src/01_arranjo.html"
# Saída: docs/Motion Nodes/tutoriais/01_arranjo.pdf
set -euo pipefail

src="${1:?uso: tutorial-pdf.sh <fonte.html>}"
[ -f "$src" ] || { echo "✗ fonte nao existe: $src"; exit 1; }

chrome="$(command -v google-chrome-stable || command -v chromium || true)"
[ -n "$chrome" ] || { echo "✗ sem google-chrome-stable/chromium — ver doc 103 §3"; exit 1; }

base="$(basename "$src" .html)"
outdir="$(dirname "$(dirname "$src")")"      # .../tutoriais/src -> .../tutoriais
out="$outdir/$base.pdf"
profile="$(mktemp -d)"
trap 'rm -rf "$profile"' EXIT

# --print-to-pdf-no-header tira o cabecalho/rodape do browser (URL e data), que
# num tutorial impresso e' ruido. O perfil temporario evita tocar no do Enio.
"$chrome" --headless=new --disable-gpu --no-sandbox \
  --user-data-dir="$profile" \
  --no-pdf-header-footer \
  --print-to-pdf="$out" \
  --virtual-time-budget=10000 \
  "file://$(realpath "$src")" >/dev/null 2>&1

[ -s "$out" ] || { echo "✗ o PDF saiu vazio: $out"; exit 1; }
bytes=$(stat -c%s "$out")
echo "✓ $out  ($((bytes/1024)) KiB)"
