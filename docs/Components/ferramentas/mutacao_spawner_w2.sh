#!/usr/bin/env bash
# Provas de mutação das W2, W3 e W4 da FÁBRICA (line/components, 2026-09-14) — a LEI do transiente
# (os dois leitores), a ponte, a ordem do quadro, o painel e as duas cenas.
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


SV=crates/ph2d-ecs/src/scene/save.rs
SN=crates/ph2d-ecs/src/scene/snapshot.rs
BR=crates/ph2d-app-components/src/factory_bridge.rs
SM=crates/ph2d-app-components/src/factory_smoke.rs
UN=crates/ph2d-unique-name/src/lib.rs
FM=shells/desktop/src/render_loop/fase_fabrica_e_morte.rs
IF=shells/desktop/src/render_loop/inspector_factory.rs
MD=crates/ph2d-editor-core/src/factory_edits.rs

echo "load $(cut -d' ' -f1 /proc/loadavg)"

# ── A LEI: o que nasce numa corrida não é documento ───────────────────────────
mutacao "M1 a corrida entra no DOCUMENTO" "$SV" \
  '        if crate::is_transient(world, entity) {
            continue;
        }' \
  '        if false {
            continue;
        }' \
  ph2d-ecs --test=it what_was_born_in_a_run_never_enters_the_document

mutacao "M2 a corrida entra na HIERARQUIA" "$SN" \
  '        if crate::is_transient(sim_w, entity) {
            continue;
        }' \
  '        if false {
            continue;
        }' \
  ph2d-ecs --test=it the_hierarchy_shows_the_document_not_the_run

mutacao "M3 a poda apanha o DOCUMENTO tambem (o controlo)" "$SV" \
  '        if crate::is_transient(world, entity) {' \
  '        if !crate::is_transient(world, entity) {' \
  ph2d-ecs --test=it the_same_objects_enter_the_document_when_nobody_was_born

# ── A PONTE ───────────────────────────────────────────────────────────────────
# ⚠️⚠️ **O dreno tem DOIS guardas** (o conjunto que deduplica e o `get_entity(..).is_ok()` que
# salta quem já saiu), e mutar um só devolve SOBREVIVEU **sobre um produto correcto** — a terceira
# vez que esta linha paga a forma em dois dias. Os dois ficam: o conjunto dá também a ORDEM, e o
# `is_ok` cobre a entidade que outro braço do quadro já tirou.
mutacao2 "M4 o dreno da morte mata duas vezes (os DOIS guardas caem)" \
  "$BR" '    let unicos: BTreeSet<Entity> = deaths.iter().map(|d| d.entity).collect();' \
         '    let unicos: Vec<Entity> = deaths.iter().map(|d| d.entity).collect();' \
  "$BR" '        if sim.world().get_entity(e).is_ok() {
            sim.world_mut().entity_mut(e).despawn();
            n += 1;
        }
    }
    n
}

/// **Varre tudo o que nasceu numa corrida**' \
         '        sim.world_mut().entity_mut(e).despawn();
        n += 1;
    }
    n
}

/// **Varre tudo o que nasceu numa corrida**' \
  ph2d-app-components --lib the_death_drain_deduplicates

mutacao "M5 a varredura apanha o que o artista desenhou" "$BR" \
  '    let mut q = sim.world_mut().query::<(Entity, &Spawned)>();
    let todos: Vec<Entity> = q.iter(sim.world()).map(|(e, _)| e).collect();' \
  '    let mut q = sim.world_mut().query::<Entity>();
    let todos: Vec<Entity> = q.iter(sim.world()).collect();' \
  ph2d-app-components --lib the_sweep_takes_the_born_and_leaves_the_authored

mutacao "M6 uma receita morta passa em SILENCIO" "$BR" \
  '            out.recusadas += grupo.len();
            continue;
        };
        let n = u32::try_from(grupo.len()).unwrap_or(u32::MAX);' \
  '            continue;
        };
        let n = u32::try_from(grupo.len()).unwrap_or(u32::MAX);' \
  ph2d-app-components --lib a_recipe_that_no_longer_exists_is_a_counted_refusal

mutacao "M7 a copia nasce na pose da RECEITA" "$BR" \
  '            if let Some(mut t) = sim.world_mut().get_mut::<Transform>(copia) {
                t.translation.x = pedido.at[0];
                t.translation.y = pedido.at[1];
            }' \
  '            let _ = pedido;' \
  ph2d-app-components --lib a_signal_makes_copies_and_they_land_where_the_factory_said

mutacao "M8 a copia nasce SEM a marca de nascimento" "$BR" \
  '            sim.world_mut().entity_mut(copia).insert(Spawned {
                by,
                born_tick: tick,
            });' \
  '            let _ = (by, tick);' \
  ph2d-app-components --lib a_signal_makes_copies_and_they_land_where_the_factory_said

