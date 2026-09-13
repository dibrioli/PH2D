#!/usr/bin/env bash
# Provas de mutação da W3a das Tags (line/components, 2026-09-13) — o TECTO do objecto, a costura
# do Inspector (instantâneo + commit) e a secção do painel (chips · caixa de escolha · criar).
#
# ⚠️ Estas mutações correm DEPOIS do código, e não antes: os gates desta wave nasceram ao lado da
# implementação em vez de vermelhos à frente dela. A prova de que não são inertes é ESTA, e por
# isso ela cobre TODAS as asserções que carregam a lei — não uma amostra.
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


T=crates/ph2d-ecs/src/tags.rs
I=shells/desktop/src/render_loop/inspector_tags.rs
SEC=crates/ph2d-panel-inspector/src/sections/tags.rs
P=crates/ph2d-panel-inspector/src/populate_tags.rs

echo "load $(cut -d' ' -f1 /proc/loadavg)"

# ── O tecto do objecto ────────────────────────────────────────────────────────
mutacao "M1 o objecto aceita mais tags do que a seccao desenha" "$T" \
  '        id.0 != 0 && self.0.len() < TAGS_MAX && self.0.insert(id.0)' \
  '        id.0 != 0 && self.0.insert(id.0)' \
  ph2d-ecs --lib an_object_takes_no_more_tags_than_the_panel_shows

# ── A costura na shell ────────────────────────────────────────────────────────
mutacao "M2 o snapshot nasce para quem nao tem o componente" "$I" \
  '    let tags = world.get::<Tags>(Entity::from_bits(entity_bits))?;' \
  '    let vazio = Tags::default();
    let tags = world
        .get::<Tags>(Entity::from_bits(entity_bits))
        .unwrap_or(&vazio);' \
  ph2d-host-desktop --bins an_object_without_the_component_has_no_section

mutacao "M3 os chips saem do CONJUNTO e nao da arvore" "$I" \
  '    for t in tree.tags() {
        if tags.direct_ids().any(|i| i == t.id) {
            on_object.push(row(t));
        }
        all.push(row(t));
    }' \
  '    for t in tree.tags() {
        all.push(row(t));
    }
    for id in tags.direct_ids() {
        if let Some(t) = tree.get(id) {
            on_object.push(row(t));
        }
    }' \
  ph2d-host-desktop --bins the_snapshot_carries_the_object_and_the_project_in_tree_order

mutacao "M4 o tecto mede os CHIPS em vez do conjunto" "$I" \
  '        full: tags.len() >= TAGS_MAX,' '        full: on_object.len() >= TAGS_MAX,' \
  ph2d-host-desktop --bins an_orphan_id_takes_room_without_painting_a_chip

mutacao "M5 um objecto cheio faz a arvore crescer" "$I" \
  '            if t.len() >= TAGS_MAX {' '            if false {' \
  ph2d-host-desktop --bins a_full_object_does_not_grow_the_tree_with_a_tag_it_cannot_take

mutacao "M6 o commit devolve «chamei o create» em vez da revisao" "$I" \
  '    tree.revision() != antes' '    let _ = antes;
    mudou' \
  ph2d-host-desktop --bins creating_a_path_that_already_exists_changes_nothing_in_the_tree

mutacao "M7 o Create nao marca o objecto" "$I" \
  '                    Ok(id) => t.insert(id),' '                    Ok(id) => {
                        let _ = id;
                        false
                    }' \
  ph2d-host-desktop --bins creating_a_tag_from_the_section_writes_both_documents

# ⚠️ O cap SOBE e o array fica para trás. ⛔ Apagar um elemento do array não serve: o comprimento
# faz parte do TIPO, logo o mutante não compila e a mutação não mede nada.
mutacao "M8 o cap do modelo sobe sem o painel o acompanhar" crates/ph2d-ecs/src/tags.rs \
  'pub const TAGS_MAX: usize = 16;' 'pub const TAGS_MAX: usize = 17;' \
  ph2d-host-desktop --bins the_tag_chip_ids_cover_the_model_cap

