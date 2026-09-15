#!/usr/bin/env bash
# Provas de mutação do REPORT DO DONO de 2026-09-15 — *«a simulação não funciona. Nada se move.
# Para o Hero ser visível deve ficar abaixo na Hierarchy»* (line/components).
#
# Dois defeitos, e os dois tinham a suíte inteira verde por cima:
#
#   A) a pergunta «quem lê o teclado?» tinha DUAS respostas, e a que a shell usa não conhecia o
#      mover de vista de cima ⇒ intenção zero, e a fita nem gravava (contagem de players = 0);
#   B) a varredura foundational que numera as raízes congelava a ordem por `to_bits()`, que no bevy
#      é a ordem de criação INVERTIDA ⇒ o chão (1.ª raiz) desenhava por cima de tudo.
#
# ⚠️ As 24 provas da wave original entravam pelo canal interno da ponte e ficavam verdes sobre os
# dois. Estas entram pela PORTA DO PRODUTO.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 2
export LC_ALL=C
# ⛔ **`S` é a pasta dos LOGS.** Um alias de ficheiro chamado `S` no corpo faz o arnês escrever
# `…/tags.rs/1-controlo.log` e TODOS os controlos saem «inválidos» de uma vez — falha alto, mas
# o sintoma não aponta para a causa. Os aliases do corpo usam nomes de DUAS letras.
S=${PH2D_MUT_DIR:-$(mktemp -d)}
mkdir -p "$S/bak"
echo "logs em $S"
declare -a BACKED=()

restore_all() {
  for f in "${BACKED[@]:-}"; do
    [ -n "$f" ] || continue
    local b="$S/bak/$(echo "$f" | tr '/' '_')"
    [ -f "$b" ] && cp "$b" "$f" && touch "$f"
  done
}
trap restore_all EXIT INT TERM

backup() {
  local f=$1 b="$S/bak/$(echo "$1" | tr '/' '_')"
  if [ ! -f "$b" ]; then cp "$f" "$b"; BACKED+=("$f"); fi
}

replace() {
  python3 - "$1" "$2" "$3" <<'PY'
import sys
p, old, new = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(p, encoding="utf-8").read()
n = s.count(old)
if n != 1:
    print(f"ANCORA {n}x em {p}: {old!r}")
    sys.exit(3)
open(p, "w", encoding="utf-8").write(s.replace(old, new))
PY
}

corre() { # crate alvo filtro log -> "passed failed" ou "ERRO"
  local crate=$1 target=$2 filter=$3 log=$4
  cargo test -q -p "$crate" $target -- "$filter" >"$log" 2>&1
  local p f
  p=$(grep -oE '[0-9]+ passed' "$log" | awk '{s+=$1} END {print s+0}')
  f=$(grep -oE '[0-9]+ failed' "$log" | awk '{s+=$1} END {print s+0}')
  if ! grep -q 'test result' "$log"; then echo "ERRO"; else echo "$p $f"; fi
}

N=0
mutacao() { # nome ficheiro velho novo crate alvo filtro
  local nome=$1 file=$2 old=$3 new=$4 crate=$5 target=$6 filter=$7
  N=$((N+1))
  local ctl mut
  ctl=$(corre "$crate" "$target" "$filter" "$S/$N-controlo.log")
  if [ "$ctl" = "ERRO" ] || [ "${ctl%% *}" -lt 1 ] || [ "${ctl##* }" -ne 0 ]; then
    echo "✗ $nome — CONTROLO inválido ($ctl) · filtro «$filter»"; return
  fi
  backup "$file"
  if ! replace "$file" "$old" "$new"; then
    echo "✗ $nome — âncora não casou"; return
  fi
  mut=$(corre "$crate" "$target" "$filter" "$S/$N-mutante.log")
  cp "$S/bak/$(echo "$file" | tr '/' '_')" "$file" && touch "$file"
  if [ "$mut" = "ERRO" ]; then
    echo "? $nome — o mutante NÃO COMPILOU (ver $S/$N-mutante.log)"
  elif [ "${mut##* }" -ge 1 ]; then
    echo "✓ $nome — SANGROU (controlo $ctl · mutante $mut)"
  else
    echo "✗ $nome — SOBREVIVEU (controlo $ctl · mutante $mut)"
  fi
}

