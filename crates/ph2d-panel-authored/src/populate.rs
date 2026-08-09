//! O registro dos widgets — percorre a MESMA lista que o `paint`.
//!
//! ⚠️ Um widget que o `paint` põe no índice de hit e que ninguém regista tem `is_focusable() ==
//! false`, e o clique dele é descartado **em silêncio**: sem erro de compilação, sem warning, só
//! um controle que não faz nada. É a classe que o `architecture_panel_wiring_parity` existe para
//! pegar — e derivar esta lista da tabela que o `paint` percorre é o que a torna algo que ninguém
//! pode esquecer, nem o gerador.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{
    ButtonState, CheckboxState, CheckboxValue, ListItemState, SliderOrientation, SliderState,
    TagState, TextInputState, ToggleState, WidgetKind, format_number,
};

use crate::rows::{Row, rows};

/// O estado inicial de uma row que RESPONDE, ou `None` para as que só desenham.
///
/// ⚠️ **O valor inicial é o neutro do tipo, não um valor "bonito".** A prévia do canvas mostra um
/// slider a meio porque um slider a zero é indistinguível de uma trilha quebrada — mas ali não há
/// gesto: é uma foto. Aqui há, e um painel que nasce com meia opacidade afirmaria um valor que o
/// artista não escolheu. O zero é honesto: nada foi mexido ainda.
fn initial(kind: WidgetKind) -> Option<InteractiveState> {
    Some(match kind {
        // ⚠️ **Os dois botões partilham o estado, e isso não é atalho:** um botão de ícone é um
        // botão cuja face é um glifo — o que muda é o que ele DESENHA, nunca o que ele SENTE. Um
        // `InteractiveState::IconButton` seria um segundo nome para *pressionado*, e o pintor
        // teria de aceitar os dois.
        WidgetKind::Button | WidgetKind::IconButton => InteractiveState::Button {
            state: ButtonState::Normal,
        },
        WidgetKind::Toggle => InteractiveState::Toggle {
            state: ToggleState::Normal,
            on: false,
        },
        WidgetKind::Checkbox => InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Unchecked,
        },
        WidgetKind::Slider => InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.0,
            orientation: SliderOrientation::Horizontal,
        },
        WidgetKind::TextInput => InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
        WidgetKind::Tag => InteractiveState::Tag {
            state: TagState::Normal,
        },
        WidgetKind::ListItem => InteractiveState::ListItem {
            state: ListItemState::Normal,
            selected: false,
        },
        WidgetKind::NumberInput => InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 0.0,
            // ⚠️ O buffer ESPELHA o valor, e a formatação sai da porta do próprio widget: uma
            // segunda regra aqui faria o campo nascer mostrando um texto que o pintor não
            // escreveria, e a divergência só apareceria no primeiro foco.
            buffer: format_number(0.0),
            caret: 0,
            last_committed: 0.0,
            selection_anchor: None,
        },
        // Os que só desenham. ⚠️ `None` é a resposta, e o `is_control` é quem a decide — ver o
        // doc dele: três cópias desta pergunta divergiriam com modos de falha diferentes.
        //
        // ⚠️ O `LevelMeter` está aqui por NATUREZA, não por omissão: ele é um READOUT (nível de
        // áudio), e não existe `InteractiveState::LevelMeter` para registar. Um medidor que
        // acendesse ao clique seria o item-de-menu-morto pintado com outro nome.
        WidgetKind::ProgressBar
        | WidgetKind::SectionHeader
        | WidgetKind::Card
        | WidgetKind::Spinner
        | WidgetKind::Divider
        | WidgetKind::LevelMeter
        | WidgetKind::ColorSwatch => return None,
    })
}

fn button(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

/// Os punhos de chrome — mover e redimensionar.
///
/// ⚠️ Eles moram AQUI, e não no `pre_populate` do hero: aquele arquivo os regista para o Inspector
/// e a Hierarquia com um comentário dizendo que *"movem para o `populate` do painel quando a crate
/// possuir a alocação de NodeId"* — esta possui, então esta o faz.
fn chrome(store: &mut WidgetStore) {
    for (id, kind) in [
        (ids::AUTHORED_DRAG_HANDLE, BlenderHitKind::DragHandle),
        (ids::AUTHORED_RESIZE_HANDLE, BlenderHitKind::ResizeHandle),
        (
            ids::AUTHORED_RESIZE_HANDLE_BL,
            BlenderHitKind::ResizeHandleBl,
        ),
    ] {
        store.register(
            id,
            InteractiveState::BlenderHit {
                parent: ids::AUTHORED_PANEL,
                kind,
            },
        );
    }
}

pub fn populate(store: &mut WidgetStore) {
    button(store, ids::AUTHORED_CLOSE);
    chrome(store);
    for row in rows() {
        // A pergunta é feita ao `is_control`, e o `initial` a espelha. As duas concordam por
        // construção — há gate a exigi-lo, porque uma delas mudar sozinha é o caminho para um
        // controle registado que o `paint` não desenha (ou o contrário).
        if let Some(st) = initial(row.kind) {
            debug_assert!(Row::is_control(row), "registado sem ser controle");
            store.register(row.id, st);
        }
        // ⚠️ **Um cabeçalho não é registado como widget** — ele não tem `InteractiveState`, e o
        // despacho o trata ANTES do `switch` justamente por isso. O que ele precisa é de ser
        // MARCADO: sem esta linha o chevron desenha, o retângulo de hit existe, o clique chega —
        // e não dobra nada, porque o despacho não sabe que aquele id é uma seção.
        if row.folds_a_section() {
            store.mark_collapsible_section(row.id);
        }
    }
}

#[cfg(test)]
#[path = "populate_tests.rs"]
mod tests;
