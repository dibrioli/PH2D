//! ⭐ **A TAG do modo `Tag` do NAV AGENT** (plano 30, W6) — o chip, a lista e o aviso da tag apagada.
//! Irmão de [`super::nav`] por tecto de LOC, e por ASSUNTO: esta row pergunta à árvore das tags do
//! projecto, que nenhuma outra da secção conhece.
//!
//! ⛔ **A tag APAGADA não alcança ninguém** (a ponte falha FECHADO), e a row di-lo em WARN — a lei
//! do filtro *Only for tag* da §11 (`physics_tag_row`).

use super::*;
use ph2d_editor_core::property_row::{Seccao, paint_label_row};
use ph2d_editor_core::widget::{Dropdown, DropdownOption, paint_dropdown_chip};
use ph2d_i18n::tr;

/// A row da tag. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn tag_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    tag: u64,
    seccao: Seccao,
) -> f32 {
    let linha = paint_label_row(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        ROW_H_PX,
        tr("panel.inspector.nav.target_tag"),
        seccao,
    );
    hit_index.register(ids::INSP_NAV_TAG_PICK, linha.control);
    let aberto = matches!(
        store.get(ids::INSP_NAV_TAG_PICK),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let opcoes = nav_tag_options();
    let viva = opcoes.iter().any(|o| o.value == tag);
    let mut dd = Dropdown::new(ids::INSP_NAV_TAG_PICK, "", opcoes)
        .placeholder(tr("panel.inspector.nav.pick_a_tag_u"))
        .open(aberto)
        .visual(store.dropdown_visual(ids::INSP_NAV_TAG_PICK));
    // ⚠️ **A escolha vem do SNAPSHOT e o `open` do store** (a lei do chip da comparação).
    if tag != 0 {
        dd.select(tag);
    }
    paint_dropdown_chip(&dd, linha.control, scene, text_system, theme);
    if aberto {
        crate::state_popovers::set_pending_nav_tag_dd(Some(linha.control));
    }
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, linha.dot);
    let mut cur_y = y + ph2d_tokens::row_pitch_px();
    if tag != 0 && !viva {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.nav.that_tag_was_deleted"),
            ColorToken::Warn,
        );
    }
    cur_y
}

/// **As opções: a árvore inteira do projecto**, indentada pela profundidade.
///
/// ⚠️ `pub(crate)` porque o passe diferido e o despacho a re-derivam — a lei do popover.
pub(crate) fn nav_tag_options() -> Vec<DropdownOption<u64>> {
    crate::state::current_tag_tree()
        .iter()
        .zip(ids::INSP_NAV_TAG_OPT.iter())
        .map(|(row, &id)| {
            let recuo = "    ".repeat(row.depth);
            DropdownOption::new(id, row.id, format!("{recuo}{}", row.label))
        })
        .collect()
}