# ── A secção do painel ────────────────────────────────────────────────────────
# ⚠️ O ciclo dos chips não corre. ⛔ Renomear a função dá um símbolo inexistente e o mutante não
# compila — a mutação tem de deixar o programa VÁLIDO e errado.
mutacao "M9 os chips nao sao pintados" "$SEC" \
  '    for (row, &id) in rows.iter().zip(crate::ids::INSP_TAGS_CHIP.iter()) {' \
  '    for (row, &id) in rows.iter().take(0).zip(crate::ids::INSP_TAGS_CHIP.iter()) {' \
  ph2d-panel-inspector --test=it os_chips_e_a_caixa_de_escolha_sao_pintados

mutacao "M10 o chip inteiro apanha o clique, nao o x" "$SEC" \
  '        if let Some(close_r) = chip.close_rect(rect) {
            hit_index.register(id, close_r);
        }' '        hit_index.register(id, rect);' \
  ph2d-panel-inspector --test=it o_x_de_um_chip_tira_a_tag_deste_objecto

mutacao "M11 o chip morre sob o dedo (sem registo no populate)" "$P" \
  '    for &id in &crate::ids::INSP_TAGS_CHIP {
        store.register(
            id,
            InteractiveState::Tag {
                state: TagState::Normal,
            },
        );
    }' '' \
  ph2d-panel-inspector --test=it o_x_de_um_chip_tira_a_tag_deste_objecto

mutacao "M12 a lista oferece o que o objecto ja tem" "$SEC" \
  '        !info.on_object.iter().any(|t| t.id == row.id)
            && (filtro.is_empty() || ph2d_label_fold::fold(&row.path).contains(filtro))' \
  '        filtro.is_empty() || ph2d_label_fold::fold(&row.path).contains(filtro)' \
  ph2d-panel-inspector --test=it a_lista_nao_oferece_as_tags_que_o_objecto_ja_tem

mutacao "M13 a busca deixa de dobrar (to_lowercase)" "$SEC" \
  '            && (filtro.is_empty() || ph2d_label_fold::fold(&row.path).contains(filtro))' \
  '            && (filtro.is_empty() || row.path.to_lowercase().contains(filtro))' \
  ph2d-panel-inspector --test=it a_busca_dobra_e_procura_no_caminho_inteiro

mutacao "M14 a busca olha so' para o rotulo" "$SEC" \
  'ph2d_label_fold::fold(&row.path).contains(filtro))' \
  'ph2d_label_fold::fold(&row.label).contains(filtro))' \
  ph2d-panel-inspector --test=it a_busca_dobra_e_procura_no_caminho_inteiro

mutacao "M15 um objecto cheio continua a oferecer a caixa" "$SEC" \
  '        return fold.finish(store, scene, hit_index, cur_y + SECTION_BOTTOM_PAD_PX);' \
  '        ();' \
  ph2d-panel-inspector --test=it um_objecto_cheio_esconde_a_caixa_e_diz_porque

mutacao "M16 o Create e' pintado sem nome nenhum" "$SEC" \
  '    if !escrito.trim().is_empty() && !ja_existe {' '    if true {' \
  ph2d-panel-inspector --test=it o_create_nao_existe_sem_um_nome_novo

mutacao "M17 a seccao TAGS empilha sobre a CAMERA" "crates/ph2d-panel-inspector/src/paint_optional.rs" \
  '    y = paint_camera_section(' '    let _ = paint_camera_section(' \
  ph2d-panel-inspector --test=it two_live_sections_never_share_a_band

# ⚠️ **O que se muta é o RECURSO, não o array**: a barra do `64` sai da altura da região do
# popover, e alargá-la à janela inteira põe mais linhas à vista do que a lista sabe oferecer.
# ⛔ Encolher o array é erro de compilação (o comprimento é o tipo).
mutacao "M18 a regiao do popover cresce e a lista deixa de cobrir um ecra" \
  crates/ph2d-editor-core/src/screens/layout.rs \
  '            self.viewport.w,
            self.inspector.h,
        )
    }
}' \
  '            self.viewport.w,
            self.viewport.h,
        )
    }
}' \
  ph2d-panel-inspector --test=it a_lista_de_tags_cobre_um_ecra_cheio

echo "load $(cut -d' ' -f1 /proc/loadavg)"
git status --short -- crates shells | grep -v '^??' || true