KD=crates/ph2d-physics-ecs/src/keyboard_driven.rs
DP=crates/ph2d-app-physics/src/bridge/dispatch.rs
TP=crates/ph2d-physics-ecs/src/bridge/tape.rs
RO=crates/ph2d-ecs/src/root_order.rs

echo "── A) o dedo do teclado chega ao mover de vista de cima ──────────────────"

# ⚠️ **A mutação é o CÓDIGO DE ANTES DO REPORT**, à letra: a varredura só via o platformer.
mutacao "A1 a varredura esquece o mover de vista de cima" "$KD" \
  'if let Some(mut q) = world.try_query::<(Entity, &TopDownPlayer)>() {' \
  'if false { let mut q = world.try_query::<(Entity, &TopDownPlayer)>().unwrap();' \
  ph2d-app-physics --lib topdown_finger_tests

# O CONTROLO do «motor puro» tem de ser vivo: sem o guarda, `default_controls = false` anda.
mutacao "A2 o guarda dos controlos de fabrica evapora" "$KD" \
  'if cfg.default_controls && world.get::<PlatformPlayer>(e).is_none() {' \
  'if world.get::<PlatformPlayer>(e).is_none() {' \
  ph2d-app-physics --lib topdown_finger_tests

# ⭐ E a ENTREGA tem de ler a porta: um `try_query` escrito à mão aqui é o defeito original.
mutacao "A3 a entrega volta a varrer so' o platformer" "$DP" \
  'ph2d_physics_ecs::for_each_keyboard_driven(sim.world(), |entity| {' \
  'sim.world().try_query::<(Entity, &ph2d_physics_ecs::PlatformPlayer)>().into_iter().flat_map(|mut q| q.iter(sim.world()).map(|(e, _)| e).collect::<Vec<_>>()).for_each(|entity| {' \
  ph2d-app-physics --lib topdown_finger_tests

# ⚠️ E a metade do REPLAY tem de ler a MESMA porta — senão o scrub perde o mover.
mutacao "A4 a fita volta a ter a pergunta escrita a' mao" "$TP" \
  '.filter(|&e| crate::reads_the_keyboard(world, e))' \
  '.filter(|&e| world.get::<crate::components::PlatformPlayer>(e).is_some())' \
  ph2d-physics-ecs "--test it" topdown_slide

# ⭐⭐⭐ E o gate da SHELL — a cena que o dono corre, pela porta do produto — tem de o apanhar.
mutacao "A5 a cena do smoke deixa de andar" "$DP" \
  'ph2d_physics_ecs::for_each_keyboard_driven(sim.world(), |entity| {' \
  'sim.world().try_query::<(Entity, &ph2d_physics_ecs::PlatformPlayer)>().into_iter().flat_map(|mut q| q.iter(sim.world()).map(|(e, _)| e).collect::<Vec<_>>()).for_each(|entity| {' \
  ph2d-host-desktop "--test it" the_top_down_smoke_scene_actually_walks

echo "── B) a ordem de desenho das raizes ──────────────────────────────────────"

# ⛔ **O defeito EXACTO**, restaurado: a varredura congela por `to_bits`, que INVERTE a criação.
mutacao "B1 a varredura numera as raizes por to_bits" "$RO" \
  'missing.sort_unstable_by_key(|&e| crate::root_key(world, e));' \
  'missing.sort_unstable_by_key(|e| e.to_bits());' \
  ph2d-ecs --lib root_order

# E o mesmo defeito tem de ser visível na CENA que o dono correu.
mutacao "B2 e a cena de smoke acusa-o" "$RO" \
  'missing.sort_unstable_by_key(|&e| crate::root_key(world, e));' \
  'missing.sort_unstable_by_key(|e| e.to_bits());' \
  ph2d-app-components --lib o_boneco_desenha_por_cima

echo "── $N mutações ──────────────────────────────────────────────────────────"
