//! **O QUE A FAMÍLIA app.components DIZ** — os avisos (toasts), as recusas e os rótulos que a crate
//! `ph2d-app-components` mostra (das INSTÂNCIAS e dos componentes), na forma `app.components.<ficheiro>.<frase>`.
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

/// A tradução de uma chave `app.components.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "app.components.component_attach.attach_failed" => "Attach failed: {e}",
        "app.components.component_attach.has_no_default_to_attach" => {
            "{name} has no default to attach"
        }
        "app.components.component_attach.unknown_component" => "Unknown component: {name}",
        "app.components.component_attach.no_such_entity" => "No such entity",
        "app.components.component_palette.show_all" => "Show all",
        "app.components.component_palette.add_component" => "Add Component",
        "app.components.component_palette.not_for_this_object_type" => "Not for this object type",
        "app.components.component_palette.not_for_this_object_type_2" => {
            "  \u{2014}  not for this object type"
        }
        "app.components.component_palette.brings" => "  \u{2014}  brings ",
        "app.components.instance_open.editing_move_a_piece_and_every_copy_follows" => {
            "Editing \u{201c}{name}\u{201d} \u{2014} move a piece and every copy follows"
        }
        "app.components.instance_open.that_is_not_a_copy_of_a_prefab_pick_one_or_the_p" => {
            "That is not a copy of a prefab \u{2014} pick one, or the prefab row"
        }
        "app.components.instance_revert.reverted_override_s_to_the_prefab" => {
            "Reverted {r} override(s) to the prefab"
        }
        "app.components.instance_revert.reverted_change_s_position_kept" => {
            "Reverted {r} change(s) — position kept"
        }
        "app.components.instance_revert.only_the_position_differs_it_stays_where_you_put" => {
            "Only the position differs — it stays where you put it"
        }
        "app.components.instance_revert.nothing_overridden_here" => "Nothing overridden here",
        "app.components.instance_revert.put_back_piece_s_and_reverted_change_s" => {
            "Put back {pieces_back} piece(s) \u{2014} and reverted {r} change(s)"
        }
        "app.components.instance_revert.not_part_of_an_instance_nothing_to_revert" => {
            "Not part of an instance — nothing to revert"
        }
        "app.components.instance_verbs.that_is_not_in_the_library" => "That is not in the library",
        "app.components.instance_verbs.removed_from_library_it_had_no_copies_so_it_came" => {
            "Removed from library \u{2014} it had no copies, so it came back to the canvas"
        }
        "app.components.instance_verbs.removed_from_library_copy_ies_are_now_independen" => {
            "Removed from library \u{2014} {copies} copy(ies) are now independent"
        }
        "app.components.instance_verbs.not_part_of_an_instance" => "Not part of an instance",
        "app.components.instance_verbs.applied_change_s_to_the_prefab" => {
            "Applied {n} change(s) to the prefab"
        }
        "app.components.instance_verbs.nothing_overridden_here" => "Nothing overridden here",
        "app.components.instance_verbs.detached_piece_s_from_the_prefab" => {
            "Detached {n} piece(s) from the prefab"
        }
        "app.components.instance_verbs.not_a_prefab_pick_the_prefab_row" => {
            "Not a prefab — pick the prefab row"
        }
        "app.components.instance_verbs.that_would_put_the_prefab_inside_itself" => {
            "That would put the prefab inside itself"
        }
        "app.components.instance_verbs.instantiated" => "Instantiated",
        "app.components.instance_verbs.instantiated_linked_its_art_follows_the_prefab_b" => {
            "Instantiated linked — its art follows the prefab both ways"
        }
        "app.components.instance_verbs.inside_an_instance_detach_it_first_or_edit_the_p" => {
            "Inside an instance — detach it first, or edit the prefab"
        }
        "app.components.instance_verbs.inside_a_prefab_prefabs_cannot_nest_yet" => {
            "Inside a prefab — prefabs cannot nest yet"
        }
        "app.components.instance_verbs.this_is_already_a_prefab" => "This is already a prefab",
        "app.components.instance_verbs.made_a_prefab_an_instance_took_its_place" => {
            "Made a prefab \u{2014} an instance took its place"
        }
        "app.components.instance_verbs.made_a_variant_it_still_follows_its_base" => {
            "Made a variant \u{2014} it still follows its base"
        }
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
