# tempos-teste-lib.sh — fonte (`source`) partilhada pelos portões de teste: quanto de uma corrida é
# COMPILAR e quanto é CORRER, com o `loadavg` ao lado (comissão de velocidade 29/10, item K).
# Medido antes: os `cargo test` somavam 71 h/semana e ninguém sabia a divisão.
#
# O log é um TSV por clone (`git-common-dir`, partilhado entre worktrees), agregado por
# `python3 scripts/tempos-teste.py`. Colunas: TT_COLUNAS.

TT_COLUNAS="data	script	branch	alvo	check_s	compilar_s	correr_s	total_s	load_ini	load_fim	rc"

tt_agora() { date +%s.%N; }
tt_load() { cut -d' ' -f1 /proc/loadavg 2>/dev/null || echo "?"; }
tt_dif() { awk -v a="$1" -v b="$2" 'BEGIN{printf "%.1f", b-a}'; }

# Soma dos `Finished … in 12.34s` / `in 1m 05s` do cargo numa saída (o tempo de BUILD que ele reporta).
tt_compilar_de() {
  grep -aoE 'Finished `[^`]+` profile .* in ([0-9]+m )?[0-9.]+s' "$1" 2>/dev/null \
    | sed -E 's/.* in //' \
    | awk '{ t=0; if ($0 ~ /m /) { t=$1*60; s=$2 } else { s=$1 }
             sub(/s$/,"",s); tot+=t+s } END { printf "%.1f", tot+0 }'
}

tt_ficheiro() { echo "${PH2D_TEMPOS_LOG:-$(git rev-parse --git-common-dir 2>/dev/null || echo .)/ph2d-tempos-teste.tsv}"; }

# tt_registar <script> <alvo> <check_s> <compilar_s> <total_s> <load_ini> <rc>
# `correr_s` = total − compilar (o arranque dos binários conta como correr).
tt_registar() {
  local f correr
  f="$(tt_ficheiro)"
  correr="$(awk -v t="$5" -v c="$4" 'BEGIN{r=t-c; if (r<0) r=0; printf "%.1f", r}')"
  [ -f "$f" ] || printf '%s\n' "$TT_COLUNAS" >"$f" 2>/dev/null
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$(date '+%F %T')" "$1" \
    "$(git branch --show-current 2>/dev/null)" "$2" "$3" "$4" "$correr" "$5" "$6" "$(tt_load)" "$7" \
    >>"$f" 2>/dev/null
  echo "⏱ check ${3}s · compilar ${4}s · correr ${correr}s · load $6→$(tt_load)"
}
