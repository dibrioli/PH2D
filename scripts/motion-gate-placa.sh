#!/usr/bin/env bash
# motion-gate-placa.sh — o portão de fecho do Motion que precisa da PLACA (doc 123 §5).
#
# Uso:
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash scripts/motion-gate-placa.sh [-- <args do libtest>]
#   bash scripts/motion-gate-placa.sh --listar     # só os nomes e a contagem; não compila nem corre
#   PH2D_PLACA_BIN=<binário de teste> ...          # reusa um binário já compilado (salta o build)
#
# Porquê: `cargo test -p ph2d-app-motion --lib -- --include-ignored` corre ~296 ignorados e passa
# a fatia de 1800 s — a maior parte são SONDAS (imprimem, nunca reprovam) e GERADORES de figuras,
# e duas sondas nem acabam em 900 s. A classe vive no prefixo da razão do `#[ignore]` (`placa: `,
# `sonda: `, `gerador: `; o censo `motion_ignore_census` garante-o), que nem o libtest nem o
# nextest filtram. Este script lê o prefixo nas fontes e corre SÓ os `placa: ` num processo, em
# RELEASE, com cwd = a pasta da crate (como o `cargo test`). Exit = o do binário de teste.
set -euo pipefail

raiz="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
crate="$raiz/crates/ph2d-app-motion"

listar=0
if [[ "${1:-}" == "--listar" ]]; then
  listar=1
  shift
fi
if [[ "${1:-}" == "--" ]]; then
  shift
elif [[ $# -gt 0 ]]; then
  echo "uso: $0 [--listar] [-- <args do libtest>]" >&2
  exit 2
fi

# 1. As fns `placa:` pelas fontes: a linha do atributo arma, a próxima `fn <nome>` desarma.
attrs=$(find "$crate/src" "$crate/tests" -name '*.rs' -print0 | xargs -0 cat \
  | grep -cE '^[[:space:]]*#\[ignore = "placa: ' || true)
mapfile -t fns < <(
  find "$crate/src" "$crate/tests" -name '*.rs' -print0 | sort -z | xargs -0 awk '
    FNR == 1 { arm = 0 }
    /^[[:space:]]*#\[ignore = "placa: / { arm = 1; next }
    arm && /^[[:space:]]*(pub[^ ]* )?(async )?fn [A-Za-z_]/ {
      match($0, /fn [A-Za-z_][A-Za-z0-9_]*/); print substr($0, RSTART + 3, RLENGTH - 3); arm = 0
    }'
)
if [[ ${#fns[@]} -ne $attrs || $attrs -eq 0 ]]; then
  echo "✗ $attrs atributos \`placa:\` mas ${#fns[@]} fns achadas" >&2
  exit 2
fi
dups=$(printf '%s\n' "${fns[@]}" | sort | uniq -d)
if [[ -n "$dups" ]]; then
  echo "✗ nome de fn \`placa:\` repetido (o --exact não os distingue): $dups" >&2
  exit 2
fi

if [[ $listar -eq 1 && -z "${PH2D_PLACA_BIN:-}" ]]; then
  printf '%s\n' "${fns[@]}"
  echo "${#fns[@]} testes placa"
  exit 0
fi

# 2. O binário de teste da lib, em RELEASE, compilado uma vez.
if [[ -n "${PH2D_PLACA_BIN:-}" ]]; then
  bin="$PH2D_PLACA_BIN"
else
  bin=$(cd "$raiz" && cargo test -p ph2d-app-motion --lib --release --no-run --message-format=json \
    | jq -r 'select(.reason == "compiler-artifact" and .target.name == "ph2d_app_motion"
                    and .profile.test == true and .executable != null) | .executable')
fi
if [[ -z "$bin" || ! -x "$bin" ]]; then
  echo "✗ binário de teste não encontrado: '$bin'" >&2
  exit 2
fi

# 3. Cada fn casa EXACTAMENTE um ignorado listado (caminho do módulo completo).
mapfile -t listados < <(cd "$crate" && "$bin" --list --ignored --format=terse | sed -n 's/: test$//p')
nomes=()
for f in "${fns[@]}"; do
  casa=()
  for t in "${listados[@]}"; do
    [[ "$t" == "$f" || "$t" == *"::$f" ]] && casa+=("$t")
  done
  if [[ ${#casa[@]} -ne 1 ]]; then
    echo "✗ \`$f\` casa ${#casa[@]} testes ignorados no binário (módulo fora do cfg?): ${casa[*]:-}" >&2
    exit 2
  fi
  nomes+=("${casa[0]}")
done

if [[ $listar -eq 1 ]]; then
  printf '%s\n' "${nomes[@]}"
  echo "${#nomes[@]} testes placa (resolvidos em $bin)"
  exit 0
fi

echo "▸ ${#nomes[@]} testes placa · $bin"
cd "$crate"
exec "$bin" --ignored --exact "$@" "${nomes[@]}"
