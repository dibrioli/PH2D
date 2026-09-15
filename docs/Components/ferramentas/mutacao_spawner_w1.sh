#!/usr/bin/env bash
# Provas de mutação da W1 da FÁBRICA e do CICLO DE VIDA (line/components, 2026-09-14) —
# a porta em LOTE, as duas leis de morte e a lei da fábrica.
#
# ⚠️ Elas correm DEPOIS do código: a prova de que os gates não são inertes é ESTA, e por isso ela
# cobre as asserções que CARREGAM a lei — não uma amostra.
#
# ⛔ Um `✗ SOBREVIVEU` é um gate que não afirma o que o doc-comment dele diz. Um `✗ CONTROLO
# inválido` é o FILTRO errado, não o produto.
#
# Para cada mutação: (1) CONTROLO — o filtro corre >= 1 teste e passa na árvore limpa; (2) muta, com
# a âncora a ocorrer EXACTAMENTE uma vez; (3) corre; (4) restaura do backup e dá `touch`.
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


# ⚠️ **Algumas leis têm DOIS guardas e nenhum é observável sozinho** — o outro tapa o buraco do
# primeiro, e uma mutação de uma agulha só devolve «SOBREVIVEU» sobre uma lei que está certa. Esta
# variante muta os dois de uma vez, que é a redacção que de facto apaga a lei.
mutacao2() { # nome f1 old1 new1 f2 old2 new2 crate alvo filtro
  local nome=$1 f1=$2 o1=$3 n1=$4 f2=$5 o2=$6 n2=$7 crate=$8 target=$9 filter=${10}
  N=$((N+1))
  local ctl mut
  ctl=$(corre "$crate" "$target" "$filter" "$S/$N-controlo.log")
  if [ "$ctl" = "ERRO" ] || [ "${ctl%% *}" -lt 1 ] || [ "${ctl##* }" -ne 0 ]; then
    echo "✗ $nome — CONTROLO inválido ($ctl) · filtro «$filter»"; return
  fi
  backup "$f1"; backup "$f2"
  if ! replace "$f1" "$o1" "$n1" || ! replace "$f2" "$o2" "$n2"; then
    echo "✗ $nome — âncora não casou"
    cp "$S/bak/$(echo "$f1" | tr '/' '_')" "$f1" && touch "$f1"
    cp "$S/bak/$(echo "$f2" | tr '/' '_')" "$f2" && touch "$f2"
    return
  fi
  mut=$(corre "$crate" "$target" "$filter" "$S/$N-mutante.log")
  cp "$S/bak/$(echo "$f1" | tr '/' '_')" "$f1" && touch "$f1"
  cp "$S/bak/$(echo "$f2" | tr '/' '_')" "$f2" && touch "$f2"
  if [ "$mut" = "ERRO" ]; then
    echo "? $nome — o mutante NÃO COMPILOU (ver $S/$N-mutante.log)"
  elif [ "${mut##* }" -ge 1 ]; then
    echo "✓ $nome — SANGROU (controlo $ctl · mutante $mut)"
  else
    echo "✗ $nome — SOBREVIVEU (controlo $ctl · mutante $mut)"
  fi
}

IN=crates/ph2d-ecs/src/instantiate.rs
LF=crates/ph2d-ecs/src/lifetime.rs
FC=crates/ph2d-ecs/src/factory.rs

echo "load $(cut -d' ' -f1 /proc/loadavg)"

