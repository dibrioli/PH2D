//! **O despacho da secção PARALLAX** (plano 24, W7).
//!
//! ⚠️ **Irmão do [`crate::event`] por CAP de função** — o mesmo padrão das irmãs.
//!
//! # ⚠️ O CLIQUE afirma o contrário do que está no ECRÃ, e o ecrã vem do SNAPSHOT
//!
//! Nunca do store, que guarda o visual. É a lei que a §11 pagou com um report: ler o store fazia o
//! primeiro clique depois de trocar de objecto mandar o valor do objecto **anterior**.
//!
//! ⭐⭐ **E é por isso que cada variante leva o PAR:** um eixo de cada vez obrigaria o drenador a ir
//! buscar o outro ao mundo, um quadro atrás do que o painel mostra.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::parallax_edits::ParallaxFieldEdit;

/// Despacha um evento da secção PARALLAX. `true` = consumido.
pub(crate) fn apply_parallax_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = crate::state_components::current_inspector_parallax() else {
        return false;
    };
    let WidgetEvent::ValueChanged(id) = ev else {
        return false;
    };
    let v = host.store().number_value(id).unwrap_or(0.0);
    #[allow(clippy::cast_possible_truncation)]
    let f = v as f32;
    // ⚠️ **O outro eixo vem do SNAPSHOT**, e os três blocos opcionais caem para o neutro quando o
    // componente não está lá — ⛔ nunca para um valor «plausível»: sem o componente esta rota nem
    // chega a ser alcançável, porque o painel não pinta a fileira.
    let rep = info.repeat.unwrap_or([0.0, 0.0]);
    let mov = info.motion.unwrap_or([0.0, 0.0]);
    let (lmin, lmax) = info.limits.unwrap_or(([0.0, 0.0], [0.0, 0.0]));
    // ⚠️ **Um `match` e não um `if` por campo** — com `if`s, o terceiro acaba a escrever no
    // primeiro (a lição dos três campos de texto da tabela de acções).
    let edit = match id {
        crate::ids::INSP_PARALLAX_K_X => ParallaxFieldEdit::Factor([f, info.factor[1]]),
        crate::ids::INSP_PARALLAX_K_Y => ParallaxFieldEdit::Factor([info.factor[0], f]),
        crate::ids::INSP_PARALLAX_TILE_X => ParallaxFieldEdit::Repeat([f, rep[1]]),
        crate::ids::INSP_PARALLAX_TILE_Y => ParallaxFieldEdit::Repeat([rep[0], f]),
        crate::ids::INSP_PARALLAX_VEL_X => ParallaxFieldEdit::Motion([f, mov[1]]),
        crate::ids::INSP_PARALLAX_VEL_Y => ParallaxFieldEdit::Motion([mov[0], f]),
        crate::ids::INSP_PARALLAX_MIN_X => ParallaxFieldEdit::LimitsMin([f, lmin[1]]),
        crate::ids::INSP_PARALLAX_MIN_Y => ParallaxFieldEdit::LimitsMin([lmin[0], f]),
        crate::ids::INSP_PARALLAX_MAX_X => ParallaxFieldEdit::LimitsMax([f, lmax[1]]),
        crate::ids::INSP_PARALLAX_MAX_Y => ParallaxFieldEdit::LimitsMax([lmax[0], f]),
        _ => return false,
    };
    host.bus_mut().push(EditorAction::InspectorComponentEdit {
        entity_bits: info.entity_bits,
        edit: ComponentEdit::Parallax(edit),
    });
    true
}
