# shellcheck shell=bash
# Item A + B2 (comissão 29/10): a rodada em cadeia (`scripts/integrar-rodada.sh`), sem cargo.
echo "— A: a rodada em cadeia, a atribuição dos vermelhos, o B2 e o ff —"
_a="$tmp/a"
_gi() { git -C "$_a" -c user.email=p@p -c user.name=p "$@"; }
git init -q -b main "$_a"
mkdir -p "$_a/pa" "$_a/pb" "$_a/scripts" && echo 0 >"$_a/pa/x" && echo 0 >"$_a/pb/y"
cp scripts/suite-completa.sh "$_a/scripts/"
_gi add -A && _gi commit -q -m base
_gi checkout -q -b line/a && echo a >"$_a/pa/x" && _gi commit -q -am a
_gi checkout -q main && _gi checkout -q -b line/b && echo b >"$_a/pb/y" && _gi commit -q -am b
_gi checkout -q main && _gi checkout -q -b integ/r
# Os falsos: o impacto alcança o vermelho do pacote cujo directório o diff toca (e `glob` em todo o diff);
# o teste `glob t_g` falha quando pa/x é «a» (a linha a trouxe-o), `pa t_a` idem, `pz t_z` ninguém o toca.
cat >"$tmp/impacto.sh" <<'SH'
while IFS=$'\t' read -r b t; do p="${b%%::*}"
  if [ "$p" = glob ] || git diff --name-only "$PRE" "$TIP" | grep -q "^$p/"; then printf '%s\t%s\n' "$b" "$t"; fi
done <"$REDS"
SH
cat >"$tmp/testa.sh" <<'SH'
while IFS=$'\t' read -r b t; do case "$b" in glob::*|pa::*) [ "$(cat pa/x)" = a ] && printf '%s\t%s\n' "$b" "$t" ;; esac; done <"$LISTA"
SH
cat >"$tmp/junit.xml" <<'XML'
<testsuites><testsuite name="pa::it"><testcase name="t_a"><failure/></testcase><testcase name="t_ok"/></testsuite>
<testsuite name="glob::it"><testcase name="t_g"><failure/></testcase></testsuite>
<testsuite name="pz::it"><testcase name="t_z"><failure/></testcase></testsuite></testsuites>
XML
export PH2D_SUITES_VERDES="$tmp/a-verdes.tsv"
_ir() {
  PH2D_INTEG_CHECK=true PH2D_INTEG_CENSOS="${_censos:-echo 'controlo do filtro: 1 de 1 censos correram ✓'}" PH2D_INTEG_RECONTAR=true \
  PH2D_INTEG_SUITE=false PH2D_INTEG_JUNIT="$tmp/junit.xml" PH2D_INTEG_IMPACTO="bash $tmp/impacto.sh" \
  PH2D_INTEG_TESTA="bash $tmp/testa.sh" PH2D_INTEG_IMPACTADO=true PH2D_INTEG_ARCH="echo '3 tests run'" \
  bash "${_script:-scripts/integrar-rodada.sh}" -C "$_a" "$@"
}
_ir cadeia r line/a line/b >"$tmp/a-cadeia.log" 2>&1
_est="$(git -C "$_a" rev-parse --absolute-git-dir)/integ-rodada"
igual "a cadeia regista um tip por linha" "$(cut -f2 "$_est/estado.tsv" | tr '\n' ' ')" "line/a line/b "
igual "o tip da última linha é o HEAD da rodada" "$(cut -f4 "$_est/estado.tsv" | tail -1)" "$(git -C "$_a" rev-parse HEAD)"
[ "$(cut -f4 "$_est/estado.tsv" | sort -u | wc -l)" = 2 ] && ok "os dois tips são distintos" || ko "os tips coincidem"
_ir suite >/dev/null 2>&1
_ir falhas --bisecar >"$tmp/a-falhas.log" 2>&1
_de() { awk -F'\t' -v t="$1" '$3==t{print $1}' "$_est/falhas-por-linha.tsv"; }
igual "o vermelho de um só diff vai à linha dele" "$(_de t_a)" "line/a"
igual "o vermelho que todos alcançam vai ao 1.º tip em que falha (bisect)" "$(_de t_g)" "line/a"
igual "o vermelho que ninguém alcança não acusa nenhuma linha" "$(_de t_z)" "-"
# ⛔ Mutação vista vermelha: «atribui sempre à última linha» tem de mudar o veredito desta prova.
python3 - "$tmp/mutado.sh" <<'PY'
import sys
s = open("scripts/integrar-rodada.sh").read()
a = 'atribui_a="${c%% *}"; como="o único diff'
assert s.count(a) == 1, "âncora da mutação"
open(sys.argv[1], "w").write(s.replace(a, 'atribui_a="$(tail -1 "$EST/estado.tsv" | cut -f1)"; como="o único diff'))
PY
_script="$tmp/mutado.sh" _ir falhas >/dev/null 2>&1
[ "$(_de t_a)" != "line/a" ] && ok "mutação «sempre a última linha» vista vermelha" || ko "a mutação passou despercebida"
_ir falhas --bisecar >/dev/null 2>&1
_ir ff >"$tmp/a-ff0.log" 2>&1 && ko "o ff aceitou uma árvore sem verde" || ok "o ff recusa a árvore sem verde no registo"
# A cura: pa/x deixa de ser «a» — o B2 corre só as falhas + o impacto + os globais.
echo a-curado >"$_a/pa/x" && _gi commit -q -am cura
_censos=true _ir b2 >"$tmp/a-b2.log" 2>&1 && ko "o B2 registou sem o censo global provar que correu" \
  || ok "o B2 recusa registar sem o censo global"
[ -s "$PH2D_SUITES_VERDES" ] && ko "o B2 recusado escreveu no registo" || ok "o B2 recusado não escreve no registo"
_ir b2 >"$tmp/a-b2.log" 2>&1 && ok "o B2 verde regista" || ko "o B2 verde falhou: $(tail -3 "$tmp/a-b2.log")"
grep -q "B2 desde a árvore" "$PH2D_SUITES_VERDES" && ok "o registo diz «B2 desde a árvore da suíte»" || ko "o registo não diz B2"
_m0="$(git -C "$_a" rev-parse main)"
_ir ff >"$tmp/a-ff.log" 2>&1
igual "o ff sem --aplicar só imprime" "$(git -C "$_a" rev-parse main)" "$_m0"
grep -q "merge --ff-only integ/r" "$tmp/a-ff.log" && ok "o ff imprime o comando" || ko "o ff não imprimiu o comando"
unset PH2D_SUITES_VERDES _censos _script
