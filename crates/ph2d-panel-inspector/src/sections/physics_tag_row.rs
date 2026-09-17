//! ⭐⭐⭐ **O QUE UM CORPO EMITE, E PARA QUEM** — as rows de sinal (*On Hit* / *On Leave*) e o
//! filtro *Only for tag* (TOP-20 #9, W3c). Irmão de [`super`] por tecto de LOC.
//!
//! ⚠️ **O corte é por ASSUNTO, não por tamanho:** o resto do `physics_rows` desenha o CORPO de um
//! objecto (massa, atrito, camada, forma, zona) e estas três rows desenham a **conversa** dele com
//! o resto da cena — e a do filtro pergunta à árvore do projecto, que nenhuma das vizinhas conhece.
//!
//! ⛔ **A tag APAGADA não passa ninguém** (falha FECHADA — ver o `SignalTagFilter`), e a row di-lo
//! em WARN: sem a frase, a armadilha cala-se e o artista procura o defeito no sinal.

use super::*;
use ph2d_editor_core::widget::{Dropdown, DropdownOption, paint_dropdown_chip};
use ph2d_i18n::tr;

/// A row do filtro por tag, e o aviso quando ele deixou de alcançar alguém. Devolve o `y` seguinte.
///
/// ⛔ **A tag APAGADA não passa ninguém** (falha fechada — ver o `SignalTagFilter`), e a row di-lo em
/// WARN: sem a frase, a armadilha cala-se e o artista procura o defeito no sinal.
#[allow(clippy::too_many_arguments)]
pub(super) fn signal_tag_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    signal_tag: Option<u64>,
    signal_tag_path: &str,
) -> f32 {
    let (control_w, dot) = ph2d_editor_core::widget::form_row_columns(x, w, y, ROW_H_PX);
    let rect = Rect::new(x, y, control_w, ROW_H_PX);
    hit_index.register(ids::INSP_PHYS_SIGNAL_TAG, rect);
    let open = matches!(
        store.get(ids::INSP_PHYS_SIGNAL_TAG),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let mut dd = Dropdown::new(ids::INSP_PHYS_SIGNAL_TAG, "", phys_tag_options())
        .placeholder(tr("panel.inspector.physics.only_for_tag_u_any"))
        .open(open)
        .visual(store.dropdown_visual(ids::INSP_PHYS_SIGNAL_TAG));
    if let Some(t) = signal_tag.filter(|t| *t != 0) {
        dd.select(t);
    }
    paint_dropdown_chip(&dd, rect, scene, text_system, theme);
    if open {
        crate::state_popovers::set_pending_phys_tag_dd(Some(rect));
    }
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, dot);
    let mut cur_y = y + ph2d_tokens::row_pitch_px();

    // ⛔ A tag foi apagada ⇒ a armadilha não grita com ninguém. ⚠️ Só com o id VIVO e o caminho
    // vazio: um filtro por escolher (`0`) passa todos e não é um defeito.
    if signal_tag.is_some_and(|t| t != 0) && signal_tag_path.is_empty() {
        let font = TypeToken::Sm.px();
        paint_text(
            text_system,
            scene,
            tr("panel.inspector.physics.that_tag_was_deleted_u_this_reaches_nobody"),
            x,
            cur_y,
            font,
            w,
            resolve(ColorToken::Warn, theme),
        );
        cur_y += font + ph2d_tokens::control_gap_px();
    }
    cur_y
}

/// **As opções: a árvore inteira do projecto**, indentada pela profundidade, com *(any)* à frente.
///
/// ⚠️ `pub(crate)` porque o passe diferido a re-deriva — a lei do popover.
pub(crate) fn phys_tag_options() -> Vec<DropdownOption<u64>> {
    // ⭐ A primeira entrada LIMPA o filtro. ⛔ Sem ela, anexar o componente seria um caminho sem
    // volta pelo painel — e um controlo que não se desfaz é pior que um que não existe.
    let mut out = vec![DropdownOption::new(
        ids::INSP_PHYS_SIGNAL_TAG_CLEAR,
        0u64,
        String::from(tr("panel.inspector.physics.any")),
    )];
    out.extend(
        crate::state::current_tag_tree()
            .iter()
            .zip(ids::INSP_PHYS_TAG_OPT.iter())
            .map(|(row, &id)| {
                let recuo = "    ".repeat(row.depth);
                DropdownOption::new(id, row.id, format!("{recuo}{}", row.label))
            }),
    );
    out
}

/// A row de um nome de sinal — um `TextInput`, porque o valor É uma string e o
/// contrato é por NOME (ADR-0143).
///
/// ⚠️ **Uma função, dois extremos.** Chegada e saída desenham a MESMA coisa e só
/// diferem no id e no placeholder; duas cópias divergiriam no dia em que uma
/// delas ganhasse um estado (foco, erro, dimmed) e a outra não.
#[allow(clippy::too_many_arguments)]
pub(super) fn signal_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    id: ph2d_a11y::NodeId,
    placeholder: &str,
) -> f32 {
    let (control_w, dot) = ph2d_editor_core::widget::form_row_columns(x, w, y, ROW_H_PX);
    let host = Rect::new(x, y, control_w, ROW_H_PX);
    hit_index.register(id, host);
    let (state, text, caret, anchor) = match store.get(id) {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, Some(text.as_str()), *caret, *selection_anchor),
        _ => (TextInputState::Normal, None, 0, None),
    };
    let input = TextInput::new(id, "")
        .placeholder(placeholder)
        .visual((state, store.hover_live(id)));
    paint_text_input_with_buffer(
        &input,
        text,
        Some(caret),
        anchor,
        host,
        scene,
        text_system,
        theme,
    );
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, dot);
    y + ROW_H_PX
}
