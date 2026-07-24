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
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerEvent, PointerKind, PointerSource};
use ph2d_panel_flip_frames::FlipStripIntent;

/// Três chaves em 0/4/8, cada uma expondo 4 quadros — a fixture das células.
fn strip_snapshot() -> FlipStripSnapshot {
    let cell = |key: i32| FlipCell {
        key,
        exposure: 4,
        breakdown: false,
        instanced: false,
        selected: false,
        weight: 1.0,
    };
    FlipStripSnapshot {
        has_layer: true,
        cells: vec![cell(0), cell(4), cell(8)],
        ..Default::default()
    }
}

/// Pinta e devolve o rect da célula `i` — **o que o artista vê é o que o paint registrou**.
fn cell_rect(
    host: &mut MockPanelHost,
    state: &mut FlipStripState,
    viewport: Rect,
    i: usize,
) -> Rect {
    host.paint::<FlipFramesPanel>(state, viewport)
        .into_iter()
        .find(|(id, r)| *id == ids::flip_cell_id(i) && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| r)
        .expect("a célula tem de estar pintada e clicável")
}

/// Um arrasto REAL: Down, um Move além da folga, Up — pelo dispatcher do shell.
fn drag(host: &mut MockPanelHost, x0: f32, y0: f32, x1: f32, y1: f32) {
    let at = |x: f32, y: f32, kind: PointerKind| PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: ph2d_host::PointerButton::Primary,
        timestamp_ns: 0,
    };
    host.dispatch_pointer_event(at(x0, y0, PointerKind::Down));
    host.dispatch_pointer_event(at(x1, y1, PointerKind::Move));
    host.dispatch_pointer_event(at(x1, y1, PointerKind::Up));
}

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
        let mut state = FlipStripState::default();
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
        let mut state = FlipStripState::default();
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

/// 🔴 **Um TOQUE numa célula chega ao barramento — pelo ponteiro REAL.**
///
/// Este gate mudou de caminho junto com o produto: a célula deixou de ser um botão e virou
/// **superfície de gesto**, então o `WidgetEvent::Click` que a versão anterior deste teste
/// entregava ao `apply_event` **não é mais o que acontece** — o dispatch captura o Down na
/// superfície, o Up volta como `GesturePhase::Click`, e o `strip_drag` o traduz no MESMO
/// `PanelEvent::Click(flip_cell_id(i))` de sempre.
///
/// ⚠️ Mantê-lo como estava teria deixado o gate **verde sobre um caminho morto** (ele
/// passava: o braço antigo ainda existia no `event.rs`, sem nunca mais rodar no produto).
/// Por isso ele agora PINTA, CLICA com o ponteiro do dispatcher, e pinta de novo — é o
/// segundo paint que drena o gesto.
#[test]
fn tapping_a_cell_reaches_the_bus_through_the_real_pointer() {
    let mut host = MockPanelHost::with_panel::<FlipFramesPanel>();
    let mut state = FlipStripState::default();
    ph2d_panel_flip_frames::set_current_flip_strip(strip_snapshot());

    let viewport = Rect::new(0.0, 0.0, 1280.0, 800.0);
    let cell = cell_rect(&mut host, &mut state, viewport, 1);
    host.click_at(cell.x + cell.w * 0.5, cell.y + cell.h * 0.5);
    host.paint::<FlipFramesPanel>(&mut state, viewport); // o paint que DRENA o gesto

    assert!(
        drain(&mut host)
            .iter()
            .any(|e| matches!(e, PanelEvent::Click(i) if *i == ids::flip_cell_id(1))),
        "o toque na célula não chegou ao barramento"
    );
    ph2d_panel_flip_frames::set_current_flip_strip(FlipStripSnapshot::default());
}

/// 🔴 **Arrastar a célula pede o `MoveKey`** — a costura inteira, com o ponteiro real:
/// paint (registra a superfície) → Down → Move além da folga → Up → paint (drena) →
/// `drain_flip_strip_intents`.
///
/// É o gate que prova que a célula não está **morta sob o mouse**: ela pode pintar,
/// registrar hit e ter braço no painel e ainda assim nunca virar gesto, porque o Down do
/// dispatcher só ativa um id que carrega `InteractiveState` no store. Foi isso que matou o
/// rig de luz do Impasto com a suíte verde.
#[test]
fn dragging_a_cell_asks_the_document_to_move_the_key() {
    let mut host = MockPanelHost::with_panel::<FlipFramesPanel>();
    let mut state = FlipStripState::default();
    ph2d_panel_flip_frames::set_current_flip_strip(strip_snapshot());
    let _ = ph2d_panel_flip_frames::drain_flip_strip_intents();

    let viewport = Rect::new(0.0, 0.0, 1280.0, 800.0);
    let cell = cell_rect(&mut host, &mut state, viewport, 1);
    let y = cell.y + cell.h * 0.5;
    let x0 = cell.x + cell.w * 0.5;
    // A célula do meio (chave 4) expõe 4 quadros: uma largura de célula à direita = +4.
    drag(&mut host, x0, y, x0 + cell.w, y);
    host.paint::<FlipFramesPanel>(&mut state, viewport);

    let intents = ph2d_panel_flip_frames::drain_flip_strip_intents();
    assert!(
        matches!(
            intents.as_slice(),
            [FlipStripIntent::MoveKey { from: 4, to }] if *to > 4
        ),
        "o arrasto não pediu para mover a chave: {intents:?}"
    );
    ph2d_panel_flip_frames::set_current_flip_strip(FlipStripSnapshot::default());
}

/// 🔴 **Arrastar a BORDA da célula pede a exposição, não o movimento.** Os dois alvos se
/// sobrepõem em pixels (o grip mora dentro da célula), então quem decide é a ORDEM de
/// registro no hit index — e é ela que este gate pina. Trocar a ordem faz o grip virar
/// código morto: pintado, sobreposto e inalcançável.
#[test]
fn dragging_the_cells_edge_asks_for_the_exposure_instead() {
    let mut host = MockPanelHost::with_panel::<FlipFramesPanel>();
    let mut state = FlipStripState::default();
    ph2d_panel_flip_frames::set_current_flip_strip(strip_snapshot());
    let _ = ph2d_panel_flip_frames::drain_flip_strip_intents();

    let viewport = Rect::new(0.0, 0.0, 1280.0, 800.0);
    let cell = cell_rect(&mut host, &mut state, viewport, 0);
    let y = cell.y + cell.h * 0.5;
    // 2 px para dentro da borda direita: dentro do grip, e ainda dentro da célula.
    let x0 = cell.x + cell.w - 2.0;
    drag(&mut host, x0, y, x0 + cell.w, y);
    host.paint::<FlipFramesPanel>(&mut state, viewport);

    let intents = ph2d_panel_flip_frames::drain_flip_strip_intents();
    assert!(
        matches!(
            intents.as_slice(),
            [FlipStripIntent::SetHold { key: 0, frames }] if *frames > 4
        ),
        "o arrasto da borda não pediu a exposição: {intents:?}"
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
    let mut state = FlipStripState::default();
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
    let mut state = FlipStripState::default();
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
    let mut state = FlipStripState::default();

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
    let mut state = FlipStripState::default();
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
        let mut state = FlipStripState::default();
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
        let mut state = FlipStripState::default();
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
