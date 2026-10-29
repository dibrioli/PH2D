# shellcheck shell=bash
# Item D (comissão 29/10): recontagens automáticas — `scripts/recontar.py` (ADR, registos, cenas, schema).
echo "— D: recontar —"
# 1. A árvore real: só se exige que o recontar CORRA e diga o veredito (um ✗ real é achado, não falha
#    desta prova — ela prova a régua, e a régua é provada nas fixturas abaixo; o ✗ real reprova
#    na linha «recontagens» do `ship.sh`).
saida="$(python3 scripts/recontar.py 2>&1)"; rc=$?
case "$saida" in *"recontar: "*) ok "corre na árvore real (exit $rc, $(grep -c '✓' <<<"$saida") ✓ · $(grep -c '  ✗' <<<"$saida") ✗)" ;;
  *) ko "o recontar não deu veredito na árvore real" ;; esac

# 2. Fixtura ADR: limpa = verde; dois `0007-*` (não-amendment) = vermelho; o amendment não conta.
mkdir -p "$tmp/D/docs/architecture/decisions"
: >"$tmp/D/docs/architecture/decisions/0006-a.md"
: >"$tmp/D/docs/architecture/decisions/0007-b.md"
: >"$tmp/D/docs/architecture/decisions/0007-amendment-1.md"
python3 scripts/recontar.py --raiz "$tmp/D" --apenas adr >/dev/null; igual "ADR limpo (amendment não duplica) = verde" "$?" "0"
igual "--proximo dá o ADR livre" "$(python3 scripts/recontar.py --raiz "$tmp/D" --apenas adr --proximo | awk '/ADR livre/{print $3}')" "0008"
: >"$tmp/D/docs/architecture/decisions/0007-c.md"
python3 scripts/recontar.py --raiz "$tmp/D" --apenas adr >/dev/null; igual "ADR duplicado = vermelho" "$?" "1"

# 3. Fixtura registo: as fontes reais copiadas, verde; o literal do ecs escrito errado = vermelho.
for f in crates/ph2d-ecs/src/scene/registry.rs crates/ph2d-ecs/src/scene/registry_camera.rs \
         crates/ph2d-ecs/src/scene/registry_tests.rs crates/ph2d-render/src/registry.rs \
         crates/ph2d-script/src/registry.rs; do
  mkdir -p "$tmp/D/$(dirname "$f")"; cp "$f" "$tmp/D/$f"
done
python3 scripts/recontar.py --raiz "$tmp/D" --apenas registos >/dev/null; igual "registos copiados = verde" "$?" "0"
t="$tmp/D/crates/ph2d-ecs/src/scene/registry_tests.rs"
n="$(grep -oE 'assert_eq!\(reg\.len\(\), *[0-9]+\)' "$t" | head -1 | grep -oE '[0-9]+\)$' | tr -d ')')"
igual "a agulha do literal casa 1×" "$(grep -c "assert_eq!(reg.len(), $n)" "$t")" "1"
sed -i "s/assert_eq!(reg.len(), $n)/assert_eq!(reg.len(), $((n + 1)))/" "$t"
python3 scripts/recontar.py --raiz "$tmp/D" --apenas registos >/dev/null; igual "registo escrito errado = vermelho" "$?" "1"

# 4. Fixtura cenas: o roteador do Quadro + uma linha do CLAUDE.md com o tecto dele; escrito +1 = vermelho.
mkdir -p "$tmp/D/crates/ph2d-app-board/src"; cp crates/ph2d-app-board/src/smoke.rs "$tmp/D/crates/ph2d-app-board/src/"
m="$(python3 scripts/recontar.py --apenas cenas --proximo | awk -F'[= ]' '/^  PH2D_BOARD_SMOKE=/{print $4 - 1}')"
echo "- Smoke \`PH2D_BOARD_SMOKE=1..$m\`." >"$tmp/D/CLAUDE.md"
python3 scripts/recontar.py --raiz "$tmp/D" --apenas cenas >/dev/null; igual "cenas = roteador ($m) = verde" "$?" "0"
echo "- Smoke \`PH2D_BOARD_SMOKE=1..$((m + 1))\`." >"$tmp/D/CLAUDE.md"
python3 scripts/recontar.py --raiz "$tmp/D" --apenas cenas >/dev/null; igual "cena escrita além do roteador = vermelho" "$?" "1"
