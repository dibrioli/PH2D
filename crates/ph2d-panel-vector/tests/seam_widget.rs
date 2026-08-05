//! Seam da **PELE POR-WIDGET** (plano UI/UX W6.2) — os dois verbos e os chips de tipo estão vivos
//! sob o MOUSE, chegam ao barramento, e cada um aparece **só onde faz sentido**.
//!
//! O gesto é REAL (Down+Up sobre o retângulo que o painel pintou), e não um `WidgetEvent::Click`
//! sintético: o sintético prova a allowlist do painel mas **pula a checagem de focabilidade no
//! store** — a lacuna que já deixou as 36 células da matriz de física e os dez chips de ferramenta
//! do Painter *pintados, hit-registrados e mortos sob o ponteiro*.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::{VectorPanelState, WidgetSkinState};
use ph2d_panel_vector::{VectorPanel, ids, state};
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

fn clear() {
    state::set_widget_skin_state(None, 0);
}

/// Três tipos, o do índice `sel` aceso (ou nenhum).
fn skin(sel: Option<usize>, unknown: bool) -> WidgetSkinState {
    WidgetSkinState {
        kinds: ["Button", "Toggle", "Slider"]
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
        selected: sel,
        unknown,
        drives: None,
    }
}

/// A mesma pele, mas de um tipo que DIRIGE — `bound` diz se já está preso a uma forma.
fn driving(sel: usize, bound: bool) -> WidgetSkinState {
    WidgetSkinState {
        drives: Some(bound.then(|| "Star".to_string())),
        ..skin(Some(sel), false)
    }
}

fn rect_under(st: WidgetSkinState, id: ph2d_a11y::NodeId) -> Option<Rect> {
    state::set_widget_skin_state(Some(st), 0);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
}

/// Clica de verdade no widget `id` e exige que o Click chegue ao barramento.
fn click_reaches_bus(st: WidgetSkinState, id: ph2d_a11y::NodeId, what: &str) {
    state::set_widget_skin_state(Some(st), 0);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
        .unwrap_or_else(|| panic!("{what} nao foi PINTADO com area clicavel"));
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
        "o ponteiro sobre {what} nao virou Click — ele esta' desenhado e nao existe para o \
         dispatcher (falta o `register` no populate)"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
    }
    assert!(
        host.drained_actions().into_iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id
        )),
        "o Click de {what} nao chegou ao bus — ele acende sob o mouse e nao faz nada (falta a \
         linha na allowlist do event_clicks)"
    );
}

/// **Os dois verbos e TODO chip pintado chegam ao bus.**
#[test]
fn the_verbs_and_every_chip_reach_the_bus() {
    clear();
    click_reaches_bus(skin(None, false), ids::VECTOR_WIDGET_WEAR, "Wear a Widget");
    click_reaches_bus(
        skin(Some(0), false),
        ids::VECTOR_WIDGET_REMOVE,
        "Back to Drawing",
    );
    for i in 0..3 {
        click_reaches_bus(
            skin(Some(0), false),
            ids::vector_widget_kind_id(i),
            &format!("o chip de tipo {i}"),
        );
    }
    clear();
}

/// **Cada verbo aparece SÓ onde faz sentido** — a metade da AUSÊNCIA.
///
/// ⚠️ Sem ela, o gate acima ficaria verde sobre uma seção que pinta os dois botões sempre, um
/// deles inerte — o botão-morto que este repo persegue e que ensina o artista a duvidar dos
/// outros.
#[test]
fn each_verb_appears_only_where_it_makes_sense() {
    clear();
    // Forma NUA: Wear sim, Remove não, e nenhum chip (não há tipo a trocar).
    assert!(rect_under(skin(None, false), ids::VECTOR_WIDGET_WEAR).is_some());
    assert!(rect_under(skin(None, false), ids::VECTOR_WIDGET_REMOVE).is_none());
    assert!(rect_under(skin(None, false), ids::vector_widget_kind_id(0)).is_none());

    // Forma VESTIDA: Remove sim, Wear não (ela já veste), e os chips existem.
    assert!(rect_under(skin(Some(1), false), ids::VECTOR_WIDGET_REMOVE).is_some());
    assert!(rect_under(skin(Some(1), false), ids::VECTOR_WIDGET_WEAR).is_none());
    assert!(rect_under(skin(Some(1), false), ids::vector_widget_kind_id(1)).is_some());
    clear();
}

/// **O tipo do FUTURO oferece DESPIR, nunca VESTIR.**
///
/// ⚠️ É o gate que impede a perda silenciosa: um *Wear* sobre uma forma que já carrega um `kind`
/// desconhecido sobrescreveria esse `kind` — trabalho apagado sem um erro. E os chips ficam fora,
/// porque acender um deles seria afirmar que a forma veste um tipo que este build não sabe qual é.
#[test]
fn a_future_kind_offers_undressing_not_dressing() {
    clear();
    assert!(rect_under(skin(None, true), ids::VECTOR_WIDGET_REMOVE).is_some());
    assert!(rect_under(skin(None, true), ids::VECTOR_WIDGET_WEAR).is_none());
    assert!(rect_under(skin(None, true), ids::vector_widget_kind_id(0)).is_none());
    clear();
}

/// **Sem estado publicado, a seção INTEIRA some** — nada de cabeçalho órfão.
#[test]
fn no_state_no_section() {
    clear();
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    for id in [
        ids::VECTOR_SECTION_WIDGET,
        ids::VECTOR_WIDGET_WEAR,
        ids::VECTOR_WIDGET_REMOVE,
    ] {
        assert!(
            host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
                .is_none(),
            "a seção pintou sem estado publicado"
        );
    }
}

/// **O conta-gotas do vínculo e o *Unbind* estão VIVOS sob o ponteiro e chegam ao barramento**
/// (W8b.3).
///
/// ⚠️ O gesto é real, e é ele que separa *"o painel pintou um retângulo"* de *"o clique existe"*.
#[test]
fn the_bind_verbs_are_alive_and_reach_the_bus() {
    clear();
    click_reaches_bus(driving(2, true), ids::VECTOR_WIDGET_BIND, "Bind Shape");
    clear();
    click_reaches_bus(driving(2, true), ids::VECTOR_WIDGET_UNBIND, "Unbind");
    clear();
}

/// **A linha do vínculo só existe para quem DIRIGE, e o *Unbind* só quando há vínculo.**
///
/// ⚠️ As duas metades: oferecer o conta-gotas num `Button` daria um gesto que resolve e não faz
/// nada; oferecer *Unbind* sem vínculo daria um clique que não muda nada — e o artista aprende a
/// não confiar na seção com um só desses.
#[test]
fn the_bind_row_exists_only_where_it_means_something() {
    clear();
    // Um tipo que NÃO dirige: nem o conta-gotas, nem o Unbind.
    assert!(rect_under(skin(Some(0), false), ids::VECTOR_WIDGET_BIND).is_none());
    clear();
    assert!(rect_under(skin(Some(0), false), ids::VECTOR_WIDGET_UNBIND).is_none());
    clear();
    // Dirige e não está preso: o conta-gotas sim, o Unbind não.
    assert!(rect_under(driving(2, false), ids::VECTOR_WIDGET_BIND).is_some());
    clear();
    assert!(rect_under(driving(2, false), ids::VECTOR_WIDGET_UNBIND).is_none());
    clear();
    // Preso: os dois.
    assert!(rect_under(driving(2, true), ids::VECTOR_WIDGET_BIND).is_some());
    clear();
    assert!(rect_under(driving(2, true), ids::VECTOR_WIDGET_UNBIND).is_some());
    clear();
}
