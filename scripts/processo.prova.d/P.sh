# shellcheck shell=bash
# Aceleração 04/11: o pré-voo estático corre ANTES da suíte, e `linha` dá o B2 a uma linha só.
echo "— P: o pré-voo antes da suíte e o B2 de linha única —"
_p="$tmp/p"
_gp() { git -C "$_p" -c user.email=p@p -c user.name=p "$@"; }
git init -q -b main "$_p"
mkdir -p "$_p/scripts" "$tmp/pbin"
cp scripts/suite-completa.sh scripts/tempos-teste-lib.sh "$_p/scripts/"
cat >"$_p/scripts/ship.sh" <<'SH'
echo ship >>"$MARCA"
exit "${SHIP_RC:-0}"
SH
cat >"$_p/scripts/nextest-filtros-casam.py" <<'PY'
PY
cat >"$tmp/pbin/cargo" <<'SH'
#!/bin/sh
echo "cargo $2" >>"$MARCA"
exit 0
SH
chmod +x "$tmp/pbin/cargo"
_gp add -A && _gp commit -q -m base
_corre() { # exit na variável _rp; o que correu em _mp
  : >"$tmp/marca"
  (cd "$_p" && env "$@" MARCA="$tmp/marca" PH2D_NA_PORTA=1 PH2D_SUITES_VERDES="$tmp/pverdes.tsv" \
    PH2D_TEMPOS_LOG="$tmp/ptempos.tsv" PATH="$tmp/pbin:$PATH" bash scripts/suite-completa.sh >/dev/null 2>&1)
  _rp=$?; _mp="$(tr '\n' ' ' <"$tmp/marca")"
}
_corre SHIP_RC=1
igual "pré-voo vermelho: exit 3" "$_rp" 3
igual "pré-voo vermelho: nem compila nem corre" "$_mp" "ship "
[ ! -s "$tmp/pverdes.tsv" ] && ok "pré-voo vermelho não regista a árvore" || ko "pré-voo vermelho registou a árvore"
_corre PH2D_PREVOO=0
igual "PH2D_PREVOO=0 salta o pré-voo (o ship.sh já o correu)" "$_mp" "cargo list cargo run "
: >"$tmp/pverdes.tsv"
_corre SHIP_RC=0 PH2D_PORTAO_FORCAR=1
igual "pré-voo verde: a suíte corre depois dele" "$_mp" "ship cargo list cargo run "
_corre SHIP_RC=1
igual "árvore já verde salta sem pré-voo" "$_mp" ""
igual "PH2D_PREVOO fora da chave" \
  "$(cd "$_p" && PH2D_NA_PORTA=1 PH2D_PREVOO=0 bash scripts/suite-completa.sh --chave)" \
  "$(cd "$_p" && PH2D_NA_PORTA=1 bash scripts/suite-completa.sh --chave)"
_a1="$(grep -n 'PH2D_PREVOO=0' scripts/ship.sh | head -1 | cut -d: -f1)"
_a2="$(grep -n 'suite-completa.sh --enviar' scripts/ship.sh | head -1 | cut -d: -f1)"
_a3="$(grep -n '_so_estatico" = 1' scripts/ship.sh | head -1 | cut -d: -f1)"
{ [ -n "$_a1" ] && [ -n "$_a2" ] && [ -n "$_a3" ] && [ "$_a3" -lt "$_a1" ] && [ "$_a1" -lt "$_a2" ]; } \
  && ok "ship.sh: --so-estatico sai antes da suíte, e a suíte dele não repete o pré-voo" \
  || ko "ship.sh perdeu a ordem --so-estatico → PH2D_PREVOO=0 → suíte"
# B2 de uma linha só
_gp checkout -q -b line/x && echo x >"$_p/x" && _gp add -A && _gp commit -q -m x
_s="$(PH2D_INTEG_SUITE=true PH2D_SUITES_VERDES="$tmp/pverdes2.tsv" bash scripts/integrar-rodada.sh -C "$_p" suite 2>&1)"; _r=$?
{ [ "$_r" = 2 ] && case "$_s" in *linha*) true ;; *) false ;; esac; } \
  && ok "suite sem estado manda correr «linha»" || ko "suite sem estado: exit $_r, $_s"
PH2D_SUITES_VERDES="$tmp/pverdes2.tsv" bash scripts/integrar-rodada.sh -C "$_p" linha >/dev/null 2>&1
igual "linha: exit 0 com o main ancestral" "$?" 0
_e="$(git -C "$_p" rev-parse --git-dir)"
case "$_e" in /*) ;; *) _e="$_p/$_e" ;; esac
igual "linha: um degrau na cadeia, sobre o main" "$(cut -f1,2,5 "$_e/integ-rodada/estado.tsv")" "1	line/x	1"
PH2D_INTEG_SUITE=true PH2D_INTEG_JUNIT="$tmp/nao-existe.xml" PH2D_SUITES_VERDES="$tmp/pverdes2.tsv" \
  bash scripts/integrar-rodada.sh -C "$_p" suite >/dev/null 2>&1
igual "linha → suite corre" "$?" 0
_gp checkout -q main && echo m >"$_p/m" && _gp add -A && _gp commit -q -m m && _gp checkout -q line/x
PH2D_SUITES_VERDES="$tmp/pverdes2.tsv" bash scripts/integrar-rodada.sh -C "$_p" linha >/dev/null 2>&1
igual "linha com o main à frente: recusa (exit 2)" "$?" 2
_o="$(grep -c 'cat "$EST/ramo"' scripts/integrar-rodada.sh)"
igual "ff usa o ramo da linha" "$_o" 1
