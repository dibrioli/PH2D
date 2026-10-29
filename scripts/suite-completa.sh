#!/usr/bin/env bash
# suite-completa.sh — a suíte COMPLETA (`nextest --workspace`, perfil `ci-test`) com o REGISTO da árvore
# verde. Quem a corre: o integrador antes do `--ff-only` e o `ship.sh` (dono 23/10: nunca a linha).
#
# ⭐ Item B1 da comissão de velocidade (29/10): o `ship.sh` corria outra suíte inteira (+25–35 min) sobre
# a MESMA árvore que o integrador acabara de passar. Agora a suíte verde regista a CHAVE da árvore e uma
# segunda corrida sobre a mesma chave não corre. A regra é «mesma árvore», nunca «mesmo SHA»: um rebase
# que não mexe em conteúdo dá outro SHA e a mesma árvore; um ficheiro por comitar dá o mesmo SHA e outra
# árvore. A chave = a árvore INTEIRA (commitada + por commitar + por rastrear, por um índice temporário,
# como o selo do `nextest-impacted.sh`) + `rustc -vV` + os argumentos + as `PH2D_*`/`CARGO_*` que mudam o
# veredito. `PH2D_PORTAO_FORCAR=1` corre na mesma (flake a confirmar, driver novo).
#
#     bash scripts/suite-completa.sh            # corre (ou diz que esta árvore já passou)
#     bash scripts/suite-completa.sh --chave    # só imprime a chave (para a prova)
set -uo pipefail

if [ "${PH2D_NA_PORTA:-0}" != "1" ]; then
  _porta="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/ph2d-run.sh"
  [ -x "$_porta" ] && exec env PH2D_PRAZO="${PH2D_PRAZO:-5400}" bash "$_porta" bash "${BASH_SOURCE[0]}" "$@"
fi
_raiz="$(git rev-parse --show-toplevel)" || exit 2
cd "$_raiz" || exit 2
export CARGO_INCREMENTAL=0
# Paridade com o CI (o `ship.sh` diz porquê) — e a MESMA chave venha de quem vier: o integrador que chama
# isto à mão e o `ship.sh` têm de calcular a mesma, senão o envio nunca salta.
export CARGO_BUILD_WARNINGS="${CARGO_BUILD_WARNINGS:-deny}"

suite_arvore() {
  local idx arv
  idx="$(mktemp)"
  cp "$(git rev-parse --git-dir)/index" "$idx" 2>/dev/null
  # A lista do nível lento sai da chave: esta suíte corre TODOS os níveis, a lista não muda o veredito
  # (e é ela própria que a regera — dentro da chave, cada verde mudaria a árvore que acabou de passar).
  arv="$(GIT_INDEX_FILE="$idx" git add -A -- "$_raiz" 2>/dev/null \
    && GIT_INDEX_FILE="$idx" git rm --cached -q --ignore-unmatch -- .config/testes-lentos.tsv >/dev/null 2>&1 \
    && GIT_INDEX_FILE="$idx" git write-tree 2>/dev/null)"
  rm -f "$idx"
  printf '%s' "$arv"
}
# ⭐ O que o PUSH envia é a árvore do HEAD, não a pasta: o primário do dono vive com `project-memory/` por
# comitar (outras sessões), e com a chave só da pasta o envio nunca reconhecia a árvore que o integrador
# passou num worktree limpo (medido no ensaio do A, 29/10). `--enviar` (o ship.sh) aceita também a chave
# do HEAD — o por comitar não vai no push. Uma corrida, quando acontece, continua a testar a pasta.
suite_arvore_head() {
  local idx arv
  idx="$(mktemp)"
  arv="$(GIT_INDEX_FILE="$idx" git read-tree HEAD 2>/dev/null \
    && GIT_INDEX_FILE="$idx" git rm --cached -q --ignore-unmatch -- .config/testes-lentos.tsv >/dev/null 2>&1 \
    && GIT_INDEX_FILE="$idx" git write-tree 2>/dev/null)"
  rm -f "$idx"
  printf '%s' "$arv"
}
# ⚠️ Fora da chave só os botões de RECURSO da porta e o próprio forçar (o mesmo corte do nextest-impacted).
suite_chave() {
  printf '%s\n%s\n%s\n%s\n' "$1" "$(rustc -vV 2>/dev/null)" "$2" \
    "$(env | grep -E '^(PH2D_|CARGO_)' | grep -vE '^PH2D_(PORTAO_FORCAR|NA_PORTA|PRAZO|GPU|GPU_ESPERA|GPU_VEZ|GPU_FECHADURA|CPU_PCT|MEM_MAX|SUITE|SUITE_CADA|TEMPOS_LOG|SUITES_VERDES)=' | sort)" \
    | sha256sum | cut -c1-32
}
_registo="${PH2D_SUITES_VERDES:-$(git rev-parse --git-common-dir)/ph2d-suites-verdes.tsv}"

