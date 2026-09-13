//! **O dreno do barramento — a Hierarquia.** Braços do `match` da [`fase_bus_drain`](super), movidos pela
//! ordem de sempre; a única troca no corpo deles é `pd.<pedido>` onde escreviam `<pedido>`, e o `gfx` de cada
//! sub-dreno é re-derivado (o dreno só corre com ele). Ver o cabeçalho de lá.

use super::*;
use ph2d_editor_core::action_bus::EditorAction;

impl crate::App {
    /// A Hierarquia — os interruptores da linha, a árvore (reparentar, duplicar, agrupar, a raiz) e os verbos de
    /// instância, também pelo navegador de assets.
    pub(super) fn fase_bus_hierarchy_tree(
        &mut self,
        action: EditorAction,
        pd: &mut DrainOut,
    ) -> Option<EditorAction> {
        match action {
            EditorAction::Hierarchy(
                ph2d_editor_core::action_bus::HierRequest::ToggleVisibility { row },
            ) => {
                pd.visibility_toggle_row.get_or_insert(row);
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::ToggleLock {
                row,
            }) => {
                pd.lock_toggle_row.get_or_insert(row);
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::ToggleGroup {
                row,
            }) => {
                pd.group_toggle_row.get_or_insert(row);
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::Reparent(
                intent,
            )) => {
                pd.reparent_intent.get_or_insert(intent);
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::Duplicate {
                row,
            }) => {
                pd.duplicate_row.get_or_insert(row);
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::AddChild {
                row,
            }) => {
                pd.add_child_row.get_or_insert(row);
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::Group { row }) => {
                pd.group_row.get_or_insert((row, true));
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::Ungroup { row }) => {
                pd.group_row.get_or_insert((row, false));
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::AddRoot) => {
                pd.add_root = true;
            }
            EditorAction::Hierarchy(
                ph2d_editor_core::action_bus::HierRequest::ResetTransform { row },
            ) => {
                pd.reset_transform_row.get_or_insert(row);
            }
            EditorAction::Hierarchy(
                ph2d_editor_core::action_bus::HierRequest::RevertToMaster { row },
            ) => {
                pd.revert_to_master_row.get_or_insert(row);
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::MakeComponent {
                row,
            }) => {
                pd.instance_verb_row
                    .get_or_insert((row, ph2d_app_components::instance_verbs::Verb::Make));
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::Instantiate {
                row,
            }) => {
                pd.instance_verb_row
                    .get_or_insert((row, ph2d_app_components::instance_verbs::Verb::Place));
            }
            // ⭐⭐⭐ **ABRIR a receita desta cópia** — pelo MESMO dreno dos outros verbos,
            // que é onde vivem a resolução do sujeito e a voz de cada recusa.
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::EditPrefab {
                row,
            }) => {
                pd.instance_verb_row
                    .get_or_insert((row, ph2d_app_components::instance_verbs::Verb::Edit));
            }
            // ⭐ **O verbo de USAR do navegador de assets** (plano `docs/Components/07`,
            // wave A7). ⚠️ O sujeito é o `StableId`, não uma `row`: o navegador não tem
            // linhas, e uma receita está **escondida** da Hierarquia por construção — não
            // há `row` que a endereçe. A resolução `StableId → Entity` acontece na fase
            // da hierarquia, onde o `sim` está emprestado.
            EditorAction::AssetInstantiate { stable_id, at } => {
                pd.instance_verb_stable_id.get_or_insert((
                    stable_id,
                    ph2d_app_components::instance_verbs::Verb::Place,
                    at,
                ));
            }
            // ⭐⭐ **O menu do cartão** (etapa C). ⚠️ `get_or_insert`, como os irmãos: um
            // quadro tem um gesto, e o menu fecha ao primeiro clique.
            EditorAction::AssetCardVerb { asset, verb } => {
                pd.asset_card_verb.get_or_insert((asset, verb));
            }
            EditorAction::AssetCatalogVerb(v) => {
                pd.catalog_verbs.push(v);
            }
            EditorAction::Hierarchy(
                ph2d_editor_core::action_bus::HierRequest::InstantiateLinked { row },
            ) => {
                pd.instance_verb_row
                    .get_or_insert((row, ph2d_app_components::instance_verbs::Verb::PlaceLinked));
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::Detach { row }) => {
                pd.instance_verb_row
                    .get_or_insert((row, ph2d_app_components::instance_verbs::Verb::Detach));
            }
            // ⭐⭐ *Remove from Library* pela linha da Hierarquia — o MESMO verbo do cartão,
            // com o outro sujeito. Ele resolve a receita a partir de uma cópia
            // (`instance_unmake::recipe_root_of`), que é o que torna esta porta útil.
            EditorAction::Hierarchy(
                ph2d_editor_core::action_bus::HierRequest::RemoveFromLibrary { row },
            ) => {
                pd.instance_verb_row
                    .get_or_insert((row, ph2d_app_components::instance_verbs::Verb::Unmake));
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::ApplyToMaster {
                row,
            }) => {
                pd.instance_verb_row
                    .get_or_insert((row, ph2d_app_components::instance_verbs::Verb::Apply));
            }
            other => return Some(other),
        }
        None
    }

    /// A Hierarquia — os verbos de linha (apagar, fundir, as folhas, usar como…), a selecção e o renomear.
    pub(super) fn fase_bus_hierarchy_rows(
        &mut self,
        action: EditorAction,
        pd: &mut DrainOut,
    ) -> Option<EditorAction> {
        let gfx = self.gfx.as_mut()?;
        let FrameGfx { hero_screen, .. } = FrameGfx::of(gfx);
        let hero = hero_screen.as_mut()?;
        match action {
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::Delete { row }) => {
                pd.delete_row.get_or_insert(row);
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::MergeSprites {
                row,
            }) => {
                pd.merge_sprites_row.get_or_insert(row);
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::MergeToLayers {
                row,
            }) => {
                pd.merge_to_layers_row.get_or_insert(row);
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::PackSheet {
                row,
            }) => {
                pd.pack_sheet_row.get_or_insert(row);
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::ArrangeSheet {
                row,
            }) => {
                pd.arrange_sheet_row.get_or_insert(row);
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::BakeSheet {
                row,
            }) => {
                pd.bake_sheet_row.get_or_insert(row);
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::ExportSheet {
                row,
            }) => {
                pd.export_sheet_row.get_or_insert(row);
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::ExportImage {
                row,
            }) => {
                pd.export_image_row.get_or_insert(row);
            }
            EditorAction::Hierarchy(
                ph2d_editor_core::action_bus::HierRequest::RemoveFromSheet { row },
            ) => {
                pd.remove_from_sheet_row.get_or_insert(row);
            }
            EditorAction::Hierarchy(
                ph2d_editor_core::action_bus::HierRequest::UseAsBrushTexture { row },
            ) => {
                pd.use_as_brush_texture_row.get_or_insert(row);
            }
            EditorAction::Hierarchy(
                ph2d_editor_core::action_bus::HierRequest::UseAsBrushShape { row },
            ) => {
                pd.use_as_brush_shape_row.get_or_insert(row);
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::UseAsPaper {
                row,
            }) => {
                pd.use_as_paper_row.get_or_insert(row);
            }
            EditorAction::Hierarchy(
                ph2d_editor_core::action_bus::HierRequest::UseAsGranulation { row },
            ) => {
                pd.use_as_granulation_row.get_or_insert(row);
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::RowClick {
                row,
            }) => {
                pd.hierarchy_row_click.get_or_insert(row);
            }
            // Fase 0e: multi-select-aware hierarchy click +
            // shift-range. Collect into a single latest-wins
            // intent — the dispatch resolves row → entity_bits
            // and applies the matching `GizmoStateGroup`
            // mutation. Range overrides Row when both arrive
            // in the same frame (the user can only be in one
            // selection-gesture at a time).
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::SelectRow {
                row,
                modifier,
            }) if !matches!(
                pd.hierarchy_select_intent,
                Some(hierarchy::HierarchySelectIntent::Range { .. })
            ) =>
            {
                pd.hierarchy_select_intent =
                    Some(hierarchy::HierarchySelectIntent::Row { row, modifier });
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::RangeSelect {
                row,
            }) => {
                pd.hierarchy_select_intent = Some(hierarchy::HierarchySelectIntent::Range { row });
            }
            // Fase 0e: canvas-side select via the bus (reserved
            // for callers that don't have direct hero access —
            // input_dispatch.rs:435 mutates hero.gizmo directly
            // because it already holds the borrow).
            EditorAction::SelectSprite {
                entity_bits,
                modifier,
            } => match modifier {
                ph2d_editor_core::action_bus::SelectModifier::Replace => {
                    hero.gizmo.replace_selection(Some(entity_bits));
                }
                ph2d_editor_core::action_bus::SelectModifier::Add => {
                    hero.gizmo.add_to_selection(entity_bits);
                }
                ph2d_editor_core::action_bus::SelectModifier::Toggle => {
                    hero.gizmo.toggle_in_selection(entity_bits);
                }
            },
            EditorAction::ClearSelection => {
                hero.gizmo.clear_all_selection();
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::RenameSeed {
                row,
            }) => {
                pd.rename_seed_row.get_or_insert(row);
            }
            EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::RenameCommit {
                row,
                new_name,
            }) if pd.rename_commit.is_none() => {
                pd.rename_commit = Some((row, new_name));
            }
            other => return Some(other),
        }
        None
    }
}
