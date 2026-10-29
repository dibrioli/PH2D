# shellcheck shell=bash
# B1 (cont., 29/10): o que vai no push é o HEAD — `suite-completa.sh --enviar` (o ship.sh) salta com o HEAD
# verde mesmo com a pasta suja (o primário vive com `project-memory/` por comitar); sem --enviar, não.
echo "— B1b: o envio olha para o que ENVIA —"
_b1h="$tmp/b1h"
git init -q "$_b1h"; mkdir -p "$_b1h/scripts"; cp scripts/suite-completa.sh "$_b1h/scripts/"
echo limpo >"$_b1h/f.txt"; git -C "$_b1h" add -A
git -C "$_b1h" -c user.email=p@p -c user.name=p commit -q -m h
printf '%s\tarvore\tprova\n' "$(cd "$_b1h" && PH2D_NA_PORTA=1 bash scripts/suite-completa.sh --chave)" >"$tmp/verdes-h.tsv"
echo rascunho >"$_b1h/memoria.md"
_s="$(cd "$_b1h" && PH2D_NA_PORTA=1 PH2D_SUITES_VERDES="$tmp/verdes-h.tsv" bash scripts/suite-completa.sh --enviar 2>&1)"
case "$_s" in *"push ENVIA"*) ok "--enviar salta com o HEAD verde e a pasta suja" ;; *) ko "--enviar NÃO saltou com o HEAD verde: $_s" ;; esac
_s="$(cd "$_b1h" && PH2D_NA_PORTA=1 PH2D_SUITES_VERDES="$tmp/verdes-h.tsv" PATH=/usr/bin:/bin bash scripts/suite-completa.sh 2>&1)"
case "$_s" in *"JÁ PASSOU"*) ko "sem --enviar a pasta suja SALTOU" ;; *) ok "sem --enviar a pasta suja não salta" ;; esac
grep -q 'bash scripts/suite-completa.sh --enviar' scripts/ship.sh && ok "o ship.sh pede o salto do que ENVIA" \
  || ko "o ship.sh não passa --enviar"
