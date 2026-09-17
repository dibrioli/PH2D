//! **O QUE A FAMÍLIA app.field3d DIZ** — os avisos (toasts), as recusas e os rótulos que a crate
//! `ph2d-app-field3d` mostra (do MODELADOR de campo (as peças, a paleta de formas, importar/exportar)), na forma `app.field3d.<ficheiro>.<frase>`.
//!
//! ⚠️ **Frases com peças do código usam [`crate::tr_with`] com marcadores NOMEADOS** — uma língua
//! pode reordenar os marcadores; não pode mudar o que eles valem.
//!
//! ⛔ **O que NÃO está aqui, de propósito** (cada um com excepção NOMEADA no gate da crate): as
//! cenas de smoke, o diagnóstico de consola, os formatos de ficheiro e os **nomes por omissão de
//! objecto** — um nome que entra no `Name` é identidade durável (`stable_name_id` fecha um hash
//! sobre ele), e traduzi-lo é decisão do dono, não desta migração.
//!
//! ⚠️ **Os braços entre os marcadores são do script** — uma chave nova escreve-se à mão FORA deles.

/// A tradução de uma chave `app.field3d.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "app.field3d.export.export_failed" => "Export failed: {e}",
        "app.field3d.export.exported_quads_tris_x_x_kb_in_ms" => {
            "Exported {quads} quads = {tris} tris, {sx_2} x {sy_2} x {sz_2}, \
                 {size} KB in {ms_0} ms -- {name} ({fmt}){sitio}{quality}"
        }
        "app.field3d.export.retopology_skew" => " · retopology: {skew_p50_1}° skew",
        "app.field3d.export.meshing_failed" => "Meshing failed: {e_}",
        "app.field3d.export.could_not_start_the_export" => "Could not start the export",
        "app.field3d.export.exporting_the_file_is_being_written" => {
            "Exporting... the file is being written"
        }
        "app.field3d.export.unknown_extension_use" => "Unknown extension: use {join}",
        "app.field3d.export.nothing_to_export_the_part_is_empty" => {
            "Nothing to export: the part is empty"
        }
        "app.field3d.export.an_export_is_already_running" => "An export is already running",
        "app.field3d.export.at" => " · at ({c_2}, {c2_2}, {c3_2})",
        "app.field3d.import.relinked_to_tris_field_in_ms" => {
            "Relinked to {name}: {tris} tris -> field in {ms_0} ms"
        }
        "app.field3d.import.could_not_relink_to" => "Could not relink to {name}: {e}",
        "app.field3d.import.imported_tris_field_in_ms_detail" => {
            "Imported {name}: {tris} tris -> field in {ms_0} ms (detail {cell_4})"
        }
        "app.field3d.import.could_not_import" => "Could not import {name}: {e}",
        "app.field3d.import.scene_sculpture_in_tris_field_in_ms_detail" => {
            "Scene sculpture in: {tris} tris -> field in {ms_0} ms (detail {cell_4})"
        }
        "app.field3d.import.could_not_use_the_scene_sculpture" => {
            "Could not use the scene sculpture: {e}"
        }
        "app.field3d.import.that_mesh_is_empty" => "that mesh is empty",
        "app.field3d.import.could_not_merge_its_pieces" => "could not merge its pieces ({e_})",
        "app.field3d.import.could_not_read_it" => "could not read it ({e})",
        "app.field3d.import.that_file_has_no_mesh_in_it" => "that file has no mesh in it",
        "app.field3d.notice.a_sculpture_cannot_take_shell_offset_mirror_or_t" => {
            "A sculpture cannot take shell, offset, mirror or the other modifiers"
        }
        "app.field3d.notice.a_sculpture_here_has_no_file_behind_it" => {
            "A sculpture here has no file behind it"
        }
        "app.field3d.notice.the_drawn_profile_crosses_the_axis_it_turns_arou" => {
            "The drawn profile crosses the axis it turns around"
        }
        "app.field3d.notice.a_shape_here_has_an_impossible_size" => {
            "A shape here has an impossible size"
        }
        "app.field3d.notice.the_rounding_here_is_bigger_than_the_shape_can_t" => {
            "The rounding here ({round_3}) is bigger than the shape can take ({limit_3})"
        }
        "app.field3d.notice.a_shape_here_has_a_of_zero_or_less" => {
            "A shape here has a {what} of zero or less"
        }
        "app.field3d.notice.an_operation_here_has_nothing_left_to_combine" => {
            "An operation here has nothing left to combine"
        }
        "app.field3d.notice.two_parts_of_this_piece_point_at_each_other_in_a" => {
            "Two parts of this piece point at each other in a loop"
        }
        "app.field3d.notice.this_piece_has_nothing_the_model_can_start_from" => {
            "This piece has nothing the model can start from"
        }
        "app.field3d.profile.this_outline_cannot_become_a_solid_it_may_cross" => {
            "This outline cannot become a solid — it may cross itself or be too small"
        }
        "app.field3d.profile.this_shape_has_no_points" => "This shape has no points",
        "app.field3d.profile.this_shape_is_open_close_it_before_making_it_sol" => {
            "This shape is open — close it before making it solid"
        }
        "app.field3d.profile.were_gone_and_one_was_skipped" => {
            "{head}. {n} were gone, and one was skipped: {why}"
        }
        "app.field3d.profile.were_no_longer_in_the_scene" => {
            "{head}. {n} were no longer in the scene"
        }
        "app.field3d.profile.one_was_skipped" => "{head}. One was skipped: {why}",
        "app.field3d.profile.shapes_edges" => "{verb} {made} shapes{axis} ({edges} edges)",
        "app.field3d.profile.the_shape_edges" => "{verb} the shape{axis} ({edges} edges)",
        "app.field3d.profile.around_y" => " around Y",
        "app.field3d.profile.revolved" => "Revolved",
        "app.field3d.profile.extruded" => "Extruded",
        "app.field3d.profile.the_selected_shape_is_no_longer_in_the_scene" => {
            "The selected shape is no longer in the scene"
        }
        "app.field3d.profile.draw_and_select_a_closed_shape_first" => {
            "Draw and select a closed shape first"
        }
        "app.field3d.profile_live.the_drawing_this_shape_came_from_is_gone_the_sha" => {
            "The drawing this shape came from is gone - the shape keeps its last form"
        }
        "app.field3d.reload.sculpture_is_missing" => "Sculpture {name} is missing: {e}",
        "app.field3d.scene.isolation_off_the_whole_part_is_back" => {
            "Isolation off: the whole part is back"
        }
        "app.field3d.scene.isolated_showing_only_this_object_shift_i_brings" => {
            "Isolated: showing only this object (Shift+I brings the part back)"
        }
        "app.field3d.scene.isolation_dropped_that_object_is_gone" => {
            "Isolation dropped: that object is gone"
        }
        "app.field3d.scene_acts.lnk" => "LNK",
        "app.field3d.scene_acts.iso" => "ISO",
        "app.field3d.scene_intents.not_available_right_now" => "{shape}: not available right now",
        "app.field3d.scene_intents.group_created_with_object_s_inside" => {
            "Group created with {many} object(s) inside"
        }
        "app.field3d.scene_intents.linked_this_shape_now_follows_the_selected_drawi" => {
            "Linked: this shape now follows the selected drawing"
        }
        "app.field3d.scene_intents.unlinked_this_shape_no_longer_follows_the_drawing" => {
            "Unlinked: this shape no longer follows the drawing"
        }
        "app.field3d.scene_intents.isolation_off_the_whole_part_is_back" => {
            "Isolation off: the whole part is back"
        }
        "app.field3d.scene_intents.isolated_showing_only_this_object" => {
            "Isolated: showing only this object"
        }
        "app.field3d.scene_verb.int" => "INT",
        "app.field3d.scene_verb.sub" => "SUB",
        "app.field3d.scene_verb.uni" => "UNI",
        "app.field3d.scene_verb.bse" => "BSE",
        "app.field3d.shape_palette.add_shape" => "Add Shape",
        "app.field3d.shape_palette.needs_something_selected" => "Needs something selected",
        "app.field3d.shape_palette.not_available_right_now" => "not available right now",
        "app.field3d.shape_palette.there_is_no_sculpture_in_the_scene_yet" => {
            "there is no sculpture in the scene yet"
        }
        "app.field3d.shape_palette.pick_a_closed_outline_in_the_vector_editor_first" => {
            "pick a closed outline in the vector editor first"
        }
        "app.field3d.shapes.imported" => "Imported",
        "app.field3d.shapes.from_a_drawing" => "From a drawing",
        "app.field3d.shapes.signs_and_symbols" => "Signs & symbols",
        "app.field3d.shapes.plates" => "Plates",
        "app.field3d.shapes.rings_and_tubes" => "Rings & tubes",
        "app.field3d.shapes.round" => "Round",
        "app.field3d.shapes.blocks" => "Blocks",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