_so_chave=0; _enviar=0
[ "${1:-}" = "--chave" ] && { _so_chave=1; shift; }
[ "${1:-}" = "--enviar" ] && { _enviar=1; shift; }
_arvore="$(suite_arvore)"
_chave="$(suite_chave "$_arvore" "$*")"
[ "$_so_chave" = 1 ] && { echo "$_chave"; exit 0; }
if [ "$_enviar" = 1 ]; then
  _arv_head="$(suite_arvore_head)"
  _chave_head="$(suite_chave "$_arv_head" "$*")"
  if [ -n "$_arv_head" ] && [ "${PH2D_PORTAO_FORCAR:-0}" != "1" ] && grep -q "^$_chave_head	" "$_registo" 2>/dev/null; then
    echo "[suite-completa] ✓ a árvore que o push ENVIA (HEAD, $_arv_head) JÁ PASSOU a suíte completa:"
    grep "^$_chave_head	" "$_registo" | tail -1 | cut -f3- | sed 's/^/[suite-completa]   /'
    [ "$_arv_head" != "$_arvore" ] && echo "[suite-completa]   (o que está por comitar nesta pasta não vai no push e não foi testado)"
    exit 0
  fi
fi

if [ -n "$_arvore" ] && [ "${PH2D_PORTAO_FORCAR:-0}" != "1" ] && grep -q "^$_chave	" "$_registo" 2>/dev/null; then
  echo "[suite-completa] ✓ esta árvore ($_arvore) JÁ PASSOU a suíte completa:"
  grep "^$_chave	" "$_registo" | tail -1 | cut -f3- | sed 's/^/[suite-completa]   /'
  echo "[suite-completa]   (PH2D_PORTAO_FORCAR=1 corre na mesma)"
  exit 0
fi

# shellcheck source=scripts/tempos-teste-lib.sh
source scripts/tempos-teste-lib.sh
_tt="$(mktemp)"; _t0="$(tt_agora)"; _l0="$(tt_load)"
# ⛔ Cada termo dos `filter =` do nextest.toml casa ≥ 1 teste real (29/10: um ficheiro partido mudou o nome
# de um teste, o prazo de 10 min deixou de se lhe aplicar e ele morreu aos 180 s). A lista reaproveita a
# compilação do `ci-test` que a corrida a seguir usa.
set -o pipefail
cargo nextest list --workspace --cargo-profile ci-test --message-format json 2> >(tee -a "$_tt" >&2) \
  | python3 scripts/nextest-filtros-casam.py -
_lst=("${PIPESTATUS[@]}")
if [ "${_lst[0]}" != 0 ]; then
  echo "[suite-completa] ✗ a workspace NÃO COMPILOU no ci-test (nextest list exit ${_lst[0]}) — nenhum teste correu" >&2
  tt_registar suite-completa workspace 0 "$(tt_compilar_de "$_tt")" "$(tt_dif "$_t0" "$(tt_agora)")" "$_l0" 2
  rm -f "$_tt"; exit 2
fi
[ "${_lst[1]}" = 0 ] || { echo "[suite-completa] ✗ um filtro do nextest.toml casa zero testes" >&2; rm -f "$_tt"; exit 1; }
cargo nextest run --workspace --cargo-profile ci-test "$@" 2> >(tee -a "$_tt" >&2)
_saida=$?
tt_registar suite-completa workspace 0 "$(tt_compilar_de "$_tt")" "$(tt_dif "$_t0" "$(tt_agora)")" "$_l0" "$_saida"
rm -f "$_tt"
# A árvore é relida: se alguém escreveu nela durante a corrida, a chave de antes já não é a que passou.
if [ "$_saida" = 0 ] && [ -n "$_arvore" ] && [ "$(suite_arvore)" = "$_arvore" ]; then
  printf '%s\t%s\t%s · %s · HEAD %s\n' "$_chave" "$_arvore" "$(date '+%F %R')" \
    "$(git branch --show-current 2>/dev/null)" "$(git rev-parse --short HEAD)" >>"$_registo"
  echo "[suite-completa] ✓ verde — árvore $_arvore registada em $_registo"
elif [ "$_saida" = 0 ]; then
  echo "[suite-completa] ⚠ verde, mas a árvore mudou durante a corrida — NÃO registada"
fi
# ⭐ Item G: a lista do nível lento acompanha a ÚLTIMA suíte verde (o junit que esta corrida escreveu).
_lista=.config/testes-lentos.tsv
_junit="${CARGO_TARGET_DIR:-target}/nextest/default/junit.xml"
if [ "$_saida" = 0 ] && [ -f "$_junit" ]; then
  _antes="$(grep -v '^#' "$_lista" 2>/dev/null | cut -f1,2)"
  python3 scripts/testes-lentos.py --regerar "$_junit"
  [ "$(grep -v '^#' "$_lista" | cut -f1,2)" != "$_antes" ] \
    && echo "[suite-completa] ⚠ o nível lento MUDOU — comite $_lista"
fi
exit "$_saida"
