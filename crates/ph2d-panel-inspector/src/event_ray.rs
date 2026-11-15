//! **O despacho da secção RAY SENSOR** (suplente #21).
//!
//! ⚠️ **Irmão do [`crate::event`] por CAP de função** — o mesmo padrão das irmãs.
//!
//! # ⚠️ O CLIQUE afirma o contrário do que está no ECRÃ, e o ecrã vem do SNAPSHOT
//!
//! Nunca do store, que guarda o visual. É a lei que a §11 pagou com um report: ler o store fazia o
//! primeiro clique depois de trocar de objecto mandar o valor do objecto **anterior**.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::ray_edits::RayFieldEdit;

/// Despacha um evento da secção RAY SENSOR. `true` = consumido.
pub(crate) fn apply_ray_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = crate::state_components::current_inspector_ray() else {
        return false;
    };
    let bits = info.entity_bits;

    // A camada pela lista (a mesma do collider) e os quadros «Sees» (a linha da matriz do mundo).
    if let WidgetEvent::Click(id) = ev {
        if let Some(n) = crate::ids::INSP_RAY_LAYER_OPTS
            .iter()
            .position(|&o| o == id)
        {
            #[allow(clippy::cast_possible_truncation)]
            push(host, bits, RayFieldEdit::Layer(n as u32));
            return true;
        }
        let quem = &crate::sections::physics_colide::DO_RAIO;
        #[allow(clippy::cast_possible_truncation)]
        let a = info.layer.min(u32::from(u8::MAX)) as u8;
        if crate::sections::physics_colide::clique_da_fila(host, id, quem, a, true) {
            return true;
        }
        if id == crate::ids::INSP_RAY_TURNS {
            ph2d_editor_core::panel::seam_reset_button(host, id);
            push(
                host,
                bits,
                RayFieldEdit::TurnsWithFacing(!info.turns_with_facing),
            );
            return true;
        }
        if let Some(edit) = clique_do_filtro(id, &info) {
            ph2d_editor_core::panel::seam_reset_button(host, id);
            push(host, bits, edit);
            return true;
        }
        // RS-6 «Enabled» e os conta-gotas do «Aim at» e do «Also ignore»; o «×» de um chip tira-o.
        let mais = match id {
            crate::ids::INSP_RAY_ENABLED => Some(RayFieldEdit::Enabled(!info.enabled)),
            crate::ids::INSP_RAY_AIM_PICK => Some(RayFieldEdit::PickAimAt(!info.aim_pick_armed)),
            crate::ids::INSP_RAY_IGNORE_PICK => {
                Some(RayFieldEdit::PickIgnore(!info.ignore_pick_armed))
            }
            _ => crate::ids::INSP_RAY_IGNORE_CHIP
                .posicao(id, info.also_ignore.len())
                .map(RayFieldEdit::IgnoreRemove),
        };
        if let Some(edit) = mais {
            ph2d_editor_core::panel::seam_reset_button(host, id);
            push(host, bits, edit);
            return true;
        }
        // O conta-gotas do «Ignore up to» (C8): arma ou desarma; o clique no palco é da app.
        if id == crate::ids::INSP_RAY_UP_TO_PICK {
            ph2d_editor_core::panel::seam_reset_button(host, id);
            push(host, bits, RayFieldEdit::PickUpTo(!info.up_to_pick_armed));
            return true;
        }
        // «Open in tab…»: a aba central na camada do RAIO (emenda 4).
        if id == crate::ids::INSP_RAY_EDIT_MATRIX {
            ph2d_editor_core::panel::seam_reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::LayerMatrixOpen { layer: a });
            return true;
        }
    }

    if let WidgetEvent::TextChanged(id) = ev {
        // ⚠️ **O NOME cru** — quem o apara é quem o lê (a regra do `SignalOnHit`).
        let nome = host.store().text(id).unwrap_or("").to_string();
        let edit = match id {
            crate::ids::INSP_RAY_ON_ENTER => RayFieldEdit::OnEnter(nome),
            crate::ids::INSP_RAY_ON_EXIT => RayFieldEdit::OnExit(nome),
            crate::ids::INSP_RAY_UP_TO => RayFieldEdit::IgnoreUpTo(nome),
            crate::ids::INSP_RAY_AIM => RayFieldEdit::AimAt(nome),
            _ => return false,
        };
        push(host, bits, edit);
        return true;
    }

    if let WidgetEvent::ValueChanged(id) = ev {
        let v = crate::unidade_mundo::le(host, id).unwrap_or(0.0);
        #[allow(clippy::cast_possible_truncation)]
        let f = v as f32;
        // ⚠️ **Um `match` e não um `if` por campo** — com `if`s, o terceiro acaba a escrever no
        // primeiro (a lição dos três campos de texto da tabela de acções).
        let edit = match id {
            crate::ids::INSP_RAY_ANGLE => RayFieldEdit::Angle(f),
            crate::ids::INSP_RAY_REACH => RayFieldEdit::Reach(f),
            crate::ids::INSP_RAY_THICKNESS => RayFieldEdit::Thickness(f),
            crate::ids::INSP_RAY_CONE => RayFieldEdit::Cone(f),
            _ => return false,
        };
        push(host, bits, edit);
        return true;
    }
    false
}

/// O grupo «Filter»: cada caixa escreve o CONTRÁRIO do snapshot; as peças, o modo; as opções, a tag.
fn clique_do_filtro(
    id: ph2d_a11y::NodeId,
    i: &ph2d_editor_core::ray_edits::InspectorRayInfo,
) -> Option<RayFieldEdit> {
    use crate::ids as d;
    let caixa = match id {
        d::INSP_RAY_IGNORE_BODY => RayFieldEdit::IgnoreBody(!i.ignore_body),
        d::INSP_RAY_IGNORE_PARTS => RayFieldEdit::IgnoreParts(!i.ignore_parts),
        d::INSP_RAY_WALLS_BLOCK => RayFieldEdit::WallsBlock(!i.walls_block),
        d::INSP_RAY_HIT_TRIGGERS => RayFieldEdit::HitTriggers(!i.hit_triggers),
        d::INSP_RAY_STARTS_INSIDE => RayFieldEdit::StartsInside(!i.starts_inside),
        d::INSP_RAY_ALL_HITS => RayFieldEdit::AllHits(!i.all_hits),
        _ => {
            if let Some(n) = d::INSP_RAY_ONLY_SEE.iter().position(|&o| o == id) {
                return Some(RayFieldEdit::OnlySee(
                    crate::sections::ray_filtro::MODOS[n].0,
                ));
            }
            let n = d::INSP_RAY_SEE_TAG_OPT.posicao(id, crate::state::current_tag_tree_len())?;
            return crate::sections::ray_filtro::opcoes_da_tag()
                .get(n)
                .map(|o| RayFieldEdit::SeeTag(o.value));
        }
    };
    Some(caixa)
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: RayFieldEdit) {
    host.bus_mut().push(EditorAction::InspectorComponentEdit {
        entity_bits,
        edit: ComponentEdit::Ray(edit),
    });
}
