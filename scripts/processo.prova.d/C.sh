# shellcheck shell=bash
# Item C (12/11): `collision-surface.sh` — a sonda dos espelhos do registo (`so_ecs.len() + K`) e
# qualquer sonda que lê «—» dos DOIS lados reprovam ALTO (SONDA CEGA), e a const acha-se pelo NOME
# no `src/` do crate quando muda de ficheiro. De 07/11 a 12/11 a regex casava e o `val` (só '=' ou
# ',') devolvia «—»: os espelhos de ph2d-render e ph2d-script liam cego com o caminho certo.
echo "— C: collision-surface, sondas cegas —"
_cs="$PWD/scripts/collision-surface.sh"
_linha() { grep -F "$1" | head -1 | sed -E 's/^ *//; s/  +/ /g'; }
# 1) a árvore real: o número do script é o da contagem à mão.
_k=$(grep -rhoE 'so_ecs\.len\(\) *\+ *[0-9]+' crates/ph2d-render/src | head -1 | grep -oE '[0-9]+$')
_out=$(bash "$_cs" 2>&1)
igual "ph2d-render lê K da árvore (= grep)" "$(echo "$_out" | _linha 'ph2d-render (espelho' | awk '{print $(NF-2)}')" "$_k"
igual "árvore real sem SONDA CEGA" "$(echo "$_out" | grep -c 'SONDA CEGA')" "0"
# 2) fixtura: a const MUDA de ficheiro dentro do crate ⇒ achada pela pasta, não cega.
_r="$tmp/cs-repo"; mkdir -p "$_r/crates/ph2d-render/src" "$_r/crates/ph2d-script/src"
printf 'assert_eq!(reg.len(), so_ecs.len() + 3);\n' >"$_r/crates/ph2d-render/src/registry.rs"
printf 'assert_eq!(reg.len(), so_ecs.len() + 4);\n' >"$_r/crates/ph2d-script/src/registry.rs"
git -C "$_r" init -q -b main && git -C "$_r" add -A && git -C "$_r" -c user.name=p -c user.email=p@p commit -qm base
git -C "$_r" mv -k crates/ph2d-render/src/registry.rs crates/ph2d-render/src/registro_espelho.rs
_out=$(cd "$_r" && bash "$_cs" 2>&1)
igual "const mudada de ficheiro: achada pelo nome no crate" "$(echo "$_out" | _linha 'ph2d-render (espelho')" "ph2d-render (espelho = ecs + K) 3 (base: 3)"
igual "o vizinho no sítio de sempre lê o seu K" "$(echo "$_out" | _linha 'ph2d-script (espelho')" "ph2d-script (espelho = ecs + K) 4 (base: 4)"
# 3) mutação: a const some (renomeada) aqui e na base ⇒ ✗ e SONDA CEGA, exit 2.
printf 'assert_eq!(reg.len(), so_ECS_RENOMEADA.len() + 3);\n' >"$_r/crates/ph2d-render/src/registro_espelho.rs"
git -C "$_r" add -A && git -C "$_r" -c user.name=p -c user.email=p@p commit -qm x
_out=$(cd "$_r" && bash "$_cs" 2>&1); _rc=$?
igual "const renomeada: a linha reprova com ✗" "$(echo "$_out" | _linha 'ph2d-render (espelho' | cut -c1-3)" "✗ p"
igual "o rodapé nomeia a sonda cega" "$(echo "$_out" | grep 'SONDA CEGA em:' | grep -c 'ph2d-render (espelho')" "1"
igual "sonda cega sai com 2" "$_rc" "2"
igual "tripla ausente dos dois lados também é cega" "$(echo "$_out" | grep 'SONDA CEGA em:' | grep -c 'tripla do gate')" "1"
