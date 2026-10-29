# shellcheck shell=bash
# Item K (comissão 29/10): compilar × correr × load por corrida — `scripts/tempos-teste-lib.sh`.
echo "— K: compilar × correr —"
# shellcheck source=scripts/tempos-teste-lib.sh
source scripts/tempos-teste-lib.sh
printf '%s\n' '   Compiling x v0.1.0' \
  '    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 05s' \
  'test result: ok. 3 passed; 0 failed; finished in 40.00s' \
  '    Finished `ci-test` profile [optimized + debuginfo] target(s) in 12.50s' >"$tmp/saida"
igual "soma os Finished (1m 05s + 12.50s)" "$(tt_compilar_de "$tmp/saida")" "77.5"
igual "sem Finished = 0" "$(tt_compilar_de /dev/null)" "0.0"
PH2D_TEMPOS_LOG="$tmp/log.tsv" tt_registar prova alvo 2.0 77.5 100.0 1.0 0 >/dev/null
igual "o log ganha cabeçalho + 1 linha" "$(wc -l <"$tmp/log.tsv")" "2"
igual "correr = total − compilar" "$(tail -1 "$tmp/log.tsv" | cut -f7)" "22.5"
igual "o agregador lê o log" "$(python3 scripts/tempos-teste.py "$tmp/log.tsv" | awk '/^TOTAL/{print $3}')" "80s"

