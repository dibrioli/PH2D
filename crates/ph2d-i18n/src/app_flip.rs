//! **O QUE A FAMÍLIA app.flip DIZ** — os avisos (toasts), as recusas e os rótulos que a crate
//! `ph2d-app-flip` mostra (dos VERBOS do Flip (pintar, preencher, colorir, esculpir)), na forma `app.flip.<ficheiro>.<frase>`.
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

/// A tradução de uma chave `app.flip.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "app.flip.colorize.colorize_no_regions_scribble_inside_the_closed_s" => {
            "Colorize: no regions — scribble inside the closed shapes"
        }
        "app.flip.colorize.colorize_draw_the_line_art_first" => "Colorize: draw the line-art first",
        "app.flip.colorize.colorize_the_layer_is_locked_or_has_no_drawing_o" => {
            "Colorize: the layer is locked, or has no drawing on this frame"
        }
        "app.flip.fill.fill_trap_is_wider_than_this_area_lower_it" => {
            "Fill: Trap is wider than this area — lower it"
        }
        "app.flip.fill.fill_no_region_under_the_cursor" => "Fill: no region under the cursor",
        "app.flip.fill.fill_nothing_to_fill_here" => "Fill: nothing to fill here",
        "app.flip.fill.fill_clicked_on_a_line" => "Fill: clicked on a line",
        "app.flip.fill.fill_leaked_raise_gap_closure_to_seal_the_outlin" => {
            "Fill leaked — raise Gap Closure to seal the outline"
        }
        "app.flip.fill.fill_the_layer_is_locked_or_has_no_drawing_on_th" => {
            "Fill: the layer is locked, or has no drawing on this frame"
        }
        "app.flip.reshape.sculpt_the_layer_is_locked_or_has_no_drawing_on" => {
            "Sculpt: the layer is locked, or has no drawing on this frame"
        }
        "app.flip.select.point_move_needs_exclusive_art_unlink_the_key_fi" => {
            "Point move needs exclusive art - Unlink the key first"
        }
        "app.flip.select.edit_the_layer_is_locked_or_has_no_drawing_on_th" => {
            "Edit: the layer is locked, or has no drawing on this frame"
        }
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
