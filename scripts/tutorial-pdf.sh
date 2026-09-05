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

# ⛔⛔ **O INCLUDE, e por que ele existe.** A 1.a versao deste tutorial buscava a tabela
# derivada com `fetch()` -- e num `file://` o browser NAO a busca: o PDF saiu com a seccao 6
# **em branco**, a prometer uma tabela que nao estava la'. *Um tutorial que promete e nao
# entrega e' pior que um que nao promete.* Agora a substituicao e' feita AQUI, antes de
# imprimir, e um include que nao resolva ou venha vazio ABORTA a geracao.
expand_includes() {
  local f="$1" dir out line inc
  dir="$(dirname "$f")"
  out=""
  while IFS= read -r line; do
    case "$line" in
      *"<!--#include "*)
        inc="${line#*<!--#include }"
        inc="${inc%%-->*}"
        inc="$(echo "$inc" | tr -d ' ')"
        if [ ! -s "$dir/$inc" ]; then
          echo "✗ include vazio ou ausente: $dir/$inc" >&2
          return 1
        fi
        out="$out$(cat "$dir/$inc")"$'\n'
        ;;
      *) out="$out$line"$'\n' ;;
    esac
  done < "$f"
  printf '%s' "$out"
}

base="$(basename "$src" .html)"
outdir="$(dirname "$(dirname "$src")")"      # .../tutoriais/src -> .../tutoriais
out="$outdir/$base.pdf"
profile="$(mktemp -d)"
expanded="$(dirname "$src")/.$base.expanded.html"
trap 'rm -rf "$profile"; rm -f "$expanded"' EXIT

expand_includes "$src" > "$expanded" || exit 1

# --print-to-pdf-no-header tira o cabecalho/rodape do browser (URL e data), que
# num tutorial impresso e' ruido. O perfil temporario evita tocar no do Enio.
"$chrome" --headless=new --disable-gpu --no-sandbox \
  --user-data-dir="$profile" \
  --no-pdf-header-footer \
  --print-to-pdf="$out" \
  --virtual-time-budget=10000 \
  "file://$(realpath "$expanded")" >/dev/null 2>&1

[ -s "$out" ] || { echo "✗ o PDF saiu vazio: $out"; exit 1; }
bytes=$(stat -c%s "$out")
echo "✓ $out  ($((bytes/1024)) KiB)"
