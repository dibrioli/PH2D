//! **Fase do quadro: AGRUPAR, RECOLHER E FUNDIR** — os verbos da Hierarquia que mexem na ÁRVORE a partir
//! de uma linha: agrupar/desagrupar, recolher o grupo recém-criado uma vez, e fundir as sprites seleccionadas
//! (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ Correm ANTES do `image_edit::dispatch`: uma edição de imagem no mesmo quadro sobre um dos originais
//! vê a fusão já feita.

use super::*;
use ph2d_i18n::tr;

/// Os pedidos de agrupar e fundir que o dreno do barramento recolheu neste quadro.
pub(super) struct HierarchyMergeIntents {
    pub(super) group_row: Option<(NodeId, bool)>,
    pub(super) merge_sprites_row: Option<NodeId>,
    pub(super) merge_to_layers_row: Option<NodeId>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_hierarchy_group_merge(&mut self, intents: HierarchyMergeIntents) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            renderer,
            sim,
            asset_db,
            toasts,
            tools,
            hero_screen,
            hero_live,
            atlas_asset_map,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let HierarchyMergeIntents {
            group_row,
            merge_sprites_row,
            merge_to_layers_row,
        } = intents;
        // Merge Sprites (Enio 2026-05-27, Hierarchy right-click).
        // Drains BEFORE `image_edit::dispatch` so a same-frame
        // image-edit on one of the originals (extremely unlikely
        // path but documented) doesn't race with the despawn. The
        // multi-selection comes from `hero.gizmo` — primary first,
        // extras after. Right-clicked row resolves to the "primary
        // anchor" the merged sprite parents under.
        // ⚠️ As DUAS entradas do menu caem aqui: «Merge Sprites» e «Merge to Layers». A
        // geometria é a mesma e o modo é a única diferença — ver a chamada abaixo.
        // ⭐⭐⭐ **AGRUPAR / DESAGRUPAR** (Enio, 2026-08-30) — o alcance de um verbo que já
        // existia em `Ctrl+G` e que nenhum menu do app nomeava. A lei do sujeito e as frases
        // vivem em [`ph2d_app_vec::hier_group`], puras e gateadas; aqui só se resolve a linha em bits,
        // se aplica e se diz.
        if let Some((row, agrupar)) = group_row
            && let Some(live) = hero_live.as_ref()
            && let Some(row_bits) = live.bridge.entity_for(row)
        {
            let escolhidos: Vec<u64> = hero.gizmo.iter_selected().collect();
            let sujeito = ph2d_app_vec::hier_group::subject(row_bits, &escolhidos);
            let desfecho = ph2d_app_vec::hier_group::apply(sim, &sujeito, agrupar);
            if let ph2d_app_vec::hier_group::Outcome::Grouped { group, .. } = desfecho {
                // O grupo novo passa a ser a selecção — o gesto seguinte do artista é sobre
                // ELE, e não sobre as peças que acabaram de deixar de ser objectos de topo.
                hero.gizmo.selection = Some(group);
                hero.gizmo.extra_selection.clear();
                // ⚠️ Recolher fica para quando a Hierarquia conhecer a linha — ver o campo.
                self.pending_group_collapse = Some(group);
            }
            toasts.push(desfecho.toast());
            self.title_dirty = true;
        }
        // ⭐ A segunda metade do recolher: a linha do grupo já existe? Então recolhe-a **uma
        // vez** e esquece. ⚠️ Sem o `take`, o artista abria o grupo e o quadro seguinte
        // fechava-o outra vez — um controlo que se desfaz sozinho lê-se como avaria.
        if let Some(bits) = self.pending_group_collapse
            && let Some(live) = hero_live.as_ref()
            && let Some(node) = live.bridge.node_for(bits)
        {
            if !hero.store.is_hierarchy_collapsed(node) {
                hero.store.toggle_hierarchy_collapsed(node);
            }
            self.pending_group_collapse = None;
        }
        if let Some(row) = merge_sprites_row.or(merge_to_layers_row)
            && let Some(live) = hero_live.as_ref()
            && let Some(primary_bits) = live.bridge.entity_for(row)
        {
            let in_selection = hero.gizmo.is_selected(primary_bits);
            let selected_count = hero.gizmo.iter_selected().count();
            // Audit B-M3: if the user right-clicked OUTSIDE the
            // multi-selection (and they already had 2+ sprites
            // selected), the previous behaviour silently fell back
            // to "single-entity merge → <2 warning" which read as
            // "select 2+ first" — misleading. Steer them to the
            // actual fix.
            if !in_selection && selected_count >= 2 {
                toasts.push(ph2d_editor_core::Toast::warning(tr(
                    "shell.fase_hierarchy_group_merge.merge_sprites_right",
                )));
                self.title_dirty = true;
            } else {
                let to_merge: Vec<u64> = if in_selection {
                    hero.gizmo.iter_selected().collect()
                } else {
                    vec![primary_bits]
                };
                if hero_intents::drain_merge_sprites(
                    to_merge.clone(),
                    primary_bits,
                    // ⚠️ O MODO vem de qual das duas linhas do menu foi clicada, e as duas
                    // caem neste mesmo dreno: a geometria é idêntica e duplicá-la seria pedir
                    // que duas cópias concordassem para sempre.
                    merge_to_layers_row.is_some(),
                    hero.project.pixels_per_meter,
                    sim,
                    renderer,
                    asset_db,
                    atlas_asset_map,
                    toasts,
                ) {
                    self.title_dirty = true;
                }
                // Clear merged entity_bits from gizmo so the global
                // gizmo doesn't paint over vanished entities
                // (mirror of Delete).
                for bits in &to_merge {
                    if hero.gizmo.selection == Some(*bits) {
                        hero.gizmo.selection = None;
                    }
                    hero.gizmo.extra_selection.retain(|b| b != bits);
                }
                // Audit B-H2: promote the freshly-spawned merged
                // entity to the selection so the user's next
                // action (Move, Apply tool, etc.) operates on the
                // merged result — matches Photoshop / Figma "after
                // merge, the merged layer IS the selection".
                // **O DOCUMENTO EM CAMADAS** (plano `docs/Sprite_projeto/18` W10). A sprite
                // fundida já tem a textura achatada — ela desenha certo desde já, e grava. O
                // que isto acrescenta é o documento do Painter: abrir o Painter nela mostra
                // uma camada por sprite de origem, na ordem em que foram compostas.
                //
                // ⚠️ **Instala-se AQUI e não dentro da fusão**: quem tem a `ToolRegistry` é o
                // shell. O `sprite_merge` sabe geometria; não sabe onde vive a ferramenta.
                if let Some(doc) = hero_intents::take_last_merged_layers() {
                    crate::merge_layers::install(tools, &doc, toasts);
                }
                if let Some(result) = hero_intents::take_last_merge_result() {
                    hero.gizmo.replace_selection(Some(result.new_entity_bits));
                } else if hero.gizmo.selection.is_none() && !hero.gizmo.extra_selection.is_empty() {
                    // Merge bailed before spawning — promote oldest
                    // surviving extra (mirror of Delete's
                    // headless-cleanup path).
                    hero.gizmo.selection = Some(hero.gizmo.extra_selection.remove(0));
                }
            }
        }
    }
}
