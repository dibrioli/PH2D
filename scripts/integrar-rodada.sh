#!/usr/bin/env bash
# integrar-rodada.sh — a RODADA de várias linhas numa só suíte (item A + B2 da comissão de velocidade,
# docs/DevOps/COMISSAO_VELOCIDADE_2026-10-29.md). Medido em 29/10: 5 linhas em série = ~9 h, 11 suítes
# de 21–31 min; a 1.ª de cada linha sempre vermelha, a 2.ª inteira só confirmava 1–7 curas.
#
#   bash scripts/integrar-rodada.sh [-C <worktree>] linha                         # UMA linha com o main ancestral: sem rebase, a seguir suite/falhas/b2
#   bash scripts/integrar-rodada.sh [-C <worktree>] cadeia <rodada> <linha>...   # rebase em CADEIA + check/censos/recontar por linha
#   bash scripts/integrar-rodada.sh [-C <worktree>] suite                         # UMA suíte completa no tip
#   bash scripts/integrar-rodada.sh [-C <worktree>] falhas [--bisecar]            # cada vermelho → a linha que o trouxe
#   bash scripts/integrar-rodada.sh [-C <worktree>] b2                            # depois das curas: só o que elas podem mudar
#   bash scripts/integrar-rodada.sh [-C <worktree>] ff [--aplicar]                # imprime (ou faz) o --ff-only do main
#
# Corre-se no worktree da rodada (`git worktree add <dir> -b integ/<rodada> main`). O estado vive no
# git-dir do worktree (`integ-rodada/`), FORA da árvore: dentro dela mudaria a chave da suíte verde.
# Os comandos pesados trocam-se por variável (a prova `processo.prova.d/A.sh` corre sem cargo):
#   PH2D_INTEG_CHECK · PH2D_INTEG_CENSOS · PH2D_INTEG_RECONTAR · PH2D_INTEG_SUITE · PH2D_INTEG_JUNIT
#   PH2D_INTEG_IMPACTO (env PRE TIP REDS → imprime as linhas de REDS que o diff PRE..TIP alcança)
#   PH2D_INTEG_TESTA   (env LISTA → corre esses testes por nome, imprime os que falham)
#   PH2D_INTEG_IMPACTADO · PH2D_INTEG_ARCH · PH2D_INTEG_MAIN (omissão: main)
set -uo pipefail

_aqui="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WT="$PWD"
if [ "${1:-}" = "-C" ]; then WT="$2"; shift 2; fi
WT="$(git -C "$WT" rev-parse --show-toplevel)" || exit 2
G() { git -C "$WT" "$@"; }
EST="$(cd "$WT" && cd "$(git rev-parse --git-dir)" && pwd)/integ-rodada"
mkdir -p "$EST"
MAIN="${PH2D_INTEG_MAIN:-main}"
REG="${PH2D_SUITES_VERDES:-$(cd "$WT" && cd "$(git rev-parse --git-common-dir)" && pwd)/ph2d-suites-verdes.tsv}"
# A ferramenta da PRÓPRIA árvore quando existe (é a lei dela); senão a deste script (ensaio sobre árvore antiga).
ferr() { if [ -f "$WT/scripts/$1" ]; then echo "$WT/scripts/$1"; else echo "$_aqui/$1"; fi; }
em_wt() { (cd "$WT" && eval "$1"); }
agora() { date +%s.%N; }
dif() { python3 -c "print(f'{$2-$1:.1f}')"; }
rodada() { cat "$EST/rodada" 2>/dev/null; }
# estado.tsv: ordem  linha  base  tip  commits  t_rebase  t_check  t_censos  t_recontar  veredito
linhas_feitas() { cut -f2 "$EST/estado.tsv" 2>/dev/null; }
chave_da_arvore() {
  (cd "$WT" && env $(compgen -e | grep '^PH2D_INTEG_' | sed 's/^/-u /') PH2D_NA_PORTA=1 bash "$(ferr suite-completa.sh)" --chave)
}

passo() { # nome comando log → exit do comando; imprime o tempo
  local t0 rc
  t0="$(agora)"
  em_wt "$2" >"$3" 2>&1; rc=$?
  T_PASSO="$(dif "$t0" "$(agora)")"
  [ "$rc" = 0 ] && echo "    ✓ $1 (${T_PASSO} s)" || { echo "    ✗ $1 (${T_PASSO} s, exit $rc) — $3"; tail -15 "$3" | sed 's/^/      /'; }
  return "$rc"
}

