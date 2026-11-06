#!/usr/bin/env bash
# A rodada de W2 do SFX Lab (CLAUDE.md §0.10): aliasing, custo, oráculo `sox`, hash por alvo e símbolos.
# Corra de qualquer lugar: bash docs/SfxLab/ferramentas/sonda_w2/corre.sh
# Oráculo só executado (GPL-2.0-only, triado em docs/SfxLab/02_W0_medicao.md). A fixture sai em
# crates/ph2d-sfx-dsp/tests/it/fixtures/aliasing_sox.tsv.
set -euo pipefail
AQUI="$(cd "$(dirname "$0")" && pwd)"
RAIZ="$(git -C "$AQUI" rev-parse --show-toplevel)"
T="$RAIZ/target/sonda-w2"
SAIDA="$T/saida"
mkdir -p "$SAIDA"
cd "$AQUI"
RUN="bash $RAIZ/scripts/ph2d-run.sh"

export CARGO_TARGET_DIR="$T/base"
$RUN cargo build --profile smoke --bin sonda --bin hashes 2>&1 | tail -1
CARGO_TARGET_DIR="$T/v3" RUSTFLAGS="-C target-cpu=x86-64-v3" \
  $RUN cargo build --profile smoke --bin hashes 2>&1 | tail -1
CARGO_TARGET_DIR="$T/wasm" \
  $RUN cargo build --profile smoke --lib --target wasm32-unknown-unknown 2>&1 | tail -1

"$T/base/smoke/sonda" "$SAIDA"

# O oráculo: passa-banda de 20 Hz a 0,8 × fundamental e RMS do que sobrou, sobre o RMS do arquivo. Sem conta nossa.
rms() { sox "$1" -n stat 2>&1 | awk '/RMS +amplitude/ {print $3}'; }
db() { awk -v a="$1" -v b="$2" 'BEGIN { if (a <= 0) print "-200.0"; else printf "%.1f\n", 20 * log(a / b) / log(10) }'; }
# O `stat` escreve 6 casas. Filtrado fraco sobe 60 dB para o número não virar 0,000000 e a conta devolve os 60;
# filtrado forte fica como está, porque subir 60 dB o saturaria.
mede() {
  sox "$1" -e float -b 32 "$SAIDA/inteiro.wav" trim 0.5 1
  sox "$1" -e float -b 32 "$SAIDA/filtrado.wav" sinc -n 32767 "20-$2" trim 0.5 1
  local direto; direto="$(rms "$SAIDA/filtrado.wav")"
  local valor
  if awk -v a="$direto" 'BEGIN { exit !(a >= 0.001) }'; then
    valor="$(db "$direto" "$(rms "$SAIDA/inteiro.wav")")"
  else
    sox "$1" -e float -b 32 "$SAIDA/filtrado.wav" sinc -n 32767 "20-$2" trim 0.5 1 vol 1000
    valor="$(db "$(rms "$SAIDA/filtrado.wav")" "$(rms "$SAIDA/inteiro.wav")" | awk '{printf "%.1f", $1 - 60}')"
  fi
  printf '%s\t%s\t%s\n' "$3" "$4" "$valor"
}
FIX="$RAIZ/crates/ph2d-sfx-dsp/tests/it/fixtures/aliasing_sox.tsv"
{
  echo "# Oráculo de aliasing: $(sox --version | head -1), executado por docs/SfxLab/ferramentas/sonda_w2/corre.sh em $(date +%F)."
  echo "# Comando: sox entrada.wav saida.wav sinc -n 32767 20-<0,8×hz> trim 0.5 1 (vol 1000 quando o RMS é menor que 0,001); dB = 20·log10(RMS saída / RMS entrada)."
  echo "# O trim tira o estalo das pontas do arquivo, depois de o filtro esquecê-lo. O RMS de entrada tem o mesmo trim."
  echo "# A linha «ingenua» é a serra do próprio sox: o controlo de que a medida enxerga o fenômeno."
  echo "# onda	hz	dB"
  for w in serra quadrada triangulo; do
    for hz in 5000 9000 15000; do
      f="$SAIDA/${w}_$hz.wav"
      corte=$(awk -v h="$hz" 'BEGIN { printf "%d", 0.8 * h }')
      mede "$f" "$corte" "$w" "$hz"
    done
  done
  sox -n -r 48000 -e float -b 32 -c 1 "$SAIDA/ingenua.wav" synth 2 saw 15000 vol 0.5
  mede "$SAIDA/ingenua.wav" 12000 ingenua 15000
} > "$FIX"

echo; echo "=== o oráculo"
cat "$FIX"
echo; echo "=== hash por alvo"
echo "-- x86-64 de base"; "$T/base/smoke/hashes"
echo "-- x86-64-v3 (AVX2 e FMA no compilador)"; "$T/v3/smoke/hashes" | tail -1
echo "-- wasm32 no node"
node -e '
const fs=require("fs");
const m=new WebAssembly.Instance(new WebAssembly.Module(fs.readFileSync(process.argv[1])),{});
console.log("conjunto\t"+BigInt.asUintN(64,m.exports.hash_conjunto()).toString(16).padStart(16,"0"));
' "$T/wasm/wasm32-unknown-unknown/smoke/sonda_w2.wasm"

MATE='^(sin|cos|tan|exp|exp2|log|log2|log10|pow|sinh|cosh|tanh|atan|atan2|asin|acos|cbrt|hypot|fma)f?(@|$)'
echo; echo "=== matemática do sistema importada pelo binário dos hashes (nada = certo)"
nm -D --undefined-only "$T/base/smoke/hashes" | awk '{print $NF}' | grep -E "$MATE" | sed 's/@.*//' | sort -u | tr '\n' ' '
echo
