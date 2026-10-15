//! ⭐ **O SELETOR dos scripts do projeto** no cartão Script (CodeEditor E11): um chip que lista os
//! scripts da biblioteca; escolher um aponta-o — o mesmo `Source` que escrever o nome.
//!
//! As opções partilham a tabela `INSP_SCRIPT_ENUM_OPT` com os chips de enum, e a cerca é a mesma: um
//! popover de cada vez, a linha guardada pelo passe diferido. A «linha» deste chip é
//! [`LINHA_DO_SELETOR`], que nenhuma propriedade tem.

use super::*;
use ph2d_editor_core::script_edits::InspectorScriptInfo;
use ph2d_editor_core::widget::{Dropdown, DropdownOption, paint_dropdown_chip};
use ph2d_i18n::tr;

/// A linha que o passe diferido guarda quando o popover aberto é o do seletor.
pub const LINHA_DO_SELETOR: usize = usize::MAX;

/// As opções do seletor — a PORTA, lida pelo chip, pelo passe diferido e pelo despacho. O rótulo é
/// o nome que o artista deu ao script (dado dele, não língua desta casa).
#[must_use]
pub fn opcoes_do_seletor(info: &InspectorScriptInfo) -> Vec<DropdownOption<usize>> {
    info.library
        .iter()
        .enumerate()
        .take(ids::INSP_SCRIPT_ENUM_OPT.len())
        .map(|(j, nome)| DropdownOption::new(ids::INSP_SCRIPT_ENUM_OPT[j], j, nome.clone()))
        .collect()
}

/// O chip, pronto (o escolhido marcado, o rótulo vazio quando nenhum).
#[must_use]
pub fn chip(info: &InspectorScriptInfo, aberto: bool) -> Dropdown<usize> {
    let mut dd = Dropdown::new(ids::INSP_SCRIPT_PICK, "", opcoes_do_seletor(info))
        .placeholder(tr("panel.inspector.script.choose_a_project_script_u"))
        .open(aberto);
    if let Some(j) = info.library.iter().position(|n| *n == info.source) {
        dd.select(j);
    }
    dd
}

/// Pinta o seletor na fileira `r` (apagado sem scripts no projeto), com a coluna da animação
/// reservada como toda fileira de formulário.
#[allow(clippy::too_many_arguments)]
pub(super) fn seletor(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    r: Rect,
    info: &InspectorScriptInfo,
) {
    let (control_w, dot) = ph2d_editor_core::widget::form_row_columns(r.x, r.w, r.y, r.h);
    let r = Rect::new(r.x, r.y, control_w, r.h);
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, dot);
    if info.library.is_empty() {
        paint_button(
            &Button::new(
                ids::INSP_SCRIPT_PICK,
                tr("panel.inspector.script.no_scripts_in_the_project"),
            )
            .visual((ButtonState::Disabled, ph2d_editor_core::motion::SETTLED)),
            r,
            scene,
            text_system,
            theme,
        );
        return;
    }
    hit_index.register(ids::INSP_SCRIPT_PICK, r);
    let aberto = matches!(
        store.get(ids::INSP_SCRIPT_PICK),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let dd = chip(info, aberto).visual(store.dropdown_visual(ids::INSP_SCRIPT_PICK));
    paint_dropdown_chip(&dd, r, scene, text_system, theme);
    if aberto {
        crate::state_popovers::set_pending_script_enum(Some((LINHA_DO_SELETOR, r)));
    }
}