cmd_cadeia() {
  local r="$1"; shift
  [ $# -ge 1 ] || { echo "uso: cadeia <rodada> <linha>..." >&2; exit 2; }
  local ramo="integ/$r"
  if [ ! -s "$EST/inicio" ]; then
    [ "$(G branch --show-current)" = "$ramo" ] || { echo "✗ o worktree tem de estar em $ramo (git worktree add <dir> -b $ramo $MAIN)" >&2; exit 2; }
    G rev-parse HEAD >"$EST/inicio"; echo "$r" >"$EST/rodada"
  fi
  [ "$(rodada)" = "$r" ] || { echo "✗ este worktree tem a rodada «$(rodada)»" >&2; exit 2; }
  local ordem=0 l
  for l in "$@"; do
    ordem=$((ordem + 1))
    linhas_feitas | grep -qxF "$l" && { echo "▸ $l — já na cadeia"; continue; }
    local tmp="integ-tmp/$r/${l//\//-}" pre base n t0 t_reb
    echo "▸ $ordem/$# $l"
    if [ -d "$(G rev-parse --git-path rebase-merge)" ] || [ -d "$(G rev-parse --git-path rebase-apply)" ]; then
      echo "✗ rebase a meio: resolva pelos ESTÁGIOS (:1/:2/:3), git rebase --continue, e re-corra «cadeia»" >&2; exit 1
    fi
    pre="$(G rev-parse "$ramo")"
    if [ "$(G branch --show-current)" != "$tmp" ]; then
      base="$(G merge-base "$l" "$pre")"; n="$(G rev-list --count "$pre..$l")"
      G branch -f "$tmp" "$l" && G checkout -q "$tmp" || exit 2
      t0="$(agora)"
      # A saída INTEIRA do rebase fica gravada (o Mergiraf diz «Solved» sobre o que largou — 3b).
      if ! G rebase "$pre" >"$EST/$ordem-rebase.log" 2>&1; then
        echo "✗ conflito no rebase de $l — $EST/$ordem-rebase.log"; grep -E 'CONFLICT|error' "$EST/$ordem-rebase.log" | head
        echo "  resolva pelos estágios, git rebase --continue, depois re-corra a mesma «cadeia»"; exit 1
      fi
      t_reb="$(dif "$t0" "$(agora)")"
    else
      base="$(G merge-base "$l" "$pre")"; n="$(G rev-list --count "$base..$l")"; t_reb="manual"
    fi
    G checkout -q "$ramo" && G merge -q --ff-only "$tmp" && G branch -q -D "$tmp" || exit 2
    local tip; tip="$(G rev-parse HEAD)"
    # 3b: cada commit rebaseado contra o original — o range-diff fica para quem lê.
    G range-diff "$base..$l" "$pre..$tip" >"$EST/$ordem-range-diff.txt" 2>&1
    echo "    rebase: $n commits (${t_reb} s) → tip $(G rev-parse --short HEAD)"
    local rc=0 tc tz tr
    passo check "${PH2D_INTEG_CHECK:-bash $(ferr ph2d-run.sh) env CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets}" "$EST/$ordem-check.log" || rc=1; tc="$T_PASSO"
    passo censos "${PH2D_INTEG_CENSOS:-BASE=$pre bash $(ferr censos-da-arvore-combinada.sh)}" "$EST/$ordem-censos.log" || rc=1; tz="$T_PASSO"
    passo recontar "${PH2D_INTEG_RECONTAR:-python3 $(ferr recontar.py) --raiz $WT}" "$EST/$ordem-recontar.log" || rc=1; tr="$T_PASSO"
    local v=verde; [ "$rc" = 0 ] || v=vermelho
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$ordem" "$l" "$base" "$tip" "$n" "$t_reb" "$tc" "$tz" "$tr" "$v" >>"$EST/estado.tsv"
    if [ "$rc" != 0 ]; then
      echo "✗ vermelho no tip de $l — é a 1.ª árvore em que ele aparece: a cura é desta linha, NO commit rebaseado"
      echo "  que o causa (edit-rebase, §1.5.3 (a)); depois apague a linha 「$l」 de $EST/estado.tsv e re-corra."
      exit 1
    fi
  done
  echo "✓ cadeia verde ($(wc -l <"$EST/estado.tsv") linhas). Próximo: integrar-rodada.sh suite"
}

# ⭐ Uma linha só (o caminho normal desde 03/11): o `main` é ancestral, não há rebase nem cadeia; `linha` só
# deixa o estado que `suite`, `falhas`, `b2` e `ff` leem. Reds de 04/11: 4 × ~28 min de suíte repetida.
cmd_linha() {
  local l; l="$(G branch --show-current)"
  [ -n "$l" ] || { echo "✗ HEAD destacado: corra num ramo" >&2; exit 2; }
  G merge-base --is-ancestor "$MAIN" HEAD || { echo "✗ $MAIN não é ancestral de $l: rebase primeiro, ou use «cadeia»" >&2; exit 2; }
  rm -rf "$EST"; mkdir -p "$EST"
  G rev-parse "$MAIN" >"$EST/inicio"; echo "$l" >"$EST/rodada"; echo "$l" >"$EST/ramo"
  printf '1\t%s\t%s\t%s\t%s\tsem-rebase\t-\t-\t-\tverde\n' "$l" "$(G rev-parse "$MAIN")" "$(G rev-parse HEAD)" \
    "$(G rev-list --count "$MAIN..HEAD")" >"$EST/estado.tsv"
  echo "✓ linha $l sobre $MAIN ($(G rev-list --count "$MAIN..HEAD") commits). Próximo: integrar-rodada.sh suite"
}

cmd_suite() {
  [ -s "$EST/estado.tsv" ] || { echo "✗ corra «linha» ou «cadeia» primeiro" >&2; exit 2; }
  local rc t0
  G rev-parse HEAD >"$EST/suite-sha"; G rev-parse 'HEAD^{tree}' >"$EST/suite-arvore"
  t0="$(agora)"
  em_wt "${PH2D_INTEG_SUITE:-NO_FAIL_FAST=1 bash $(ferr suite-completa.sh)}" 2>&1 | tee "$EST/suite.log"; rc=${PIPESTATUS[0]}
  echo "$rc" >"$EST/suite-rc"
  local j="${PH2D_INTEG_JUNIT:-$WT/${CARGO_TARGET_DIR:-target}/nextest/default/junit.xml}"
  rm -f "$EST/suite-junit.xml"; [ -f "$j" ] && cp "$j" "$EST/suite-junit.xml"
  echo "[integrar-rodada] suíte exit $rc em $(dif "$t0" "$(agora)") s — próximo: integrar-rodada.sh falhas"
  return "$rc"
}

vermelhos_do_junit() { # binário<TAB>teste, um por falha (as flaky que passaram ficam de fora)
  python3 - "$1" <<'PY'
import sys, xml.etree.ElementTree as ET
for s in ET.parse(sys.argv[1]).getroot().iter("testsuite"):
    for c in s.iter("testcase"):
        if c.find("failure") is not None or c.find("error") is not None:
            print(f"{s.get('name')}\t{c.get('name')}")
PY
}
filtro_de() { awk -F'\t' '{printf "%s(binary_id(=%s) & test(=%s))", (NR>1?" | ":""), $1, $2}' "$1"; }

impacto_omissao() { # PRE TIP REDS: os vermelhos cujo binário o diff PRE..TIP pode mudar (a lei do nextest-impacted)
  local pk ex
  pk="$(git -C "$WT" diff --name-only "$PRE" "$TIP" | (cd "$WT" && python3 -c '
import json, os, subprocess, sys
meta = json.loads(subprocess.run(["cargo","metadata","--no-deps","--format-version","1"],capture_output=True,text=True,check=True).stdout)
root = meta["workspace_root"]
pk = sorted(((os.path.relpath(os.path.dirname(p["manifest_path"]), root), p["name"]) for p in meta["packages"]), key=lambda x: -len(x[0]))
out = set()
for f in (l.strip() for l in sys.stdin if l.strip()):
    for d, n in pk:
        if d != "." and (f == d or f.startswith(d + "/")): out.add(n); break
    else:
        if f in ("Cargo.toml", "Cargo.lock") or f.startswith(".cargo/"): out |= {n for _, n in pk}
print(" ".join(sorted(out)))'))"
  [ -n "$pk" ] || return 0
  ex="$(cd "$WT" && python3 "$(ferr impacto-preciso.py)" $pk)" || return 2
  (cd "$WT" && cargo nextest list --workspace --cargo-profile ci-test -E "($ex) & ($(filtro_de "$REDS"))" --message-format json 2>/dev/null) \
    | python3 -c '
import json, sys
d = json.load(sys.stdin)
for b, s in d["rust-suites"].items():
    for t, c in s["testcases"].items():
        if c["filter-match"]["status"] == "matches": print(f"{b}\t{t}")'
}
testa_omissao() { # LISTA → corre por nome, imprime os que falham
  local j="${PH2D_INTEG_JUNIT:-$WT/${CARGO_TARGET_DIR:-target}/nextest/default/junit.xml}"
  rm -f "$j"
  (cd "$WT" && bash "$(ferr ph2d-run.sh)" cargo nextest run --workspace --cargo-profile ci-test --no-fail-fast -E "$(filtro_de "$LISTA")") >&2
  [ -f "$j" ] && vermelhos_do_junit "$j"
}
impacto() { if [ -n "${PH2D_INTEG_IMPACTO:-}" ]; then (cd "$WT" && PRE="$PRE" TIP="$TIP" REDS="$REDS" eval "$PH2D_INTEG_IMPACTO"); else impacto_omissao; fi; }
testa() { if [ -n "${PH2D_INTEG_TESTA:-}" ]; then (cd "$WT" && LISTA="$LISTA" eval "$PH2D_INTEG_TESTA"); else testa_omissao; fi; }

# ⭐ A ATRIBUIÇÃO, sem correr a suíte N vezes: um vermelho nasce no 1.º tip da cadeia em que falha, e o
# veredito de um teste só pode mudar num tip cujo diff (tip anterior → tip) alcança o binário dele (a lei
# do `impacto-preciso.py`: mesmo fecho de dependências ⇒ mesmo binário). ⇒ (1) por linha, quais vermelhos
# o diff dela alcança (uma `nextest list`, sem compilar — os binários são os da suíte); (2) um candidato =
# é dele; zero = nenhuma linha o mudou (carga §3d, ou já vinha do main); vários = `--bisecar` corre SÓ
# esses testes nos tips candidatos, por ordem, e pára no 1.º vermelho (k candidatos ⇒ ≤ k−1 corridas).
cmd_falhas() {
  local bisecar=0; [ "${1:-}" = "--bisecar" ] && bisecar=1
  [ -f "$EST/suite-rc" ] || { echo "✗ corra «suite» primeiro" >&2; exit 2; }
  REDS="$EST/vermelhos.tsv"; : >"$REDS"
  [ -f "$EST/suite-junit.xml" ] && vermelhos_do_junit "$EST/suite-junit.xml" >"$REDS"
  if [ ! -s "$REDS" ]; then
    [ "$(cat "$EST/suite-rc")" = 0 ] && { echo "✓ a suíte não deixou vermelhos"; : >"$EST/falhas.tsv"; exit 0; }
    echo "✗ a suíte saiu $(cat "$EST/suite-rc") e o junit não tem falhas (não compilou? filtro-casa?) — $EST/suite.log" >&2; exit 1
  fi
  local pre ordem l base tip _r
  pre="$(cat "$EST/inicio")"
  declare -A cand=()
  while IFS=$'\t' read -r ordem l base tip _r; do
    PRE="$pre" TIP="$tip" impacto >"$EST/impacto-$ordem.tsv" || { echo "✗ o impacto de $l falhou" >&2; exit 2; }
    while IFS= read -r t; do cand["$t"]="${cand["$t"]:-}${cand["$t"]:+ }$ordem"; done <"$EST/impacto-$ordem.tsv"
    pre="$tip"
  done <"$EST/estado.tsv"
  linha_de() { awk -F'\t' -v o="$1" '$1==o{print $2}' "$EST/estado.tsv"; }
  tip_de() { awk -F'\t' -v o="$1" '$1==o{print $4}' "$EST/estado.tsv"; }
  : >"$EST/falhas.tsv"
  local t c atribui_a como amb=()
  while IFS= read -r t; do
    c="${cand["$t"]:-}"
    case "$c" in
      "") atribui_a="-"; como="nenhum diff da cadeia o alcança: carga (§3d) ou já vinha do main" ;;
      *" "*) atribui_a="?"; como="candidatas: $c"; amb+=("$t") ;;
      *) atribui_a="${c%% *}"; como="o único diff que o alcança" ;;
    esac
    printf '%s\t%s\t%s\n' "$atribui_a" "$t" "$como" >>"$EST/falhas.tsv"
  done <"$REDS"
  if [ "$bisecar" = 1 ] && [ ${#amb[@]} -gt 0 ]; then
    local volta; volta="$(G branch --show-current)"
    [ -z "$(G status --porcelain)" ] || { echo "✗ --bisecar precisa da árvore limpa" >&2; exit 2; }
    declare -A feito=()
    local ordens; ordens="$(for t in "${amb[@]}"; do echo "${cand["$t"]}"; done | tr ' ' '\n' | sort -n -u)"
    for ordem in $ordens; do
      LISTA="$EST/bisect-lista.tsv"; : >"$LISTA"
      for t in "${amb[@]}"; do
        [ -n "${feito["$t"]:-}" ] && continue
        local cs=" ${cand["$t"]} " ult="${cand["$t"]##* }"
        [[ "$cs" == *" $ordem "* ]] || continue
        if [ "$ordem" = "$ult" ]; then feito["$t"]="$ordem"; continue; fi  # o último candidato não precisa de corrida
        printf '%s\n' "$t" >>"$LISTA"
      done
      [ -s "$LISTA" ] || continue
      G checkout -q --detach "$(tip_de "$ordem")" || exit 2
      echo "▸ bisect: $(wc -l <"$LISTA") teste(s) no tip de $(linha_de "$ordem")"
      while IFS= read -r t; do feito["$t"]="$ordem"; done < <(testa)
    done
    G checkout -q "$volta"
    local tmpf; tmpf="$(mktemp)"
    while IFS=$'\t' read -r atribui_a b n como; do
      t="$b"$'\t'"$n"
      if [ "$atribui_a" = "?" ] && [ -n "${feito["$t"]:-}" ]; then atribui_a="${feito["$t"]}"; como="1.º tip vermelho entre ($como)"; fi
      printf '%s\t%s\t%s\n' "$atribui_a" "$t" "$como" >>"$tmpf"
    done <"$EST/falhas.tsv"
    mv "$tmpf" "$EST/falhas.tsv"
  fi
  echo "linha	binário	teste	como"
  while IFS=$'\t' read -r atribui_a b n como; do
    case "$atribui_a" in -|"?") l="$atribui_a" ;; *) l="$(linha_de "$atribui_a")" ;; esac
    printf '%s\t%s\t%s\t%s\n' "$l" "$b" "$n" "$como"
  done <"$EST/falhas.tsv" | sort | tee "$EST/falhas-por-linha.tsv"
  grep -q '^?' "$EST/falhas-por-linha.tsv" && echo "⚠ há ambíguos: integrar-rodada.sh falhas --bisecar"
  echo "Cure em BLOCO (um commit por linha atribuída), depois: integrar-rodada.sh b2"
}