# ── A porta em LOTE ───────────────────────────────────────────────────────────
# ⚠️ **A agulha NÃO é «assinar a identidade por cópia»** — essa mutação SOBREVIVEU (2026-09-14) e
# está certo que sobreviva: a atribuição é por ordem de SPAWN, logo corrê-la N vezes dá os mesmos
# ids. O que o gate defende é o EMPARELHAMENTO entre cada cópia e o mapa dela, que é onde um lote
# de facto se engana.
mutacao "M1 o lote devolve as copias fora de ordem" "$IN" \
  '    Ok(saida
        .into_iter()' \
  '    Ok(saida
        .into_iter()
        .rev()' \
  ph2d-ecs --lib copying_in_one_batch_gives_the_same_identity_as_copying_one_by_one

# ── O CICLO DE VIDA ───────────────────────────────────────────────────────────
mutacao "M2 a morte e' o ESTADO e nao a travessia" "$LF" \
  '        if antes < vida.duration_us && relogio.elapsed_us >= vida.duration_us {' \
  '        if relogio.elapsed_us >= vida.duration_us {' \
  ph2d-ecs --lib a_life_ends_on_the_tick_its_time_is_up

mutacao "M3 a morte escorrega um tique" "$LF" \
  '        if antes < vida.duration_us && relogio.elapsed_us >= vida.duration_us {' \
  '        if antes < vida.duration_us && relogio.elapsed_us > vida.duration_us {' \
  ph2d-ecs --lib a_life_ends_on_the_tick_its_time_is_up

# ⚠️⚠️ **A lei «só morre quem nasceu» tem DOIS guardas** — o `With<Spawned>` do reconcile (que não
# dá relógio a quem nunca corre) e o `&Spawned` da query. Mutar um só devolve SOBREVIVEU **sobre um
# produto correcto**, porque o outro tapa o buraco. Medido em 2026-09-14, e é a segunda vez que esta
# linha paga a forma (a 1.ª foi o «id órfão conta para ninguém» da W4 das Tags).
mutacao2 "M4 a corrida apaga DOCUMENTO (os DOIS guardas caem)" \
  "$LF" '        .query_filtered::<Entity, (With<Lifetime>, With<Spawned>, Without<LifetimeRuntime>)>()' \
         '        .query_filtered::<Entity, (With<Lifetime>, Without<LifetimeRuntime>)>()' \
  "$LF" '    let mut q = world.query::<(Entity, &Lifetime, &mut LifetimeRuntime, &Spawned)>();
    for (e, vida, mut relogio, nascimento) in q.iter_mut(world) {' \
         '    let mut q = world.query::<(Entity, &Lifetime, &mut LifetimeRuntime, Option<&Spawned>)>();
    for (e, vida, mut relogio, nascimento) in q.iter_mut(world) {
        let nascimento = nascimento.copied().unwrap_or_default();' \
  ph2d-ecs --lib a_lifetime_on_an_authored_object_never_kills

mutacao "M5 o recem-nascido envelhece no tique em que nasce" "$LF" \
  '        if vida.duration_us == 0 || nascimento.born_tick >= tick_now {' \
  '        if vida.duration_us == 0 {' \
  ph2d-ecs --lib the_newborn_does_not_age_on_the_tick_it_was_born

mutacao "M6 a morte sai pela ordem da QUERY" "$LF" \
  '    com_id.sort_by_key(|(id, _)| *id);' \
  '    com_id.sort_by_key(|_| 0u64);' \
  ph2d-ecs --lib the_deaths_come_in_identity_order_not_in_query_order

mutacao "M7 uma margem NEGATIVA encolhe o ecra" "$LF" \
  '        let m = margem.max(0.0);' '        let m = margem;' \
  ph2d-ecs --lib the_margin_grows_the_rectangle_and_a_negative_one_does_not_shrink_it

mutacao "M8 o fora-do-ecra mede a pose LOCAL" "$LF" \
  '        let Some(t) = crate::world_transform(mundo, e) else {
            continue;
        };' \
  '        let Some(t) = mundo.get::<Transform>(e).copied() else {
            continue;
        };' \
  ph2d-ecs --lib outside_is_measured_in_world_space_not_local

mutacao "M9 um objecto autorado morre fora do ecra" "$LF" \
  'world.query_filtered::<(Entity, &DestroyOutside), (With<Spawned>, With<Transform>)>();' \
  'world.query_filtered::<(Entity, &DestroyOutside), (With<Transform>, With<Transform>)>();' \
  ph2d-ecs --lib an_authored_object_never_dies_outside

mutacao "M10 o relogio nasce em quem nunca vai correr" "$LF" \
  '        .query_filtered::<Entity, (With<Lifetime>, With<Spawned>, Without<LifetimeRuntime>)>()' \
  '        .query_filtered::<Entity, (With<Lifetime>, Without<LifetimeRuntime>)>()' \
  ph2d-ecs --lib the_clock_is_only_given_to_those_who_were_born_in_a_run

# ── A FÁBRICA ─────────────────────────────────────────────────────────────────
mutacao "M11 uma fabrica sem sinal escuta TUDO" "$FC" \
  '        .filter(|(_, f)| !f.on_signal.is_empty() && fired.contains(&f.on_signal.as_str()))' \
  '        .filter(|(_, f)| fired.contains(&f.on_signal.as_str()) || f.on_signal.is_empty())' \
  ph2d-ecs --lib a_factory_without_a_signal_never_spawns

mutacao "M12 sem receita ela nasce na origem" "$FC" \
  '        if quantas > 0 && f.master != 0 {' '        if quantas > 0 {' \
  ph2d-ecs --lib a_factory_without_a_recipe_is_silent

mutacao "M13 o tecto do quadro nao crava" "$FC" \
  '    let mut n = f.burst.min(BURST_MAX);' '    let mut n = f.burst;' \
  ph2d-ecs --lib the_frame_ceiling_clamps_the_burst

mutacao "M14 o limite de vivas conta o MUNDO" "$FC" \
  '        let quantas = quantas_nascem(&f, &estado, vivos.get(&id).copied().unwrap_or(0));' \
  '        let quantas = quantas_nascem(&f, &estado, vivos.values().sum::<u32>());' \
  ph2d-ecs --lib the_alive_limit_counts_only_this_factorys_copies

mutacao "M15 ela grita que acabou a cada sinal" "$FC" \
  '        if f.total_max > 0 && estado.total >= f.total_max && !estado.exhausted_said {' \
  '        if f.total_max > 0 && estado.total >= f.total_max {' \
  ph2d-ecs --lib the_total_limit_says_exhausted_exactly_once

mutacao "M16 a semente ignora a IDENTIDADE" "$FC" \
  '            self.rng = (seed ^ id.rotate_left(32)) | 1;' '            self.rng = seed | 1;' \
  ph2d-ecs --lib two_factories_with_the_same_authored_seed_scatter_differently

mutacao "M17 a roda-viva fica parada no primeiro ponto" "$FC" \
  '                            estado.cursor = estado.cursor.wrapping_add(1);' \
  '                            estado.cursor = estado.cursor.wrapping_add(0);' \
  ph2d-ecs --lib a_spawn_point_is_a_tagged_object_and_cycle_goes_round

mutacao "M18 uma tag vazia faz nascer na origem" "$FC" \
  '            if pontos.is_empty() {
                return Vec::new();
            }' \
  '            if pontos.is_empty() {
                return vec![base; quantas as usize];
            }' \
  ph2d-ecs --lib a_tag_with_nobody_in_it_spawns_nobody

mutacao "M19 os nascimentos saem pela ordem da QUERY" "$FC" \
  '        candidatas.sort_by_key(|(e, _)| mundo.get::<StableId>(*e).map_or(u64::MAX, |s| s.0));' \
  '        candidatas.sort_by_key(|(e, _)| { let _ = mundo; 0u64 });' \
  ph2d-ecs --lib the_births_come_in_factory_identity_order

mutacao "M20 o evento de nascimento vira um por copia" "$FC" \
  '                if !f.on_spawned.is_empty() {
                    out.spawned.push((e, f.on_spawned.clone(), nasceram));
                }' \
  '                for _ in 0..nasceram {
                    if !f.on_spawned.is_empty() {
                        out.spawned.push((e, f.on_spawned.clone(), 1));
                    }
                }' \
  ph2d-ecs --lib a_burst_puts_exactly_that_many_copies_at_the_factory

echo
echo "$N mutacoes · load $(cut -d' ' -f1 /proc/loadavg)"
