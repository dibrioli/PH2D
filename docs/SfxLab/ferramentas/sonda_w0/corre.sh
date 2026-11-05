#!/usr/bin/env bash
# A rodada de W0 do SFX Lab (CLAUDE.md §0.10): custo, semente, rack, volume, símbolos e hash por alvo.
# Corra de qualquer lugar: bash docs/SfxLab/ferramentas/sonda_w0/corre.sh
set -euo pipefail
AQUI="$(cd "$(dirname "$0")" && pwd)"
RAIZ="$(git -C "$AQUI" rev-parse --show-toplevel)"
T="$RAIZ/target/sonda-w0"
SAIDA="$T/saida"
mkdir -p "$SAIDA"
cd "$AQUI"
RUN="bash $RAIZ/scripts/ph2d-run.sh"

export CARGO_TARGET_DIR="$T/base"
$RUN cargo build --profile smoke --bin sonda --features rack 2>&1 | tail -1
$RUN cargo build --profile smoke --bin so_kernels --bin so_fundsp 2>&1 | tail -1
CARGO_TARGET_DIR="$T/v3" RUSTFLAGS="-C target-cpu=x86-64-v3" \
  $RUN cargo build --profile smoke --bin so_kernels --bin so_fundsp 2>&1 | tail -1
CARGO_TARGET_DIR="$T/wasm" \
  $RUN cargo build --profile smoke --lib --target wasm32-unknown-unknown 2>&1 | tail -1

echo "=== rodada"
"$T/base/smoke/sonda" "$SAIDA"

echo; echo "=== hash por alvo (2 s, receita típica)"
echo "-- x86-64 de base";  "$T/base/smoke/so_kernels"; "$T/base/smoke/so_fundsp"
echo "-- x86-64-v3 (AVX2 e FMA no compilador)"; "$T/v3/smoke/so_kernels"; "$T/v3/smoke/so_fundsp"
echo "-- wasm32 no node"
node -e '
const fs=require("fs");
const m=new WebAssembly.Instance(new WebAssembly.Module(fs.readFileSync(process.argv[1])),{});
const x=v=>BigInt.asUintN(64,v).toString(16).padStart(16,"0");
for (const [n,f] of [["kernels","hash_kernels"],["fundsp_tick","hash_fundsp_tick"],["fundsp_bloco","hash_fundsp_bloco"]])
  console.log(n,x(m.exports[f]()));
' "$T/wasm/wasm32-unknown-unknown/smoke/sonda_w0.wasm"

MATE='^(sin|cos|tan|exp|exp2|log|log2|log10|pow|sinh|cosh|tanh|atan|atan2|asin|acos|cbrt|hypot|fma)f?(@|$)'
echo; echo "=== matemática do sistema que cada binário importa"
for b in so_kernels so_fundsp sonda; do
  echo "-- $b: $(nm -D --undefined-only "$T/base/smoke/$b" | awk '{print $NF}' | grep -E "$MATE" | sed 's/@.*//' | sort -u | tr '\n' ' ')"
done
echo; echo "=== versões dos oráculos"
ffmpeg -hide_banner -version | head -1
sox --version
