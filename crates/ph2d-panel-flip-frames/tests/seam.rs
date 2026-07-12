//! **Gate anti-item-morto** da tira de frames (blindagem Fase 1.2).
//!
//! Um botão pintado, clicável e SILENCIOSAMENTE MORTO passa em todo teste de
//! unidade e em todo gate de contrato: o `populate` registrou, o `paint` desenhou,
//! e o braço no `event.rs` não existe. Estes testes rodam o caminho INTEIRO que o
//! shell roda, headless:
//!
//!   populate → click/edita → apply_event → **barramento** → `ToolPanelEvent`
//!
//! e exigem que CADA controle da barra chegue ao barramento. Uma célula/botão novo
//! na tabela abaixo sem o braço correspondente = VERMELHO.
//!
//! (O que o shell FAZ com o evento — mover o playhead, criar a chave, gerar o
//! tween — é testado no `ph2d-flip` e no `flip_strip`/`flip_autokey` do shell; aqui
//! o alvo é só a costura.)

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_panel_flip_frames::state::FlipStripState;
use ph2d_panel_flip_frames::{FlipCell, FlipFramesPanel, FlipStripSnapshot, ids};
use ph2d_ui_testkit::MockPanelHost;

/// Os eventos que a tira empurrou no barramento nesta interação.
fn drain(host: &mut MockPanelHost) -> Vec<PanelEvent> {
    host.drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::ToolPanelEvent(pe) => Some(pe),
            _ => None,
        })
        .collect()
}

/// **Todo botão da barra chega ao barramento como `Click`.** É a lista viva dos
/// controles: um botão novo entra aqui e, se o `event.rs` não o roteia, o teste cai.
#[test]
fn every_toolbar_button_reaches_the_bus() {
    let buttons = [
        ("play", ids::FLIP_PLAY),
        ("prev drawing", ids::FLIP_PREV_DRAWING),
        ("next drawing", ids::FLIP_NEXT_DRAWING),
        ("ghost", ids::FLIP_GHOST),
        ("autokey", ids::FLIP_AUTOKEY),
        ("additive", ids::FLIP_ADDITIVE),
        ("key add", ids::FLIP_KEY_ADD),
        ("key duplicate", ids::FLIP_KEY_DUP),
        ("key delete", ids::FLIP_KEY_DELETE),
        ("key left", ids::FLIP_KEY_LEFT),
        ("key right", ids::FLIP_KEY_RIGHT),
        ("tween add", ids::FLIP_TWEEN_ADD),
    ];
    for (name, id) in buttons {
        let mut host = MockPanelHost::with_panel::<FlipFramesPanel>();
        let mut state = FlipStripState;
        let outcome = host.apply_panel_event::<FlipFramesPanel>(&mut state, WidgetEvent::Click(id));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "o botão '{name}' não é roteado pelo event.rs — está MORTO"
        );
        let evs = drain(&mut host);
        assert!(
            evs.iter()
                .any(|e| matches!(e, PanelEvent::Click(i) if *i == id)),
            "o clique em '{name}' nunca chegou ao barramento — a costura está morta"
        );
    }
}

/// **Toda caixa numérica chega ao barramento como `SetValue`, com o valor.**
#[test]
fn every_number_box_reaches_the_bus_with_its_value() {
    let numbers = [
        ("fps", ids::FLIP_FPS_NUM, 12.0),
        ("ghost before", ids::FLIP_GHOST_BEFORE_NUM, 3.0),
        ("ghost after", ids::FLIP_GHOST_AFTER_NUM, 2.0),
        ("hold", ids::FLIP_HOLD_NUM, 4.0),
        ("tween count", ids::FLIP_TWEEN_NUM, 5.0),
    ];
    for (name, id, value) in numbers {
        let mut host = MockPanelHost::with_panel::<FlipFramesPanel>();
        let mut state = FlipStripState;
        host.set_number_value(id, value);
        let outcome =
            host.apply_panel_event::<FlipFramesPanel>(&mut state, WidgetEvent::ValueChanged(id));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "a caixa '{name}' não é roteada — está MORTA"
        );
        let evs = drain(&mut host);
        assert!(
            evs.iter().any(
                |e| matches!(e, PanelEvent::SetValue(i, v) if *i == id && (*v - value).abs() < 1e-6)
            ),
            "a edição de '{name}' não chegou ao barramento com o valor {value}: {evs:?}"
        );
    }
}

/// **Clicar numa CÉLULA chega ao barramento** — as células são registradas por
/// ÍNDICE, e o decodificador lê o snapshot deste frame. Se o snapshot e o paint
/// discordarem, o clique some: é este teste que pega.
#[test]
fn clicking_a_cell_reaches_the_bus() {
    let mut host = MockPanelHost::with_panel::<FlipFramesPanel>();
    let mut state = FlipStripState;
    ph2d_panel_flip_frames::set_current_flip_strip(FlipStripSnapshot {
        has_layer: true,
        cells: vec![
            FlipCell {
                key: 0,
                exposure: 4,
                breakdown: false,
                instanced: false,
            },
            FlipCell {
                key: 4,
                exposure: 1,
                breakdown: true,
                instanced: false,
            },
        ],
        ..Default::default()
    });
    let id = ids::flip_cell_id(1);
    let outcome = host.apply_panel_event::<FlipFramesPanel>(&mut state, WidgetEvent::Click(id));
    assert_eq!(outcome, EventOutcome::Consumed, "a célula não é roteada");
    assert!(
        drain(&mut host)
            .iter()
            .any(|e| matches!(e, PanelEvent::Click(i) if *i == id)),
        "o clique na célula não chegou ao barramento"
    );

    // E uma célula que NÃO existe no snapshot não vira evento (o decodificador não
    // pode inventar chave nenhuma).
    let ghost = ids::flip_cell_id(9);
    let outcome = host.apply_panel_event::<FlipFramesPanel>(&mut state, WidgetEvent::Click(ghost));
    assert_eq!(
        outcome,
        EventOutcome::Ignored,
        "uma célula inexistente não pode ser consumida"
    );
    ph2d_panel_flip_frames::set_current_flip_strip(FlipStripSnapshot::default());
}

/// A opção do dropdown de ciclo chega como `SelectOption` no id do CHIP (é o chip
/// que o shell decodifica, não a opção).
#[test]
fn picking_a_cycle_option_reaches_the_bus() {
    let mut host = MockPanelHost::with_panel::<FlipFramesPanel>();
    let mut state = FlipStripState;
    let loop_mode = 2u8;
    let outcome = host.apply_panel_event::<FlipFramesPanel>(
        &mut state,
        WidgetEvent::Click(ids::flip_cycle_option_id(loop_mode)),
    );
    assert_eq!(outcome, EventOutcome::Consumed);
    assert!(
        drain(&mut host).iter().any(|e| matches!(
            e,
            PanelEvent::SelectOption(i, v) if *i == ids::FLIP_CYCLE_DD && v == "2"
        )),
        "a escolha do ciclo não chegou ao barramento"
    );
}
