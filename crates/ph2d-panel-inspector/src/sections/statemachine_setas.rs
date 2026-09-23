//! **As SETAS da máquina de estados** — a lista de transições e o editor delas.
//!
//! ⚠️ **Irmão por tecto de LOC** (600 por ficheiro de painel): a migração do HR-15 de
//! 2026-09-16 alongou cada rótulo (`tr("chave")` no lugar do literal) e o ficheiro passou
//! o tecto. O corte é por RESPONSABILIDADE, que é o que o tecto pede.

use super::*;

/// **A lista de TRANSIÇÕES, os botões e o editor da linha aberta** — irmã da de cima.
#[allow(clippy::too_many_arguments)]
pub(super) fn setas(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorStateMachineInfo,
    selected: usize,
) -> f32 {
    let mut cur_y = y;
    // ── TRANSIÇÕES ───────────────────────────────────────────────────────────
    if !info.transitions.is_empty() {
        let linhas: Vec<(String, bool)> = info
            .transitions
            .iter()
            .map(|t| (resumo_seta(t, &info.states), t.on.is_empty()))
            .collect();
        cur_y = lista(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            &linhas,
            &crate::ids::INSP_SM_TRANS_ROW,
            selected,
        );
    }
    cur_y = botoes(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        (
            crate::ids::INSP_SM_TRANS_ADD,
            tr("panel.inspector.statemachine.add_transition"),
        ),
        (
            crate::ids::INSP_SM_TRANS_REMOVE,
            tr("panel.inspector.statemachine.x_remove_transition"),
        ),
        info.transitions.len() < crate::ids::INSP_SM_TRANS_ROW.len(),
        !info.transitions.is_empty(),
    );
    if let Some(t) = info.transitions.get(selected) {
        // ⭐⭐ **A coluna do nome é da SECÇÃO** (`line/UIUX`, 2026-09-15): esta secção nasceu
        //    contra a porta antiga (`anchors::field_row`, o nome POR CIMA do campo) e passa à
        //    única que existe. ⚠️ Os nomes são os da secção INTEIRA, inclusive os das linhas que
        //    este quadro não pinta.
        let seccao = ph2d_editor_core::property_row::Seccao::medida(
            text_system,
            1,
            &[
                tr("panel.inspector.statemachine.from_state"),
                tr("panel.inspector.statemachine.to_state"),
            ],
        );
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            tr("panel.inspector.statemachine.from_state"),
            &[crate::ids::INSP_SM_TRANS_FROM],
            1.0, // LITERAL-PX-OK: índice
            None,
            seccao,
        );
        cur_y = super::anim_rows::text_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            tr("panel.inspector.statemachine.on_label"),
            crate::ids::INSP_SM_TRANS_ON,
            TextInput::new(crate::ids::INSP_SM_TRANS_ON, "")
                .placeholder(tr("panel.inspector.statemachine.on_signal_u")),
            seccao,
        );
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            tr("panel.inspector.statemachine.to_state"),
            &[crate::ids::INSP_SM_TRANS_TO],
            1.0, // LITERAL-PX-OK: índice
            None,
            seccao,
        );
        // ⚠️⚠️ **A LINHA QUE RESPONDE AO «não acontece nada»** — a mesma da tabela de acções.
        if t.on.is_empty() {
            cur_y = super::rows::aviso(
                scene,
                text_system,
                theme,
                x,
                w,
                cur_y,
                tr(
                    "panel.inspector.statemachine.this_transition_never_fires_it_has_no_signal_name",
                ),
                ColorToken::Warn,
            );
        }
    }

    cur_y
}
