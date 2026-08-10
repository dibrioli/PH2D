//! ⭐ Seam da **TABELA SINAL → PAPEL** (item 4 do estudo dos contêineres).
//!
//! ⚠️ **Ele CLICA, e não apenas mede retângulos.** Um `WidgetEvent` sintético pula a checagem de
//! focabilidade da loja, e é assim que nascem trinta e seis células pintadas, hit-registradas, com
//! arm — e **mortas sob o mouse** (a cicatriz das camadas de colisão da física). O par
//! `dispatch_pointer_event(Down)` + `(Up)` dirige o ponteiro REAL pelo mesmo caminho do produto.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::{UiStatesState, VectorPanelState};
use ph2d_panel_vector::{VectorPanel, ids};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
const SEC: u128 = 1_000_000_000;

fn pointer(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

/// Uma seleção com `bindings` ligações autoradas.
fn arm(bindings: Vec<(String, usize)>) {
    ph2d_panel_vector::state::set_ui_states_state(Some(UiStatesState {
        recorded: [true, true, false, false],
        role_labels: [
            "Default".into(),
            "Hover".into(),
            "Pressed".into(),
            "Disabled".into(),
        ],
        live: None,
        duration_s: 0.15,
        easing: ph2d_anim::Easing::new(ph2d_anim::EasingFamily::Cubic, ph2d_anim::EasingMode::Out),
        spring: None,
        preview: Some(false),
        move_all: Some(true),
        bindings,
    }));
}

/// ⭐ **CADA CONTROLE DA TABELA É CLICÁVEL E CHEGA AO BARRAMENTO.**
#[test]
fn every_binding_control_reaches_the_bus() {
    arm(vec![("open".into(), 1)]);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;

    let targets = [
        ids::VECTOR_STATE_SIGNAL_ADD,
        ids::vector_state_signal_remove_id(0),
        ids::vector_state_signal_role_id(0, 2),
    ];
    for (n, id) in targets.into_iter().enumerate() {
        let r = host
            .painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
            .unwrap_or_else(|| panic!("o controle {n} da tabela nao foi pintado"));
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        let t = SEC * (n as u128 + 1);
        host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, t));
        let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, t + SEC / 100));
        assert!(
            evs.iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
            "o ponteiro sobre o controle {n} nao virou Click"
        );
        for ev in evs {
            host.apply_panel_event::<VectorPanel>(&mut st, ev);
        }
        assert!(
            host.drained_actions().into_iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id
            )),
            "o controle {n} da tabela esta' pintado e MORTO sob o mouse"
        );
    }
}

/// ⭐ **O NOME viaja no COMMIT, e pelas DUAS portas.**
///
/// ⚠️ Enter (`Submit`) **e** o campo a perder o foco (`Blur`): um campo abandonado com o nome certo
/// escrito dentro dele lê-se como *"eu autorei isto"*, e exigir o Enter faria o artista descobrir
/// a regra pelo silêncio.
#[test]
fn the_name_commits_on_enter_and_on_blur() {
    for (n, ev) in [
        WidgetEvent::Submit(ids::vector_state_signal_name_id(0)),
        WidgetEvent::Blur(ids::vector_state_signal_name_id(0)),
    ]
    .into_iter()
    .enumerate()
    {
        arm(vec![("open".into(), 0)]);
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut st = VectorPanelState;
        // Pintar UMA vez é o que semeia o buffer a partir do texto autorado (o espelho).
        let _ = host.painted_rect::<VectorPanel>(
            &mut st,
            VIEWPORT,
            ids::vector_state_signal_name_id(0),
        );
        let _ = host.drained_actions();
        host.apply_panel_event::<VectorPanel>(&mut st, ev);
        let sent = host.drained_actions().into_iter().find_map(|a| match a {
            EditorAction::ToolPanelEvent(PanelEvent::SelectOption(id, v))
                if id == ids::vector_state_signal_name_id(0) =>
            {
                Some(v)
            }
            _ => None,
        });
        assert_eq!(
            sent.as_deref(),
            Some("open"),
            "a porta {n} do commit nao entregou o nome ao barramento"
        );
    }
}

/// ⭐ **NO TETO O BOTÃO *Add* SOME** — um botão que não faz nada é pior que um botão que falta.
///
/// ⚠️ E a metade da PRESENÇA vem junto: sem ela, um painel que nunca pintasse o botão passaria.
#[test]
fn the_add_button_is_offered_below_the_pool_and_gone_at_it() {
    arm(vec![("a".into(), 0)]);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    assert!(
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_STATE_SIGNAL_ADD)
            .is_some(),
        "com uma ligacao, o *Add* tem de estar la'"
    );

    arm((0..ids::MAX_SIGNAL_BINDINGS)
        .map(|i| (format!("s{i}"), 0))
        .collect());
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    assert!(
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_STATE_SIGNAL_ADD)
            .is_none(),
        "no teto o *Add* continua pintado — um clique que nao faz nada"
    );
}

/// **A tabela fecha com a preview**, como toda a autoria desta seção: uma ligação criada ali
/// dentro perderia o passo de undo, que está suprimido enquanto o modo corre.
#[test]
fn the_table_closes_while_the_preview_runs() {
    ph2d_panel_vector::state::set_ui_states_state(Some(UiStatesState {
        recorded: [true, true, false, false],
        role_labels: [
            "Default".into(),
            "Hover".into(),
            "Pressed".into(),
            "Disabled".into(),
        ],
        live: None,
        duration_s: 0.15,
        easing: ph2d_anim::Easing::new(ph2d_anim::EasingFamily::Cubic, ph2d_anim::EasingMode::Out),
        spring: None,
        preview: Some(true),
        move_all: Some(true),
        bindings: vec![("open".into(), 1)],
    }));
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    assert!(
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_STATE_SIGNAL_ADD)
            .is_none(),
        "a tabela ficou autoravel dentro da preview"
    );
}
