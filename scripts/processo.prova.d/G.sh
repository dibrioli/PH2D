# shellcheck shell=bash
# Item G (comissão 29/10): o nível «lento» é MEDIDO do junit (`scripts/testes-lentos.py`) e sai do
# `nextest-impacted.sh` por omissão; um teste ≥ 60 s sem nível reprova.
echo "— G: o nível lento é medido, e um lento sem nível reprova —"
_tl() { python3 scripts/testes-lentos.py "$@"; }
_junit_g() {
  printf '<?xml version="1.0"?>\n<testsuites uuid="u-%s" timestamp="t" tests="4">\n<testsuite name="a">\n' "$1"
  printf '<testcase classname="ph2d-a::it" name="m::lento" time="%s"/>\n' "$2"
  printf '<testcase classname="ph2d-a" name="m::rapido" time="0.5"/>\n'
  printf '<testcase classname="ph2d-b" name="m::timeout" time="600.1"><failure type="test timeout"/></testcase>\n'
  printf '<testcase classname="ph2d-b" name="transform_determinism::golden" time="%s"/>\n' "$3"
  printf '</testsuite>\n</testsuites>\n'
}
_junit_g 1 90 1 >"$tmp/g1.xml"
_tl --regerar "$tmp/g1.xml" --saida "$tmp/g.tsv" >/dev/null
igual "a lista tem os ≥ 60 s (um timeout conta)" "$(grep -v '^#' "$tmp/g.tsv" | cut -f2 | tr '\n' ' ')" "m::lento m::timeout "
_junit_g 2 30 70 >"$tmp/g2.xml"
_tl --regerar "$tmp/g2.xml" --saida "$tmp/g.tsv" >/dev/null
igual "quem agora mede < 60 s sai; o que passou a ≥ 60 s entra" \
  "$(grep -v '^#' "$tmp/g.tsv" | cut -f2 | tr '\n' ' ')" "m::timeout transform_determinism::golden "
_tl --regerar "$tmp/g1.xml" "$tmp/g2.xml" --saida "$tmp/g.tsv" >/dev/null
igual "com vários junit vale o MÁXIMO" "$(grep -c -v '^#' "$tmp/g.tsv")" "3"
_junit_g 3 61 1 >"$tmp/g3.xml"
_tl --regerar "$tmp/g2.xml" --saida "$tmp/g.tsv" >/dev/null
_tl --confere "$tmp/g3.xml" --lista "$tmp/g.tsv" >/dev/null
igual "um teste lento sem nível reprova (exit 1)" "$?" "1"
_tl --confere "$tmp/g2.xml" --lista "$tmp/g.tsv" >/dev/null
igual "a lista da própria medição passa (exit 0)" "$?" "0"
_tl --regerar "$tmp/g3.xml" --saida "$tmp/g.tsv" >/dev/null
_f="$(_tl --filtro --lista "$tmp/g.tsv" --menos-prefixo transform_determinism:: 2>/dev/null)"
igual "o filtro exclui o lento pelo binário e pelo nome exacto" "$_f" \
  "(binary_id(=ph2d-a::it) & test(=m::lento)) | (binary_id(=ph2d-b) & test(=m::timeout))"
_tl --regerar "$tmp/g2.xml" --saida "$tmp/g.tsv" >/dev/null
_f="$(_tl --filtro --lista "$tmp/g.tsv" --menos-prefixo transform_determinism:: 2>&1 >/dev/null)"
case "$_f" in *"REDE OBRIGATÓRIA"*) ok "a rede obrigatória volta ao nível rápido, e diz" ;; *) ko "a rede obrigatória ficou no nível lento" ;; esac
[ -f .config/testes-lentos.tsv ] && ok "a lista real existe" || ko "falta .config/testes-lentos.tsv"
_mao="$(grep -v '^#' .config/testes-lentos.tsv 2>/dev/null | grep -cvP '^[A-Za-z0-9_.:/-]+\t[^\t]+\t[0-9]+\.[0-9]{3}$')"
igual "a lista real não tem linha à mão (cada linha bate o formato gerado)" "$_mao" "0"
grep -q 'GERADO por scripts/testes-lentos.py' .config/testes-lentos.tsv 2>/dev/null \
  && ok "a lista real é a gerada (cabeçalho)" || ko "a lista real perdeu o cabeçalho gerado"
grep -q -- '--filtro --menos-prefixo transform_determinism::' scripts/nextest-impacted.sh && ok "o nextest-impacted corta o nível lento" \
  || ko "o nextest-impacted já não corta o nível lento"
grep -q 'testes-lentos.py --regerar' scripts/suite-completa.sh && ok "a suíte completa regera a lista" \
  || ko "a suíte completa já não regera a lista"
