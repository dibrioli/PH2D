//! Gates do **roteamento** (plano UI/UX W8b.2) — o que uma row DIZ quando alguém a mexe.

use super::*;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{SliderState, ToggleState, WidgetKind};
use ph2d_ui_testkit::MockPanelHost;

use crate::state::{AuthoredIntent, drain_intents};

/// A row do tipo pedido, ou `None` se a tabela gerada não a contém.
fn row_of(kind: WidgetKind) -> Option<&'static crate::rows::Row> {
    crate::rows::rows().iter().find(|r| r.kind == kind)
}

/// Dirige um evento e devolve o que o painel enfileirou.
///
/// ⚠️ A fila é thread-local: sem o dreno de entrada um teste herdaria o intent do vizinho.
fn fire(store_edit: impl FnOnce(&mut WidgetStore), ev: WidgetEvent) -> Vec<AuthoredIntent> {
    let _ = drain_intents();
    let mut host = MockPanelHost::with_panel::<AuthoredPanel>();
    store_edit(host.store_mut());
    let mut st = AuthoredPanelState;
    apply_event(&mut st, &mut host, ev);
    drain_intents()
}

/// **Um slider arrastado diz QUAL chave mudou e PARA QUANTO.**
///
/// ⚠️ O valor é RELIDO do store, nunca carregado pelo evento — é o contrato do `WidgetEvent`, e é
/// o que garante que o número que sai daqui é o mesmo que o `paint` desenha no frame seguinte.
#[test]
fn a_slider_says_which_key_changed_and_to_what() {
    let Some(row) = row_of(WidgetKind::Slider) else {
        return;
    };
    let out = fire(
        |s| {
            s.register(
                row.id,
                InteractiveState::Slider {
                    state: SliderState::Normal,
                    value: 0.42,
                    orientation: Default::default(),
                },
            );
        },
        WidgetEvent::ValueChanged(row.id),
    );
    assert_eq!(
        out,
        vec![AuthoredIntent::Value {
            key: row.key.to_string(),
            value: 0.42,
        }]
    );
}

/// **Um toggle diz o estado que ele PASSOU a ter.**
#[test]
fn a_toggle_says_the_flag_it_now_carries() {
    let Some(row) = row_of(WidgetKind::Toggle) else {
        return;
    };
    let out = fire(
        |s| {
            s.register(
                row.id,
                InteractiveState::Toggle {
                    state: ToggleState::Normal,
                    on: true,
                },
            );
        },
        WidgetEvent::Toggled(row.id),
    );
    assert_eq!(
        out,
        vec![AuthoredIntent::Flag {
            key: row.key.to_string(),
            on: true,
        }]
    );
}

/// **Um botão DISPARA — sem valor, porque ele não tem nenhum.**
#[test]
fn a_button_fires_without_a_value() {
    let Some(row) = row_of(WidgetKind::Button) else {
        return;
    };
    let out = fire(|_| {}, WidgetEvent::Click(row.id));
    assert_eq!(
        out,
        vec![AuthoredIntent::Fired {
            key: row.key.to_string(),
        }]
    );
}

/// **O X fecha o painel** — a MESMA visibilidade que o interruptor da seção Frame lê.
#[test]
fn the_close_button_hides_the_panel() {
    let mut host = MockPanelHost::with_panel::<AuthoredPanel>();
    host.set_panel_visible(AuthoredPanel::ID, true);
    let mut st = AuthoredPanelState;
    apply_event(&mut st, &mut host, WidgetEvent::Click(ids::AUTHORED_CLOSE));
    assert!(
        !host.panel_visible(AuthoredPanel::ID),
        "o X nao fechou — o chip da secao Frame ficaria aceso sobre um painel escondido"
    );
}

/// **Um id que não é row nem chrome é IGNORADO.**
///
/// ⚠️ `Ignored` é o que deixa o evento seguir para o próximo handler; consumir tudo faria este
/// painel engolir cliques de quem está por baixo dele.
#[test]
fn an_id_that_is_not_a_row_is_ignored() {
    let _ = drain_intents();
    let mut host = MockPanelHost::with_panel::<AuthoredPanel>();
    let mut st = AuthoredPanelState;
    let out = apply_event(
        &mut st,
        &mut host,
        WidgetEvent::Click(ph2d_a11y::NodeId(0xDEAD_BEEF)),
    );
    assert_eq!(out, EventOutcome::Ignored);
    assert!(drain_intents().is_empty());
}