# ── A porta dos NOMES ─────────────────────────────────────────────────────────
mutacao "M9 os nomes do lote nao reservam lugar" "$UN" \
  '        usados.insert(nome.clone());' '        let _ = &nome;' \
  ph2d-unique-name --lib the_batch_door_answers_the_same_as_the_single_one

# ── A ORDEM DO QUADRO ─────────────────────────────────────────────────────────
# ⚠️ **A agulha põe a morte a drenar TAMBÉM antes da fábrica** — e o gate reprova pela metade que
# ele declara primeiro: *um marco que aparece duas vezes não tem POSIÇÃO*.
mutacao "M10 a morte drena ANTES de a fabrica nascer" "$FM" \
  '        let a_correr = self.playhead.is_playing();' \
  '        let _cedo = ph2d_app_components::factory_bridge::apply_deaths(sim, &deaths);
        let a_correr = self.playhead.is_playing();' \
  ph2d-host-desktop --test=it the_factory_reads_the_signal_before_the_death_drains

mutacao "M11 a fabrica corre com o relogio PARADO" "$FM" \
  '        let a_correr = self.playhead.is_playing();' \
  '        let a_correr = true || self.playhead.is_playing();' \
  ph2d-host-desktop --test=it the_factory_only_runs_while_the_clock_plays

mutacao "M12 a varredura deixa de ser lida do relogio" "$FM" \
  '        if !a_correr && self.playhead.time() <= 0.0 {' \
  '        if !a_correr {' \
  ph2d-host-desktop --test=it the_sweep_is_read_from_the_clock_and_not_hooked_to_a_button

# ── O PAINEL ──────────────────────────────────────────────────────────────────
mutacao "M13 todo modo le' a caixa da area" "$MD" \
  '    pub const fn uses_area(self) -> bool {
        matches!(self, Self::Area)
    }' \
  '    pub const fn uses_area(self) -> bool {
        true
    }' \
  ph2d-editor-core --lib each_mode_reads_exactly_its_own_fields

mutacao "M14 o painel mostra o ID da receita" "$IF" \
  '    world.get::<ph2d_ecs::Name>(e).map(|n| n.0.clone())' \
  '    Some(id.to_string())' \
  ph2d-host-desktop --bins the_recipe_is_a_name_on_the_panel_and_an_identity_in_the_component

# ⚠️ **A agulha é a ESCRITA e não a leitura**, e a razão é medida: a cerca existe nos dois lados
# (`mestre_por_nome` filtra por `MasterRoot`, e `nome_do_mestre` volta a perguntar), e apagar a da
# LEITURA sobrevive — com a escrita a recusar, o campo fica a `0` e a leitura sai cedo. A da leitura
# é o cinto para o caso que a escrita não alcança: um ficheiro gravado antes de alguém desfazer o
# mestre. *Terceira lei desta wave com dois guardas, e a única em que UM deles é observável.*
mutacao "M15 um objecto comum passa por receita" "$IF" \
  '    let mut q = world.query::<(Entity, &ph2d_ecs::Name, &MasterRoot)>();' \
  '    let mut q = world.query::<(Entity, &ph2d_ecs::Name, Option<&MasterRoot>)>();' \
  ph2d-host-desktop --bins a_name_that_is_not_a_master_is_not_a_recipe

mutacao "M16 o contador de vivas soma o MUNDO" "$IF" \
  '            alive: vivos.get(&meu_id).copied().unwrap_or(0),' \
  '            alive: vivos.values().sum::<u32>(),' \
  ph2d-host-desktop --bins the_snapshot_counts_only_this_factorys_copies

mutacao "M17 a tag do ponto compara CRU" "$IF" \
  '            let id = tree.find(caminho).map_or(0, |t| t.0);' \
  '            let id = tree.tags().find(|t| t.path == *caminho).map_or(0, |t| t.id.0);' \
  ph2d-host-desktop --bins the_spawn_point_tag_is_resolved_folded

# ── AS CENAS ──────────────────────────────────────────────────────────────────
mutacao "M18 a receita fica por resolver" "$SM" \
  '    resolver_receitas(world);
    cena' '    cena' \
  ph2d-app-components --lib the_first_scene_points_the_factory_at_a_master_that_exists

mutacao "M19 a =2 fica sem camera de jogo" "$SM" \
  '        GameCamera {
            height_world: 9.0,
            ..GameCamera::default()
        },' \
  '        Visibility::visible(),' \
  ph2d-app-components --lib the_second_scene_has_tagged_points_a_cap_and_a_reaper

mutacao "M20 o ciclo de vida vai para a FABRICA" "$SM" \
  '            DestroyOutside { margin: 0.5 },
        ));
    });' \
  '        ));
    });' \
  ph2d-app-components --lib the_second_scene_has_tagged_points_a_cap_and_a_reaper

echo
echo "$N mutacoes · load $(cut -d' ' -f1 /proc/loadavg)"