# ⭐ B2 — depois das curas, só o que elas podem mudar. O argumento (porque o registo é honesto): um teste
# FORA do impacto do diff «tip da suíte → árvore curada» tem o MESMO binário e o mesmo veredito que teve
# na suíte (verde, senão estaria nas falhas); os que estão DENTRO correm todos (PH2D_NIVEL=tudo inclui os
# lentos); as falhas correm por nome; e os portões que VARREM a árvore (censos HR-15, tectos de LOC,
# architecture_*) não dependem do grafo — correm SEMPRE, e cada um tem de provar que correu (≥ 1 teste).
cmd_b2() {
  [ -f "$EST/suite-sha" ] || { echo "✗ corra «suite» primeiro" >&2; exit 2; }
  [ -f "$EST/falhas.tsv" ] || { echo "✗ corra «falhas» primeiro" >&2; exit 2; }
  local s; s="$(cat "$EST/suite-sha")"
  [ -z "$(G status --porcelain --untracked-files=no)" ] || { echo "✗ comite as curas antes do B2" >&2; exit 2; }
  local k0; k0="$(chave_da_arvore)"; local rc=0
  echo "▸ B2 desde a suíte em $(G rev-parse --short "$s") ($(G rev-list --count "$s..HEAD") commit(s) de cura)"
  if [ -s "$EST/falhas.tsv" ]; then
    LISTA="$EST/b2-falhas.tsv"; cut -f2,3 "$EST/falhas.tsv" >"$LISTA"
    testa >"$EST/b2-ainda-vermelhos.tsv" || rc=1
    [ -s "$EST/b2-ainda-vermelhos.tsv" ] && { echo "    ✗ ainda vermelhos:"; sed 's/^/      /' "$EST/b2-ainda-vermelhos.tsv"; rc=1; } \
      || echo "    ✓ as $(wc -l <"$LISTA") falha(s) da suíte, por nome"
  fi
  passo impactado "${PH2D_INTEG_IMPACTADO:-BASE=$s PH2D_NIVEL=tudo PH2D_SUITE=fecho NO_FAIL_FAST=1 bash $(ferr nextest-impacted.sh)}" "$EST/b2-impactado.log" || rc=1
  passo censos "${PH2D_INTEG_CENSOS:-BASE=$(cat "$EST/inicio") bash $(ferr censos-da-arvore-combinada.sh)}" "$EST/b2-censos.log" || rc=1
  grep -q 'censos correram ✓' "$EST/b2-censos.log" || { echo "    ✗ o censo global NÃO provou que correu (falta «censos correram ✓»)"; rc=1; }
  passo arch "${PH2D_INTEG_ARCH:-bash $(ferr ph2d-run.sh) cargo nextest run --workspace --cargo-profile ci-test --no-fail-fast -E 'test(/^architecture_/)'}" "$EST/b2-arch.log" || rc=1
  grep -qE '[1-9][0-9]* tests? run' "$EST/b2-arch.log" || { echo "    ✗ os architecture_* NÃO provaram que correram (zero testes)"; rc=1; }
  passo recontar "${PH2D_INTEG_RECONTAR:-python3 $(ferr recontar.py) --raiz $WT}" "$EST/b2-recontar.log" || rc=1
  [ "$rc" = 0 ] || { echo "✗ B2 vermelho — NADA registado"; exit 1; }
  local k1; k1="$(chave_da_arvore)"
  [ "$k0" = "$k1" ] || { echo "✗ a árvore mudou durante o B2 — NADA registado"; exit 1; }
  printf '%s\t%s\t%s · %s · HEAD %s · B2 desde a árvore %s (suíte em %s)\n' "$k1" "$(G rev-parse 'HEAD^{tree}')" \
    "$(date '+%F %R')" "$(G branch --show-current)" "$(G rev-parse --short HEAD)" "$(cat "$EST/suite-arvore")" "$(G rev-parse --short "$s")" >>"$REG"
  echo "✓ B2 verde — árvore registada em $REG (o ship.sh salta a suíte com ela). Próximo: integrar-rodada.sh ff"
}

