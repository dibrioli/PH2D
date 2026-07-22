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
        ("falloff", ids::FLIP_FALLOFF),
        ("additive", ids::FLIP_ADDITIVE),
        ("key add", ids::FLIP_KEY_ADD),
        ("key duplicate", ids::FLIP_KEY_DUP),
        ("key instance", ids::FLIP_KEY_INSTANCE),
        ("key unlink", ids::FLIP_KEY_UNLINK),
        ("key delete", ids::FLIP_KEY_DELETE),
        ("key left", ids::FLIP_KEY_LEFT),
        ("key right", ids::FLIP_KEY_RIGHT),
        ("tween add", ids::FLIP_TWEEN_ADD),
        ("tween fade", ids::FLIP_TWEEN_FADE),
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
                selected: false,
                weight: 1.0,
            },
            FlipCell {
                key: 4,
                exposure: 1,
                breakdown: true,
                instanced: false,
                selected: false,
                weight: 1.0,
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

/// 🔴 **Arrastar a régua de scrub chega ao barramento como um QUADRO, não o value cru**
/// (W7.3). A mecânica de slider dá um `value` `0..1`; o `event.rs` o mapeia ao quadro pelo
/// vão exibido (`FlipStripSnapshot::scrub_frame`) e manda o QUADRO. Sem o mapa, o shell
/// receberia `1.0` (o value) e faria seek para o quadro 1 — a régua andaria errado.
///
/// Mutação que sangra: mandar `value` em vez de `scrub_frame(value)` — o barramento traz
/// `1.0`, não `8.0`.
#[test]
fn dragging_the_scrub_lane_reaches_the_bus_as_a_frame_not_a_raw_value() {
    let mut host = MockPanelHost::with_panel::<FlipFramesPanel>();
    let mut state = FlipStripState;
    // Vão exibido [0, 9): chaves 0 e 4 seguram 4 quadros, a última expõe 1.
    ph2d_panel_flip_frames::set_current_flip_strip(FlipStripSnapshot {
        has_layer: true,
        cells: vec![
            FlipCell {
                key: 0,
                exposure: 4,
                breakdown: false,
                instanced: false,
                selected: false,
                weight: 1.0,
            },
            FlipCell {
                key: 4,
                exposure: 4,
                breakdown: false,
                instanced: false,
                selected: false,
                weight: 1.0,
            },
            FlipCell {
                key: 8,
                exposure: 1,
                breakdown: false,
                instanced: false,
                selected: false,
                weight: 1.0,
            },
        ],
        ..Default::default()
    });
    // Arrasta a régua até o fim (`value = 1.0`) — o último quadro EXIBIDO é o 8 (`end − 1`).
    host.set_slider_value(ids::FLIP_SCRUB, 1.0);
    let outcome = host.apply_panel_event::<FlipFramesPanel>(
        &mut state,
        WidgetEvent::ValueChanged(ids::FLIP_SCRUB),
    );
    assert_eq!(outcome, EventOutcome::Consumed, "a régua não é roteada");
    assert!(
        drain(&mut host).iter().any(|e| matches!(
            e,
            PanelEvent::SetValue(i, v) if *i == ids::FLIP_SCRUB && (*v - 8.0).abs() < 1e-6
        )),
        "o arrasto não chegou como o QUADRO 8 (mapeou o value cru, não pelo vão?)"
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

// ─────────────────────────────────────────────────────────────────────────────
// PINTADO ≠ REGISTRADO. Tudo acima prova que o clique CHEGA. Nada acima prova que
// o controle está NA TELA para ser clicado — nenhum destes testes roda o `paint`.
// Um botão cujo desenho foi esquecido (ou mora atrás de um `return`) passa em
// TODOS eles e o usuário relata, com razão, "esse botão não existe".
// ─────────────────────────────────────────────────────────────────────────────

/// **Todo controle da barra é PINTADO com área clicável.**
///
/// Mutação que sangra: remova qualquer botão do `paint_toolbar.rs` e este teste
/// cai — enquanto `every_toolbar_button_reaches_the_bus` segue verde, porque a
/// costura do evento continua intacta. São perguntas diferentes.
#[test]
fn every_toolbar_control_is_actually_painted() {
    let mut host = MockPanelHost::with_panel::<FlipFramesPanel>();
    let mut state = FlipStripState;

    // A tira só desenha os controles com uma camada viva — é o estado que o
    // `flip_bridge` publica quando a tool está ativa.
    ph2d_panel_flip_frames::set_current_flip_strip(FlipStripSnapshot {
        has_layer: true,
        layer_name: "L".into(),
        cells: vec![FlipCell {
            key: 0,
            exposure: 1,
            breakdown: false,
            instanced: false,
            selected: false,
            weight: 1.0,
        }],
        fps: 12.0,
        ..Default::default()
    });

    let painted = host.paint::<FlipFramesPanel>(
        &mut state,
        ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1600.0, 900.0),
    );

    let controls = [
        ("play", ids::FLIP_PLAY),
        ("prev drawing", ids::FLIP_PREV_DRAWING),
        ("next drawing", ids::FLIP_NEXT_DRAWING),
        ("ghost", ids::FLIP_GHOST),
        ("autokey", ids::FLIP_AUTOKEY),
        ("falloff", ids::FLIP_FALLOFF),
        ("additive", ids::FLIP_ADDITIVE),
        ("key add", ids::FLIP_KEY_ADD),
        ("key duplicate", ids::FLIP_KEY_DUP),
        ("key instance", ids::FLIP_KEY_INSTANCE),
        ("key unlink", ids::FLIP_KEY_UNLINK),
        ("key delete", ids::FLIP_KEY_DELETE),
        ("key left", ids::FLIP_KEY_LEFT),
        ("key right", ids::FLIP_KEY_RIGHT),
        ("tween add", ids::FLIP_TWEEN_ADD),
        ("fps", ids::FLIP_FPS_NUM),
        ("hold", ids::FLIP_HOLD_NUM),
        ("tween count", ids::FLIP_TWEEN_NUM),
        ("tween ease", ids::FLIP_TWEEN_EASE_DD),
        ("tween fade", ids::FLIP_TWEEN_FADE),
        ("cycle", ids::FLIP_CYCLE_DD),
    ];
    for (name, id) in controls {
        let hit = painted
            .iter()
            .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0);
        assert!(
            hit.is_some(),
            "o controle '{name}' NAO e pintado com area clicavel: nao existe na tela"
        );
    }

    // E a célula do quadro 0 tem de estar lá — é o alvo de clique do usuário.
    let cell = ph2d_editor_core::ids::flip_cell_id(0);
    assert!(
        painted
            .iter()
            .any(|(w, r)| *w == cell && r.w > 0.0 && r.h > 0.0),
        "a celula do quadro 0 nao e pintada: a tira esta vazia na tela"
    );
    ph2d_panel_flip_frames::set_current_flip_strip(FlipStripSnapshot::default());
}

/// 🔴 **A régua de scrub é PINTADA com área clicável** (W7.3). O Slider está no `populate`
/// e roteado no `event.rs`, mas nada disso a coloca NA TELA: se `paint_cells` esquecer de
/// registrar o rect (`hit_index`), a régua não existe para o ponteiro — e o
/// `dragging_the_scrub_lane_reaches_the_bus` seguiria verde (ele injeta o value direto,
/// pulando o hit). Pergunta diferente: aqui é o pixel.
///
/// Mutação que sangra: remover o `register(FLIP_SCRUB, lane)` de `paint_scrub_lane`.
#[test]
fn the_scrub_lane_is_painted_with_a_hittable_rect() {
    let mut host = MockPanelHost::with_panel::<FlipFramesPanel>();
    let mut state = FlipStripState;
    ph2d_panel_flip_frames::set_current_flip_strip(FlipStripSnapshot {
        has_layer: true,
        layer_name: "L".into(),
        cells: vec![
            FlipCell {
                key: 0,
                exposure: 4,
                breakdown: false,
                instanced: false,
                selected: false,
                weight: 1.0,
            },
            FlipCell {
                key: 4,
                exposure: 1,
                breakdown: false,
                instanced: false,
                selected: false,
                weight: 1.0,
            },
        ],
        fps: 12.0,
        ..Default::default()
    });

    let painted = host.paint::<FlipFramesPanel>(
        &mut state,
        ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1600.0, 900.0),
    );
    let lane = painted
        .iter()
        .find(|(w, r)| *w == ids::FLIP_SCRUB && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
        .expect("a régua de scrub não é pintada com área clicável: não existe para o ponteiro");

    // 🔴 **A régua tem BANDA PRÓPRIA acima das células** — ela não pode se sobrepor à
    // célula 0 (senão estaria roubando a altura dos frames, o que deixou a tira "apertada
    // na vertical"; Enio 2026-07-14). O painel cresce por `scrub_reserved_h()` justamente
    // para a régua morar num espaço seu. E as células continuam sendo a banda DOMINANTE (a
    // régua é um pega-mão fino, não a metade da tira).
    let cell0 = painted
        .iter()
        .find(|(w, r)| *w == ph2d_editor_core::ids::flip_cell_id(0) && r.h > 0.0)
        .map(|(_, r)| *r)
        .expect("a célula 0 não foi pintada");
    assert!(
        lane.y + lane.h <= cell0.y + 0.5,
        "a régua invade a célula 0 (régua até y={}, célula começa em y={}) — está roubando a altura dos frames",
        lane.y + lane.h,
        cell0.y
    );
    assert!(
        cell0.h > lane.h,
        "a célula ({}) ficou mais baixa que a régua ({}) — os frames foram espremidos",
        cell0.h,
        lane.h
    );
    ph2d_panel_flip_frames::set_current_flip_strip(FlipStripSnapshot::default());
}

/// **A escolha de easing chega ao barramento, com o id do SEU chip.** A barra tem dois
/// dropdowns e as duas listas de opção convivem no mesmo roteador — mandar o
/// `SelectOption` com o id do outro chip despacharia a escolha para o campo errado
/// (o ciclo da camada viraria o easing do tween, em silêncio).
#[test]
fn picking_a_tween_easing_reaches_the_bus_under_its_own_chip() {
    for preset in 0u8..4 {
        let mut host = MockPanelHost::with_panel::<FlipFramesPanel>();
        let mut state = FlipStripState;
        let outcome = host.apply_panel_event::<FlipFramesPanel>(
            &mut state,
            WidgetEvent::Click(ids::flip_tween_ease_option_id(preset)),
        );
        assert_eq!(outcome, EventOutcome::Consumed);
        let want = preset.to_string();
        assert!(
            drain(&mut host).iter().any(|e| matches!(
                e,
                PanelEvent::SelectOption(i, v)
                    if *i == ids::FLIP_TWEEN_EASE_DD && *v == want
            )),
            "o preset {preset} de easing não chegou ao barramento sob o próprio chip"
        );
    }
}

/// **Abrir um dropdown FECHA o outro.** Dois popovers abertos ao mesmo tempo é um estado
/// que ninguém pediu — e só um deles chega a ser pintado, então o segundo ficaria
/// "aberto" e invisível, comendo o próximo clique.
#[test]
fn opening_one_dropdown_closes_the_other() {
    for (first, second) in [
        (ids::FLIP_CYCLE_DD, ids::FLIP_TWEEN_EASE_DD),
        (ids::FLIP_TWEEN_EASE_DD, ids::FLIP_CYCLE_DD),
    ] {
        let mut host = MockPanelHost::with_panel::<FlipFramesPanel>();
        let mut state = FlipStripState;
        // O chip é registrado pelo PAINT (é ele que sabe o rect), então o paint roda antes.
        // O chip é registrado pelo PAINT (é ele que sabe o rect), então o paint roda antes.
        ph2d_panel_flip_frames::set_current_flip_strip(FlipStripSnapshot {
            has_layer: true,
            layer_name: "L".into(),
            cells: vec![FlipCell {
                key: 0,
                exposure: 1,
                breakdown: false,
                instanced: false,
                selected: false,
                weight: 1.0,
            }],
            fps: 12.0,
            ..Default::default()
        });
        host.paint::<FlipFramesPanel>(
            &mut state,
            ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1600.0, 900.0),
        );
        host.set_dropdown_open(first, true);
        host.set_dropdown_open(second, false);

        host.apply_panel_event::<FlipFramesPanel>(&mut state, WidgetEvent::Click(second));
        assert_eq!(
            host.dropdown_is_open(first),
            Some(false),
            "o popover do outro chip continuou aberto"
        );
    }
}
