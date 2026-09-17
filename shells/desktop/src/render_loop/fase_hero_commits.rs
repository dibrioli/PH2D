//! **O ramo do `HeroScreen` — o que HONRA os pedidos.** Um bloco do [`fase_hero_frame`](super), movido pela ordem de
//! sempre, com as trocas declaradas lá.

use super::*;
use std::mem::take;

impl crate::App {
    /// A selecção da Hierarquia, as receitas e os assets, o despacho da Hierarquia, os commits do Inspector, a
    /// estratégia da fonte, a paleta, a precisão e a emissão, as folhas, a física, a autokey, o agrupar e o fundir,
    /// os usos como pincel e papel, as edições de imagem e o fim do ramo.
    pub(super) fn fase_hero_commits(
        &mut self,
        pd: &mut fase_bus_drain::DrainOut,
        view: HeroView,
        reparent_intent: Option<ph2d_editor_core::screens::hero::HierReparentIntent>,
        image_apply: fase_image_tool_bridges::ImageToolBridgesOut,
        painter_apply_committed: bool,
    ) -> Option<()> {
        let HeroView {
            window_size,
            viewport,
            ..
        } = view;
        let fase_image_tool_bridges::ImageToolBridgesOut {
            padding_apply,
            bgremoval_apply_committed,
            color_equalization_apply,
            equalize_sizes_apply,
            upscale_apply,
        } = image_apply;
        let hierarchy_select_intent =
            self.fase_hierarchy_select_lock(take(&mut pd.hierarchy_select_intent))?;
        self.fase_recipe_and_asset_verbs(fase_recipe_and_asset_verbs::RecipeVerbIntents {
            catalog_verbs: take(&mut pd.catalog_verbs),
            swap_variant: take(&mut pd.swap_variant),
            apply_added: take(&mut pd.apply_added),
            apply_to_level: take(&mut pd.apply_to_level),
            open_asset_browser: take(&mut pd.open_asset_browser),
        });
        self.fase_hierarchy_dispatch(
            fase_hierarchy_dispatch::HierarchyIntents {
                visibility_toggle_row: take(&mut pd.visibility_toggle_row),
                lock_toggle_row: take(&mut pd.lock_toggle_row),
                group_toggle_row: take(&mut pd.group_toggle_row),
                reparent_intent,
                duplicate_row: take(&mut pd.duplicate_row),
                duplicate_made: take(&mut pd.duplicate_made),
                add_child_row: take(&mut pd.add_child_row),
                add_root: take(&mut pd.add_root),
                reset_transform_row: take(&mut pd.reset_transform_row),
                revert_to_master_row: take(&mut pd.revert_to_master_row),
                instance_verb_row: take(&mut pd.instance_verb_row),
                instance_verb_stable_id: take(&mut pd.instance_verb_stable_id),
                asset_card_verb: take(&mut pd.asset_card_verb),
                delete_row: take(&mut pd.delete_row),
                hierarchy_row_click: take(&mut pd.hierarchy_row_click),
                hierarchy_select_intent,
                rename_seed_row: take(&mut pd.rename_seed_row),
                rename_commit: take(&mut pd.rename_commit),
                view_focus_kind: take(&mut pd.view_focus_kind),
            },
            window_size,
        );
        let joint_pivot_commit =
            self.fase_inspector_commits(fase_inspector_commits::InspectorIntents {
                reimport_entity: take(&mut pd.reimport_entity),
                transform_edit: take(&mut pd.transform_edit),
                visibility_edits: take(&mut pd.visibility_edits),
                sprite_edits: take(&mut pd.sprite_edits),
                ordering_edits: take(&mut pd.ordering_edits),
                sampling_edits: take(&mut pd.sampling_edits),
                blend_edits: take(&mut pd.blend_edits),
                slice_edits: take(&mut pd.slice_edits),
                anchor_edits: take(&mut pd.anchor_edits),
                anim_edits: take(&mut pd.anim_edits),
                timer_edits: take(&mut pd.timer_edits),
                audio_edits: take(&mut pd.audio_edits),
                camera_edits: take(&mut pd.camera_edits),
                factory_edits: take(&mut pd.factory_edits),
                topdown_edits: take(&mut pd.topdown_edits),
                projectile_edits: take(&mut pd.projectile_edits),
                statemachine_edits: take(&mut pd.statemachine_edits),
                script_edits: take(&mut pd.script_edits),
                particles_edits: take(&mut pd.particles_edits),
                tags_edits: take(&mut pd.tags_edits),
                tag_tree_edits: take(&mut pd.tag_tree_edits),
                inspector_queue_dirty: take(&mut pd.inspector_queue_dirty),
                action_edits: take(&mut pd.action_edits),
                physics_edits: take(&mut pd.physics_edits),
                visibility_section_edits: take(&mut pd.visibility_section_edits),
                name_edit: take(&mut pd.name_edit),
                signal_edit: take(&mut pd.signal_edit),
                signal_leave_edit: take(&mut pd.signal_leave_edit),
            })?;
        self.fase_source_strategy_and_joint_pivot(
            fase_source_strategy_and_joint_pivot::SourceStrategyIntents {
                sprite_source_change: take(&mut pd.sprite_source_change),
            },
            joint_pivot_commit,
        );
        self.fase_component_palette(pd.add_component_for);
        self.fase_sprite_precision_emissive(fase_sprite_precision_emissive::SpriteRowIntents {
            remove_from_sheet_row: take(&mut pd.remove_from_sheet_row),
            precision_request: take(&mut pd.precision_request),
            emissive_edits: take(&mut pd.emissive_edits),
        });
        self.fase_sheet_verbs(fase_sheet_verbs::SheetIntents {
            pack_sheet_row: take(&mut pd.pack_sheet_row),
            arrange_sheet_row: take(&mut pd.arrange_sheet_row),
            bake_sheet_row: take(&mut pd.bake_sheet_row),
            export_sheet_row: take(&mut pd.export_sheet_row),
            export_image_row: take(&mut pd.export_image_row),
        });
        self.fase_physics_edits(fase_physics_edits::PhysicsEditIntents {
            joint_edits: take(&mut pd.joint_edits),
            wheel_edits: take(&mut pd.wheel_edits),
            player_edits: take(&mut pd.player_edits),
            join_draw_arm: take(&mut pd.join_draw_arm),
        });
        self.fase_physics_join_rig_bake(fase_physics_join_rig_bake::PhysicsCreateIntents {
            bake_request: take(&mut pd.bake_request),
            join_chain: take(&mut pd.join_chain),
            rig_now: take(&mut pd.rig_now),
            inspector_selection: take(&mut pd.inspector_selection),
        });
        self.fase_autokey();
        self.fase_hierarchy_group_merge(fase_hierarchy_group_merge::HierarchyMergeIntents {
            group_row: take(&mut pd.group_row),
            merge_sprites_row: take(&mut pd.merge_sprites_row),
            merge_to_layers_row: take(&mut pd.merge_to_layers_row),
        });
        self.fase_use_as_brush(pd.use_as_brush_texture_row, pd.use_as_brush_shape_row);
        self.fase_use_as_paper(pd.use_as_paper_row, pd.use_as_granulation_row);
        self.fase_image_edit_apply(
            fase_image_edit_apply::ImageEditIntents {
                trim_entities: take(&mut pd.trim_entities),
                make_square_entities: take(&mut pd.make_square_entities),
                real_size_entities: take(&mut pd.real_size_entities),
                rasterize_entities: take(&mut pd.rasterize_entities),
                undo_image_edit: take(&mut pd.undo_image_edit),
            },
            padding_apply,
            bgremoval_apply_committed,
            color_equalization_apply,
            equalize_sizes_apply,
            upscale_apply,
            painter_apply_committed,
        );
        self.fase_hero_chrome_tail(viewport);
        Some(())
    }
}
