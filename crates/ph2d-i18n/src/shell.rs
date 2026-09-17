//! **O QUE A SHELL DIZ** — os avisos (toasts) dos verbos do app, os filtros dos diálogos de
//! ficheiro, as frases de importação/exportação/gravação e os nomes por omissão que a shell dá a
//! objectos novos (`shell.<ficheiro>.<frase>`).
//!
//! ⚠️ **Frases com peças do código usam `ph2d_i18n::tr_with` com marcadores NOMEADOS** (`{name}`,
//! `{err}`), convertidas de `format!` por script com o número já formatado quando o `format!` tinha
//! uma especificação (`{:.1}` → `format!("{:.1}", x)` no sítio). Uma língua pode reordenar os
//! marcadores; não pode mudar o que eles valem.
//!
//! ⛔ **O que NÃO está aqui, de propósito** (e tem excepção nomeada no gate da shell): os ficheiros de
//! cena de smoke, os formatos de ficheiro (`# PH2D layout`, `prefs`), o diagnóstico de consola e os
//! nomes-identificador de entidade (`Entity_…`, `piece_…`).

/// A tradução de uma chave `shell.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "shell.asset_card_verbs.left_alone" => " \u{b7} {skipped} left alone",
        "shell.asset_card_verbs.name_s_used_more_than" => {
            " \u{b7} {ambiguous} name(s) used more than once were skipped"
        }
        "shell.asset_card_verbs.replaced_object_s_with" => {
            "Replaced {done} object(s) with \u{201c}{name}\u{201d} \u{2014} {kept} override(s) kept"
        }
        "shell.asset_card_verbs.nothing_you_picked_is" => {
            "Nothing you picked is a copy of a prefab"
        }
        "shell.asset_card_verbs.that_copy_cannot" => "That copy cannot become this prefab",
        "shell.asset_card_verbs.those_are_already" => "Those are already copies of this prefab",
        "shell.asset_card_verbs.pick_the_copy_you_want" => {
            "Pick the copy you want to replace first \u{2014} then choose this again"
        }
        "shell.asset_card_verbs.this_image_is_in_the" => {
            "This image is in the library because {n} object(s) use it \u{2014} change those to remove it"
        }
        "shell.asset_card_verbs.removed_from_library" => "Removed from library",
        "shell.asset_card_verbs.that_prefab_is_no" => "That prefab is no longer in the project",
        "shell.asset_card_verbs.an_image_is_not_a" => {
            "An image is not a prefab \u{2014} drop it on an object to change what it draws"
        }
        "shell.asset_card_verbs.selected_object_s" => "Selected {n} object(s)",
        "shell.asset_card_verbs.nothing_is_using_this" => "Nothing is using this image",
        "shell.asset_card_verbs.no_copies_of_this" => "No copies of this prefab in the scene",
        "shell.asset_card_verbs.drop_an_image_on_an" => {
            "Drop an image on an object to use it \u{2014} an image has no place of its own"
        }
        "shell.asset_card_verbs.an_image_has_no_shape" => {
            "An image has no shape to edit here \u{2014} use \u{201c}Select users\u{201d} and \
                 edit it on an object"
        }
        "shell.asset_card_verbs.editing_move_a_piece" => {
            "Editing \u{201c}{name}\u{201d} \u{2014} move a piece and every copy follows"
        }
        "shell.asset_catalog_verbs.removed_from_its" => "Removed from its catalog",
        "shell.asset_catalog_verbs.moved_to" => "Moved to \u{201c}{name}\u{201d}",
        "shell.asset_catalog_verbs.catalog_deleted_the" => {
            "Catalog \u{201c}{label}\u{201d} deleted \u{2014} the assets in it were not"
        }
        "shell.asset_catalog_verbs.a_catalog_name_cannot" => {
            "A catalog name cannot be empty or contain \u{201c}/\u{201d}"
        }
        "shell.asset_catalog_verbs.catalog_created" => "Catalog \u{201c}{label}\u{201d} created",
        "shell.asset_catalog_verbs.catalog" => "Catalog",
        "shell.asset_drop.drop_an_image_on_a" => {
            "Drop an image on a sprite to retexture it, or on empty canvas"
        }
        "shell.asset_drop.drop_a_prefab_on_the" => "Drop a prefab on the canvas to place it",
        "shell.asset_drop_apply.could_not_place_that" => "Could not place that asset",
        "shell.asset_drop_apply.image_placed" => "Image placed",
        "shell.asset_drop_apply.image" => "Image",
        "shell.asset_drop_apply.could_not_apply" => "Could not apply: {e}",
        "shell.asset_drop_apply.texture_applied" => "Texture applied",
        "shell.asset_index_build.pieces" => "{n} pieces",
        "shell.asset_index_build.n1_piece" => "1 piece",
        "shell.asset_index_build.component" => "Component {stable_id}",
        "shell.envelope_live.envelope" => "Envelope",
        "shell.hero_bridge.prf" => "PRF",
        "shell.hierarchy.moved_into_a_prefab_it" => {
            "Moved into a prefab — it shows while the prefab row is selected"
        }
        "shell.hierarchy.that_piece_s_place" => {
            "That piece's place comes from the prefab \u{2014} open it with \u{201c}Edit \
             Prefab\u{201d} to move it there"
        }
        "shell.view.view_all_empty_scene" => "View · All (empty scene · reset)",
        "shell.view.view_all" => "View · All",
        "shell.view.view_camera_origin" => "View · Camera (origin)",
        "shell.view.view_selected_no" => "View · Selected (no selection · origin)",
        "shell.view.view_selected" => "View · Selected",
        "shell.hier_group.group" => "Group {membros}",
        "shell.hier_group.right_click_on_one_of" => "Right-click on one of the selected objects",
        "shell.hier_group.nothing_in_the" => "Nothing in the selection is inside a group",
        "shell.hier_group.select_at_least_2" => "Select at least 2 objects to group",
        "shell.hier_group.ungrouped_groups" => "Ungrouped {groups} groups",
        "shell.hier_group.ungrouped" => "Ungrouped",
        "shell.hier_group.grouped_objects" => "Grouped {members} objects",
        "shell.init.ph2d_editor" => "PH2D — editor",
        "shell.init_subsystems.press_1_brush_2_move_3" => {
            "Press 1=Brush, 2=Move, 3=Bg Removal, Tab=Zen"
        }
        "shell.init_subsystems.editor_data_layer" => "Editor data layer wired (M12)",
        "shell.despacho_clique_gizmo.ent" => "ENT",
        "shell.despacho_clique_largar.ent" => "ENT",
        "shell.despacho_clique_pick.ent" => "ENT",
        "shell.despacho_clique_reclamantes.tool" => "Tool · {tool_label}",
        "shell.despacho_clique_reclamantes.sidebar" => "Sidebar · {sidebar_side_}",
        "shell.despacho_metodos_janela_e_vetor.that_selection_is_not" => {
            "That selection is not inside a group"
        }
        "shell.despacho_metodos_janela_e_vetor.select_two_or_more" => {
            "Select two or more objects to group"
        }
        "shell.despacho_metodos_janela_e_vetor.group" => "Group {sel}",
        "shell.despacho_metodos_picks_e_arrastos.blast_bodies" => "Blast: {hit} bodies",
        "shell.handlers_teclas_editor.timeline_hidden_l" => "Timeline hidden (L)",
        "shell.handlers_teclas_editor.timeline_shown_l" => "Timeline shown (L)",
        "shell.handlers_teclas_editor.timeline_pause" => "Timeline · pause",
        "shell.handlers_teclas_editor.timeline_play" => "Timeline · play",
        "shell.handlers_teclas_editor.timeline_no_playback" => {
            "Timeline · no playback in the Containers list"
        }
        "shell.handlers_teclas_editor.grid_off" => "Grid · off",
        "shell.handlers_teclas_editor.grid_on" => "Grid · on",
        "shell.handlers_teclas_editor.camera_reset" => "Camera · reset",
        "shell.input_handlers.tokens_hidden_t" => "Tokens hidden (T)",
        "shell.input_handlers.tokens_shown_t" => "Tokens shown (T)",
        "shell.input_handlers.physics_hidden_w" => "Physics hidden (W)",
        "shell.input_handlers.physics_shown_w" => "Physics shown (W)",
        "shell.input_handlers.colliders_hidden_b" => "Colliders hidden (B)",
        "shell.input_handlers.colliders_shown_b" => "Colliders shown (B)",
        "shell.input_handlers.theme" => "Theme · {id}",
        "shell.input_handlers.no_tools_to_show_here" => "No tools to show here (P)",
        "shell.input_handlers.zen_mode_off_zones" => "Zen mode OFF (zones restored)",
        "shell.input_handlers.zen_mode_on_zones" => "Zen mode ON (zones collapsed)",
        "shell.label_live.label" => "{base} Label",
        "shell.label_live.path" => "Path {host}",
        "shell.fase_bone_smart_and_knobs.this_bone_is_driven_by" => {
            "This bone is driven by an IK anchor, so its angle is derived - turning it \
                         will not run the action. Use a free bone, or Remove IK."
        }
        "shell.fase_bone_smart_and_knobs.is_open_in_the" => {
            "\"{nome}\" is open in the timeline, so you are EDITING it - the bone will \
                         not run it. Switch the timeline to another animation to see it play."
        }
        "shell.fase_bus_inspector.dropped_1_unused" => "Dropped 1 unused override",
        "shell.fase_bus_inspector.put_the_piece_back_it" => {
            "Put the piece back \u{2014} it returns as the component has it"
        }
        "shell.fase_bus_inspector.cleared_unused" => "Cleared {n} unused override(s)",
        "shell.fase_field3d_requests.there_is_no_sculpture" => {
            "There is no sculpture in the scene to bring in"
        }
        "shell.fase_hierarchy_group_merge.merge_sprites_right" => {
            "Merge Sprites: right-click on one of the selected sprites"
        }
        "shell.fase_inspector_commits.audio_commit_failed" => "Audio commit failed: {e}",
        "shell.fase_physics_edits.wheel_commit_failed" => "Wheel commit failed: {e}",
        "shell.fase_physics_edits.joint_commit_failed" => "Joint commit failed: {e}",
        "shell.fase_physics_join_rig_bake.baked_bodies_tracks" => {
            "Baked {window} - {bodies} bodies, {tracks} tracks - now Kinematic"
        }
        "shell.fase_physics_join_rig_bake.nothing_to_bake" => "Nothing to bake: nothing moved",
        "shell.fase_physics_join_rig_bake.already_baked_the" => {
            "Already baked - the timeline drives these bodies now"
        }
        "shell.fase_physics_join_rig_bake.finish_the_current" => {
            "Finish the current edit before baking"
        }
        "shell.fase_physics_join_rig_bake.cannot_bake_here_this" => {
            "Cannot bake here: this clip does not play exactly once"
        }
        "shell.fase_physics_join_rig_bake.rigged_new_bodies_with" => {
            "Rigged {bodies} new bodies with {joints} joints"
        }
        "shell.fase_physics_join_rig_bake.rig_commit_failed" => "Rig commit failed: {e}",
        "shell.fase_physics_join_rig_bake.chained_bodies_with" => {
            "Chained {made2} bodies with {made} joints"
        }
        "shell.fase_recipe_and_asset_verbs.that_is_not_a_copy_of" => {
            "That is not a copy of a prefab"
        }
        "shell.fase_recipe_and_asset_verbs.these_components_are" => {
            "These components are not related \u{2014} switching would lose every override"
        }
        "shell.fase_recipe_and_asset_verbs.switched_variant" => {
            "Switched variant \u{2014} {overrides_kept} override(s) kept"
        }
        "shell.fase_recipe_and_asset_verbs.switched_variant_2" => {
            "Switched variant \u{2014} {overrides_kept} override(s) kept, {dropped} piece(s) unused"
        }
        "shell.fase_recipe_and_asset_verbs.that_is_not_a_piece_of" => {
            "That is not a piece of a copy"
        }
        "shell.fase_recipe_and_asset_verbs.that_piece_came_from" => {
            "That piece came from the component \u{2014} it is already in it"
        }
        "shell.fase_recipe_and_asset_verbs.added_piece_s_to_every" => {
            "Added {pieces} piece(s) to \u{201c}{name}\u{201d} \u{2014} every copy gets them"
        }
        "shell.fase_recipe_and_asset_verbs.the_prefab" => "the prefab",
        "shell.fase_recipe_and_asset_verbs.not_part_of_an" => "Not part of an instance",
        "shell.fase_recipe_and_asset_verbs.applied_change_s_to" => {
            "Applied {changed} change(s) to \u{201c}{name}\u{201d}"
        }
        "shell.fase_recipe_and_asset_verbs.applied_change_s_to_2" => {
            "Applied {changed} change(s) to \u{201c}{name}\u{201d} \u{2014} {left} left (not part of it)"
        }
        "shell.fase_recipe_and_asset_verbs.nothing_overridden" => "Nothing overridden here",
        "shell.fase_sheet_verbs.auto_arrange_select_a" => {
            "Auto-Arrange: select a sheet - to make one, use Pack into Sheet"
        }
        "shell.fase_sheet_verbs.sheet_select_at_least" => "Sheet: select at least one sprite first",
        "shell.fase_sheet_verbs.pack_into_sheet_that" => {
            "Pack into Sheet: that is already a sheet - use Auto-Arrange Pieces"
        }
        "shell.fase_signal_outbox.signal" => "Signal: {sig}",
        "shell.fase_sprite_precision_emissive.remove_from_sheet_this" => {
            "Remove from Sheet: this object is not in a sheet"
        }
        "shell.fase_sprite_precision_emissive.removed_from_sheet" => "Removed from sheet",
        "shell.fase_tool_mirrors.layer_1" => "Layer 1",
        "shell.fase_tool_mirrors.flip" => "Flip",
        "shell.fase_world_panel_bridges.modelling_took_the" => "Modelling took the canvas",
        "shell.fase_world_panel_bridges.modelling_stepped" => {
            "Modelling stepped aside for the other tool"
        }
        "shell.hierarchy.that_prefab_is_no" => "That prefab is no longer in the project",
        "shell.hierarchy.view_zero_camera_reset" => "View · Zero (camera reset)",
        "shell.hierarchy.transform_reset" => "Transform reset",
        "shell.hierarchy.added_empty_object" => "Added empty object",
        "shell.hierarchy.added_child_entity" => "Added child entity",
        "shell.hierarchy.child" => "Child",
        "shell.hierarchy_add_root.object" => "Object",
        "shell.hierarchy_delete.that_piece_comes_from" => {
            "That piece comes from a prefab \u{2014} delete it in the prefab, or Detach this copy first"
        }
        "shell.hierarchy_delete.deleted_removed_from_2" => {
            "Deleted {n} \u{2014} {refused} removed from this copy \u{2014} {kept} stayed"
        }
        "shell.hierarchy_delete.deleted_removed_from" => {
            "Deleted {n} \u{2014} {r} removed from this copy only"
        }
        "shell.hierarchy_delete.deleted_entities" => "Deleted {n} entities",
        "shell.hierarchy_delete.removed_piece_s_from" => "Removed {r} piece(s) from this copy only",
        "shell.hierarchy_delete.removed_from_this_copy" => {
            "Removed from this copy \u{2014} the prefab still has it"
        }
        "shell.hierarchy_delete.deleted_entity" => "Deleted entity",
        "shell.hierarchy_duplicate.duplicated_entity" => "Duplicated entity",
        "shell.hierarchy_duplicate.duplicated_inside_the" => {
            "Duplicated inside the prefab — it shows while the prefab is selected"
        }
        "shell.hierarchy_duplicate.duplicated_as_a_plain" => {
            "Duplicated as a plain object — use Make Component for a second component"
        }
        "shell.hierarchy_duplicate.duplicated_shape" => "Duplicated shape",
        "shell.hierarchy_rename.renamed_to" => "Renamed to {final_name}",
        "shell.hierarchy_rename.name_in_use_renamed_to" => "Name in use — renamed to {final_name}",
        "shell.image_edit.painter_apply_tool_was" => {
            "Painter Apply: tool was inactive when bake fired (gate desynced)"
        }
        "shell.inspector_action.this_object_already" => {
            "This object already has the maximum of {SIGNAL_ACTIONS_MAX} actions."
        }
        "shell.inspector_anchor.this_sprite_already" => {
            "this sprite already has {anchors_max} anchors"
        }
        "shell.inspector_anchor.that_name_is_already" => "that name is already used",
        "shell.inspector_anchor.the_name_has_a_control" => "the name has a control character",
        "shell.inspector_anchor.the_name_is_over_bytes" => {
            "the name is over {anchor_name_max_bytes} bytes"
        }
        "shell.inspector_anchor.the_name_is_empty" => "the name is empty",
        "shell.inspector_anchor.anchor_name_is_already" => {
            "Anchor name '{new_name}' is already used on this sprite"
        }
        "shell.inspector_anchor.anchor_name_rejected" => "Anchor name rejected: {e}",
        "shell.inspector_anchor.anchor_not_added" => "Anchor not added: {e}",
        "shell.inspector_anim.this_sprite_already" => {
            "this sprite already has {anim_tags_max} animations"
        }
        "shell.inspector_anim.that_name_is_already" => "that name is already used",
        "shell.inspector_anim.the_name_has_a_control" => "the name has a control character",
        "shell.inspector_anim.the_name_is_over_bytes" => {
            "the name is over {anim_name_max_bytes} bytes"
        }
        "shell.inspector_anim.the_name_is_empty" => "the name is empty",
        "shell.inspector_anim.animation_name_is" => {
            "Animation name '{new_name}' is already used on this sprite"
        }
        "shell.inspector_anim.animation_name" => "Animation name rejected: {e}",
        "shell.inspector_anim.animation_not_added" => "Animation not added: {e}",
        "shell.inspector_audio.one_source_can_hold_at" => {
            "One source can hold at most {AUDIO_MAX_POLYPHONY} voices \u{2014} the mixer's \
                     pool is shared with the whole scene."
        }
        "shell.inspector_commits.is_not_registered" => "{type_name} is not registered",
        "shell.inspector_commits.signal_encode_failed" => "Signal encode failed: {e}",
        "shell.inspector_commits.signal_commit_failed" => "Signal commit failed: {e}",
        "shell.inspector_commits.editor_queue_full" => "Editor queue full: {e}",
        "shell.inspector_commits.visibility_commit" => "Visibility commit failed: {e}",
        "shell.inspector_commits.physics_commit_failed" => "Physics commit failed: {e}",
        "shell.inspector_commits.blend_commit_failed" => "Blend commit failed: {e}",
        "shell.inspector_commits.n9_slice_commit_failed" => "9-Slice commit failed: {e}",
        "shell.inspector_commits.signal_action_commit" => "Signal action commit failed: {e}",
        "shell.inspector_commits.timer_commit_failed" => "Timer commit failed: {e}",
        "shell.inspector_commits.animation_commit" => "Animation commit failed: {e}",
        "shell.inspector_commits.anchor_commit_failed" => "Anchor commit failed: {e}",
        "shell.inspector_commits.sampling_commit_failed" => "Sampling commit failed: {e}",
        "shell.inspector_commits.ordering_commit_failed" => "Ordering commit failed: {e}",
        "shell.inspector_commits.name_encode_failed" => "Name encode failed: {e}",
        "shell.inspector_commits.name_commit_failed" => "Name commit failed: {e}",
        "shell.inspector_commits.visibility_encode" => "Visibility encode failed: {e}",
        "shell.inspector_commits.transform_encode" => "Transform encode failed: {e}",
        "shell.inspector_commits.transform_commit" => "Transform commit failed: {e}",
        "shell.inspector_commits.reimport_unavailable" => "Reimport unavailable for this source",
        "shell.inspector_commits.reimported_at_px_m_m" => {
            "Reimported at {px_per_m_0} px/m · {size_3} × {size2_3} m"
        }
        "shell.inspector_commits_sprite.sprite_encode_failed" => "Sprite encode failed: {e}",
        "shell.inspector_commits_sprite.sprite_commit_failed" => "Sprite commit failed: {e}",
        "shell.inspector_commits_sprite.editor_queue_full" => "Editor queue full: {e}",
        "shell.inspector_instance.component_no_longer_in" => "(component no longer in this build)",
        "shell.inspector_strategy.strategy_commit_failed" => "Strategy commit failed: {e}",
        "shell.inspector_strategy.editor_queue_full" => "Editor queue full: {e}",
        "shell.inspector_strategy.sprite_encode_failed" => "Sprite encode failed: {e}",
        "shell.inspector_strategy.strategy_atlas_cell" => "Strategy · Atlas (cell {key})",
        "shell.inspector_strategy.cannot_pack_into_the_2" => {
            "Cannot pack {w}×{h} into the shared atlas: {e}. Keep this sprite Individual."
        }
        "shell.inspector_strategy.the_shared_atlas_is" => {
            "the shared atlas is one texture, and it is 8-bit"
        }
        "shell.inspector_strategy.cannot_pack_into_the" => {
            "Cannot pack into the atlas — this sprite's pixels are unreadable"
        }
        "shell.inspector_strategy.strategy_individual" => {
            "Strategy · Individual (texture {texture_id})"
        }
        "shell.inspector_strategy.individual_acquire" => "Individual acquire failed: {err}",
        "shell.inspector_strategy.cannot_promote_to" => {
            "Cannot promote to Individual — source asset missing"
        }
        "shell.inspector_strategy.hand_packed_comes_from" => {
            "Hand-packed comes from a sheet: right-click in the Hierarchy \u{00b7} Pack into Sheet, then Bake Sheet"
        }
        "shell.inspector_strategy.cooked_textures_come" => {
            "Cooked textures come from the asset pipeline — render strategy is read-only"
        }
        "shell.inspector_strategy.this_sprite_is_a_piece" => {
            "This sprite is a piece of a sheet \u{00b7} its pixels already live in an individual texture"
        }
        "shell.inspector_timer.a_timer_needs_a_name" => {
            "A timer needs a name — the list is how you pick one."
        }
        "shell.inspector_timer.this_object_already" => {
            "This object already has the maximum of {TIMERS_MAX} timers."
        }
        "shell.inspector_timer.timer" => "Timer",
        "shell.inspector_timer.timer_2" => "Timer {n}",
        "shell.present_bands.world_rt_sample_view" => "world RT sample view (sRGB)",
        "shell.snapshots.ent" => "ENT",
        "shell.snapshots_inspector_sprite.sprite_sheet" => "Sprite Sheet",
        "shell.snapshots_inspector_sprite.not_baked_yet" => "{name} \u{00b7} not baked yet",
        "shell.tokens_bridge_dtcg.unusable" => ", {dropped} unusable",
        "shell.tokens_bridge_dtcg.unknown" => ", {unknown} unknown",
        "shell.tokens_bridge_dtcg.already_at_factory" => ", {at_factory} already at factory",
        "shell.tokens_bridge_dtcg.dtcg_imported_token_s" => {
            "DTCG imported: {authored} token(s) authored"
        }
        "shell.tokens_bridge_dtcg.dtcg_import_failed" => "DTCG import failed: {e}",
        "shell.tokens_bridge_dtcg.dtcg_export_failed" => "DTCG export failed: {e}",
        "shell.tokens_bridge_dtcg.dtcg_exported_tokens" => "DTCG exported: {all} tokens to {path}",
        "shell.tokens_bridge_dtcg.dtcg_tokens" => "DTCG tokens",
        "shell.sculpt3d_host.sculpt" => "Sculpt",
        "shell.sculpt3d_host.sculpting_took_the_canvas" => "Sculpting took the canvas",
        "shell.texture_pattern_pick.image" => "Image",
        "shell.undo_app.sem_entrada_neste" => "sem entrada neste quadro",
        "shell.undo_app.transicao_de_estado_de" => "transicao de estado de UI ao vivo",
        "shell.undo_app.colorize_a_recalcular" => "colorize a recalcular",
        "shell.undo_app.arrasto_do_gizmo_3d_em" => "arrasto do gizmo 3D em curso",
        "shell.undo_app.botao_do_rato_em_baixo" => "botao do rato em baixo",
        "shell.undo_route.shift_z" => "Shift+Z",
        "shell.vec_component_general.that_copy_cannot" => "That copy cannot become this prefab",
        "shell.vec_component_general.those_two_prefabs_are" => {
            "Those two prefabs are unrelated \u{2014} use \u{201c}Replace selection with \
                 this\u{201d} in the library to choose how to match the pieces"
        }
        "shell.vec_component_general.it_is_already_a_copy" => "It is already a copy of that prefab",
        "shell.vec_component_general.piece_s_the_new_prefab" => {
            " \u{b7} {dropped} piece(s) the new prefab does not have were removed"
        }
        "shell.vec_component_general.now_a_copy_of_override" => {
            "Now a copy of \u{201c}{name}\u{201d} \u{2014} {overrides_kept} override(s) kept"
        }
        "shell.vec_component_general.that_shape_is_not_a" => {
            "That shape is not a copy of a prefab \u{2014} click one, or the open prefab"
        }
        "shell.vec_component_general.that_is_not_a_copy_of" => "That is not a copy of a prefab",
        "shell.vec_component_general.reverted_override_s_to" => {
            "Reverted {r} override(s) to the prefab"
        }
        "shell.vec_text.imported" => "Imported",
        "shell.vec_text.font_ttf_otf" => "Font (TTF / OTF)",
        "shell.vec_text_object.text" => "Text",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