cmd_ff() {
  local r ramo prim; r="$(rodada)"; ramo="$(cat "$EST/ramo" 2>/dev/null || echo "integ/$r")"
  [ -n "$r" ] || { echo "✗ sem rodada neste worktree" >&2; exit 2; }
  G merge-base --is-ancestor "$MAIN" "$ramo" || { echo "✗ $MAIN andou e não está em $ramo: re-corra a cadeia sobre ele" >&2; exit 1; }
  grep -q "^$(chave_da_arvore)	" "$REG" 2>/dev/null || { echo "✗ a árvore de $ramo NÃO está verde no registo ($REG): suite + b2 primeiro" >&2; exit 1; }
  prim="$(G worktree list --porcelain | sed -n '1s/^worktree //p')"
  echo "git -C $prim merge --ff-only $ramo"
  if [ "${1:-}" = "--aplicar" ]; then
    [ "$(git -C "$prim" branch --show-current)" = "$MAIN" ] || { echo "✗ o primário não está em $MAIN" >&2; exit 1; }
    git -C "$prim" merge --ff-only "$ramo"
  else echo "(não aplicado — só com --aplicar, por ordem do Enio)"; fi
}

sub="${1:-}"; shift || true
case "$sub" in
  linha) cmd_linha "$@" ;;
  cadeia) cmd_cadeia "$@" ;;
  suite) cmd_suite "$@" ;;
  falhas) cmd_falhas "$@" ;;
  b2) cmd_b2 "$@" ;;
  ff) cmd_ff "$@" ;;
  *) sed -n '2,20p' "${BASH_SOURCE[0]}"; exit 2 ;;
esac
