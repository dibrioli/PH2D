# shellcheck shell=bash
# Item B1 (comissão 29/10): o `ship.sh` salta a suíte só com a MESMA árvore (`scripts/suite-completa.sh`).
echo "— B1: a suíte salta só com a mesma árvore —"
_b1="$tmp/b1"
git init -q "$_b1" && git -C "$_b1" -c user.email=p@p -c user.name=p commit -q --allow-empty -m base
mkdir -p "$_b1/scripts" && cp scripts/suite-completa.sh "$_b1/scripts/"
echo a >"$_b1/f.txt"
_k() { (cd "$_b1" && PH2D_NA_PORTA=1 bash scripts/suite-completa.sh --chave); }
_k1="$(_k)"
igual "a chave é estável" "$(_k)" "$_k1"
echo b >"$_b1/f.txt"
[ "$(_k)" != "$_k1" ] && ok "um byte por comitar muda a chave" || ko "um byte por comitar NÃO mudou a chave"
echo a >"$_b1/f.txt"
git -C "$_b1" -c user.email=p@p -c user.name=p commit -q --allow-empty -m outro-sha
igual "outro SHA com a mesma árvore = mesma chave" "$(_k)" "$_k1"
[ "$(cd "$_b1" && PH2D_NA_PORTA=1 PH2D_X=1 bash scripts/suite-completa.sh --chave)" != "$_k1" ] \
  && ok "uma PH2D_* nova muda a chave" || ko "uma PH2D_* nova NÃO mudou a chave"
igual "PH2D_PORTAO_FORCAR não entra na chave" "$(cd "$_b1" && PH2D_NA_PORTA=1 PH2D_PORTAO_FORCAR=1 bash scripts/suite-completa.sh --chave)" "$_k1"
# O salto: um registo com a chave salta (exit 0 sem cargo); sem ele, tentaria correr (aqui não há cargo
# workspace — `PATH` sem cargo faz a corrida falhar alto, que é o que distingue).
printf '%s\tarvore\tprova\n' "$_k1" >"$tmp/verdes.tsv"
_s="$(cd "$_b1" && PH2D_NA_PORTA=1 PH2D_SUITES_VERDES="$tmp/verdes.tsv" bash scripts/suite-completa.sh 2>&1)"
case "$_s" in *"JÁ PASSOU"*) ok "a mesma chave registada salta" ;; *) ko "a mesma chave registada NÃO saltou: $_s" ;; esac
echo c >"$_b1/f.txt"
_s="$(cd "$_b1" && PH2D_NA_PORTA=1 PH2D_SUITES_VERDES="$tmp/verdes.tsv" PATH=/usr/bin:/bin bash scripts/suite-completa.sh 2>&1)"
case "$_s" in *"JÁ PASSOU"*) ko "outra árvore SALTOU a suíte" ;; *) ok "outra árvore não salta" ;; esac
grep -q 'bash scripts/suite-completa.sh' scripts/ship.sh && ok "o ship.sh corre a suíte pelo suite-completa.sh" \
  || ko "o ship.sh já não passa pelo suite-completa.sh"
